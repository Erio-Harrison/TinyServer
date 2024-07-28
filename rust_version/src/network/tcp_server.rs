use crate::core::Reactor;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::unix::io::{AsRawFd, RawFd, FromRawFd};
use std::sync::{Arc, Mutex};

const BUFFER_SIZE: usize = 1024;

struct TcpServerInner {
    reactor: Arc<Mutex<Reactor>>,
    running: bool,
    connection_handler: Option<Arc<dyn Fn(RawFd) + Send + Sync>>,
    receive_handler: Option<Arc<dyn Fn(RawFd, &[u8]) + Send + Sync>>,
}

pub struct TcpServer {
    inner: Arc<Mutex<TcpServerInner>>,
    listener: TcpListener,
}

impl TcpServer {
    pub fn new(reactor: Arc<Mutex<Reactor>>, ip: &str, port: u16) -> io::Result<Self> {
        let addr = format!("{}:{}", ip, port);
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;

        Ok(TcpServer {
            inner: Arc::new(Mutex::new(TcpServerInner {
                reactor,
                running: false,
                connection_handler: None,
                receive_handler: None,
            })),
            listener,
        })
    }

    pub fn start(&self) -> io::Result<()> {
        let inner = self.inner.clone();
        let listener_fd = self.listener.as_raw_fd();

        let accept_handler = move || {
            let inner_clone = inner.clone();
            if let Ok(mut inner_guard) = inner_clone.lock() {
                if inner_guard.running {
                    // 在这里解锁 inner_guard，避免在 accept_connection 中持有锁
                    drop(inner_guard);
                    if let Err(e) = Self::accept_connection(&inner_clone, listener_fd) {
                        eprintln!("Error accepting connection: {:?}", e);
                    }
                }
            }
            Ok(())
        };

        // 设置 running 标志并添加处理器
        let mut inner_guard = self.inner.lock().unwrap();
        inner_guard.running = true;
        inner_guard.reactor.lock().unwrap().add_handler(listener_fd, Box::new(accept_handler))?;
        
        Ok(())
    }

    pub fn stop(&self) -> io::Result<()> {
        let mut inner = self.inner.lock().unwrap();
        inner.running = false;
        inner.reactor.lock().unwrap().remove_handler(self.listener.as_raw_fd())
    }

    pub fn set_connection_handler<F>(&self, handler: F)
    where
        F: Fn(RawFd) + Send + Sync + 'static,
    {
        let mut inner = self.inner.lock().unwrap();
        inner.connection_handler = Some(Arc::new(handler));
    }

    pub fn set_receive_handler<F>(&self, handler: F)
    where
        F: Fn(RawFd, &[u8]) + Send + Sync + 'static,
    {
        let mut inner = self.inner.lock().unwrap();
        inner.receive_handler = Some(Arc::new(handler));
    }

    pub fn send(&self, client_fd: RawFd, data: &[u8]) -> io::Result<usize> {
        let mut stream = unsafe { TcpStream::from_raw_fd(client_fd) };
        let result = stream.write(data);
        std::mem::forget(stream);  // 防止 stream 被 drop，因为我们不拥有这个 fd
        result
    }

    fn accept_connection(inner: &Arc<Mutex<TcpServerInner>>, listener_fd: RawFd) -> io::Result<()> {
        let listener = unsafe { TcpListener::from_raw_fd(listener_fd) };
        match listener.accept() {
            Ok((stream, _)) => {
                stream.set_nonblocking(true)?;
                let client_fd = stream.as_raw_fd();

                let read_handler = {
                    let inner = inner.clone();
                    move || {
                        if let Ok(inner_guard) = inner.lock() {
                            // 在这里解锁 inner_guard，避免在 handle_read 中持有锁
                            drop(inner_guard);
                            if let Err(e) = Self::handle_read(&inner, client_fd) {
                                eprintln!("Error handling read: {:?}", e);
                            }
                        }
                        Ok(())
                    }
                };

                let inner_guard = inner.lock().unwrap();
                inner_guard.reactor.lock().unwrap().add_handler(client_fd, Box::new(read_handler))?;

                if let Some(handler) = &inner_guard.connection_handler {
                    handler(client_fd);
                }

                std::mem::forget(stream);  // 防止 stream 被 drop，因为我们不拥有这个 fd
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(e) => return Err(e),
        }
        std::mem::forget(listener);  // 防止 listener 被 drop
        Ok(())
    }

    fn handle_read(inner: &Arc<Mutex<TcpServerInner>>, client_fd: RawFd) -> io::Result<()> {
        let mut buffer = [0u8; BUFFER_SIZE];
        let mut stream = unsafe { TcpStream::from_raw_fd(client_fd) };

        match stream.read(&mut buffer) {
            Ok(0) => {
                // 连接关闭
                let mut inner_guard = inner.lock().unwrap();
                inner_guard.reactor.lock().unwrap().remove_handler(client_fd)?;
                if let Some(handler) = &inner_guard.connection_handler {
                    handler(-client_fd);  // 使用负值表示连接关闭
                }
            }
            Ok(n) => {
                let inner_guard = inner.lock().unwrap();
                if let Some(handler) = &inner_guard.receive_handler {
                    handler(client_fd, &buffer[..n]);
                }
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(e) => return Err(e),
        }

        std::mem::forget(stream);  // 防止 stream 被 drop，因为我们不拥有这个 fd
        Ok(())
    }

    pub fn close(&self) -> io::Result<()> {
        let mut inner = self.inner.lock().unwrap();
        if inner.running {
            inner.running = false;
            inner.reactor.lock().unwrap().remove_handler(self.listener.as_raw_fd())?;
        }
        Ok(())
    }
}