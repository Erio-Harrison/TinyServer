use std::collections::HashMap;
use std::io::{Error, Result};
use std::os::unix::io::RawFd;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const MAX_EVENTS : usize = 10;

pub struct Reactor{
    epoll_fd : RawFd,
    running: Arc<AtomicBool>,
    handlers: HashMap<RawFd, Box<dyn FnMut(u32) + Send>>,
}

impl Reactor{
    pub fn new() -> Result<Self> {
        let epoll_fd = unsafe { libc ::epoll_create1(0)};
        if epoll_fd == -1 {
            return Err(Error::last_os_error());
        }

        Ok(
            Reactor { 
            epoll_fd, 
            running: Arc::new(AtomicBool::new(false)), 
            handlers:HashMap::new() 
            }
        )
    }

    pub fn add_handler<F>(&mut self, fd: RawFd, events: u32, handler: F) -> Result<()>
    where
        F: FnMut(u32) + Send + 'static,
    {
        let mut ev = libc::epoll_event{
            events : events as u32,
            u64: fd as u64,
        };

        let result = unsafe{
            libc::epoll_ctl(
                self.epoll_fd,
                libc::EPOLL_CTL_ADD,
                fd,
                &mut ev as *mut libc::epoll_event,
            )
        };

        if result == -1 {
            return Err(Error::last_os_error());
        }
        
        self.handlers.insert(fd, Box::new(handler));

        Ok(())
    }

    pub fn remove_handler(&mut self, fd: RawFd) -> Result<()> {
        if !self.handlers.contains_key(&fd){
            return Ok(());
        }

        let result = unsafe {
            libc::epoll_ctl(
                self.epoll_fd,
                libc::EPOLL_CTL_DEL,
                fd,
                std::ptr::null_mut()
            )
        };

        match result {
            -1 => {
                let err = Error::last_os_error();
                match err.raw_os_error() {
                    Some(libc::EBADF) => {
                        self.handlers.remove(&fd);
                        Ok(())
                    }

                    Some(libc::ENOENT) => {
                        eprintln!("Warning: File descriptor {} not found in epoll instance", fd);
                        self.handlers.remove(&fd);
                        Ok(())
                    }
                    _ => Err(err)
                }
            }
            _ => {
                self.handlers.remove(&fd);
                Ok(())
            }
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let mut events = vec![
            libc::epoll_event { events: 0, u64: 0 };
            MAX_EVENTS
        ];

        self.running.store(true, Ordering::SeqCst);

        while self.running.load(Ordering::SeqCst) {
            let nfds = unsafe {
                libc::epoll_wait(
                    self.epoll_fd,
                    events.as_mut_ptr(),
                    MAX_EVENTS as i32,
                    100, 
                )
            };

            if nfds == -1 {
                let err = Error::last_os_error();
                if err.raw_os_error() == Some(libc::EINTR) {
                    continue;
                }
                return Err(err);
            }

            for n in 0..nfds {
                let fd = events[n as usize].u64 as RawFd;
                if let Some(handler) = self.handlers.get_mut(&fd) {
                    handler(events[n as usize].events);
                }
            }
        }
        Ok(())
    }

    pub fn stop(&self) {
        println!("Reactor stopping...");
        self.running.store(false, Ordering::SeqCst);
    }

    pub fn get_epoll_fd(&self) -> RawFd {
        self.epoll_fd
    }
    
    pub fn share_running_state(&mut self, running: Arc<AtomicBool>) {
        self.running = running;
    }  
}

impl Clone for Reactor {
    fn clone(&self) -> Self {
        let epoll_fd = unsafe { libc::epoll_create1(0) };
        if epoll_fd == -1 {
            panic!("Failed to create epoll instance in clone");
        }

        Self {
            epoll_fd,
            running: Arc::clone(&self.running),
            handlers: HashMap::new(),
        }
    }
}

impl Drop for Reactor {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.epoll_fd);
        }
    }
}