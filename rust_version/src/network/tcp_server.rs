use crate::core::reactor::Reactor;
use std::io;
use std::os::unix::io::{AsRawFd, RawFd, IntoRawFd};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

const BUFFER_SIZE: usize = 1024;

struct ServerState {
    reactor: Reactor,
    receive_handler: Option<Box<dyn FnMut(RawFd, &[u8], usize) + Send>>,
}

#[allow(dead_code)]
pub struct TcpServer {
    state: Arc<Mutex<ServerState>>,
    ip: String,
    port: u16,
    server_fd: RawFd,
    running: Arc<AtomicBool>,
}

impl TcpServer {
    pub fn new(reactor: Reactor, ip: &str, port: u16) -> io::Result<Self> {
        let addr = format!("{}:{}", ip, port);
        let listener = TcpListener::bind(&addr)?;
        
        listener.set_nonblocking(true)?;
        let listen_fd = listener.as_raw_fd();
        
        unsafe {
            let opt: libc::c_int = 1;
            if libc::setsockopt(
                listen_fd,
                libc::SOL_SOCKET,
                libc::SO_REUSEADDR,
                &opt as *const _ as *const libc::c_void,
                std::mem::size_of_val(&opt) as libc::socklen_t,
            ) == -1 {
                return Err(io::Error::last_os_error());
            }
        }
        
        let _ = listener.into_raw_fd();
        
        Ok(TcpServer {
            state: Arc::new(Mutex::new(ServerState {
                reactor,
                receive_handler: None,
            })),
            ip: ip.to_string(),
            port,
            server_fd: listen_fd,
            running: Arc::new(AtomicBool::new(false)),
        })
    }

    fn handle_read(client_fd: RawFd, state: &Arc<Mutex<ServerState>>) -> io::Result<()> {
        let mut buffer = [0u8; BUFFER_SIZE];
        
        unsafe {
            let bytes_read = libc::read(
                client_fd,
                buffer.as_mut_ptr() as *mut libc::c_void,
                buffer.len(),
            );
            
            if bytes_read > 0 {
                if let Some(ref mut handler) = state.lock().unwrap().receive_handler {
                    handler(client_fd, &buffer[..bytes_read as usize], bytes_read as usize);
                }
            } else if bytes_read == 0 || (bytes_read == -1 && io::Error::last_os_error().kind() != io::ErrorKind::WouldBlock) {
                Self::handle_close(client_fd, state)?;
            }
        }
        Ok(())
    }

    fn handle_close(client_fd: RawFd, state: &Arc<Mutex<ServerState>>) -> io::Result<()> {
        state.lock().unwrap().reactor.remove_handler(client_fd)?;
        unsafe {
            if libc::close(client_fd) == -1 {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(())
    }

    pub fn accept_connection(&mut self) -> io::Result<()> {
        unsafe {
            let mut client_addr: libc::sockaddr_in = std::mem::zeroed();
            let mut client_len = std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t;
            
            let client_fd = libc::accept(
                self.server_fd,
                &mut client_addr as *mut _ as *mut libc::sockaddr,
                &mut client_len,
            );
            
            if client_fd == -1 {
                let err = io::Error::last_os_error();
                if err.kind() != io::ErrorKind::WouldBlock {
                    eprintln!("Failed to accept connection: {}", err);
                }
                return Ok(());
            }
            
            let flags = libc::fcntl(client_fd, libc::F_GETFL, 0);
            if flags == -1 {
                return Err(io::Error::last_os_error());
            }
            if libc::fcntl(client_fd, libc::F_SETFL, flags | libc::O_NONBLOCK) == -1 {
                return Err(io::Error::last_os_error());
            }

            let state = Arc::clone(&self.state);
            self.state.lock().unwrap().reactor.add_handler(
                client_fd,
                (libc::EPOLLIN | libc::EPOLLRDHUP) as u32,
                Box::new(move |events| {
                    if events & (libc::EPOLLIN as u32) != 0 {
                        if let Err(e) = Self::handle_read(client_fd, &state) {
                            eprintln!("Error handling read: {}", e);
                        }
                    }
                    if events & ((libc::EPOLLRDHUP | libc::EPOLLHUP) as u32) != 0 {
                        if let Err(e) = Self::handle_close(client_fd, &state) {
                            eprintln!("Error closing connection: {}", e);
                        }
                    }
                }),
            )?;
        }
        Ok(())
    }

    pub fn start(&mut self) -> io::Result<()> {
        self.running.store(true, Ordering::SeqCst);
        let state = Arc::clone(&self.state);
        let server_fd = self.server_fd;
        let running = Arc::clone(&self.running);

        self.state.lock().unwrap().reactor.add_handler(
            self.server_fd,
            libc::EPOLLIN as u32,
            Box::new(move |events| {
                if events & (libc::EPOLLIN as u32) != 0 {
                    let mut server = TcpServer {
                        state: Arc::clone(&state),
                        ip: String::new(),
                        port: 0,
                        server_fd,
                        running: running.clone(),
                    };
                    if let Err(e) = server.accept_connection() {
                        eprintln!("Accept error happened: {}", e);
                    }
                }
            }),
        )?;
        
        Ok(())
    }

    pub fn stop(&mut self) -> io::Result<()> {
        println!("TcpServer stopping...");
        self.running.store(false, Ordering::SeqCst);
        self.state.lock().unwrap().reactor.remove_handler(self.server_fd)?;
        self.state.lock().unwrap().reactor.stop();
        println!("TcpServer stopped");
        Ok(())
    }

    pub fn set_receive_handler<F>(&mut self, handler: F)
    where
        F: FnMut(RawFd, &[u8], usize) + Send + 'static,
    {
        self.state.lock().unwrap().receive_handler = Some(Box::new(handler));
    }

    pub fn send(&self, client_fd: RawFd, data: &[u8]) -> io::Result<usize> {
        unsafe {
            let sent = libc::send(
                client_fd,
                data.as_ptr() as *const libc::c_void,
                data.len(),
                0,
            );
            if sent == -1 {
                return Err(io::Error::last_os_error());
            }
            Ok(sent as usize)
        }
    }

    pub fn get_reactor(&self) -> Reactor {
        let mut reactor = self.state.lock().unwrap().reactor.clone();
        reactor.share_running_state(Arc::clone(&self.running));
        reactor
    }
}

impl Drop for TcpServer {
    fn drop(&mut self) {
        if self.running.load(Ordering::SeqCst) {
            let _ = self.stop();
        }
        unsafe {
            let _ = libc::close(self.server_fd);
        }
    }
}