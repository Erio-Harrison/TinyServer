// src/reactor.rs
use std::collections::HashMap;
use std::io;
use std::os::unix::io::RawFd;
use std::ptr;

const MAX_EVENTS: usize = 1024;

pub struct Reactor {
    epoll_fd: RawFd,
    handlers: HashMap<RawFd, Box<dyn FnMut() -> io::Result<()>>>,
    events: Vec<libc::epoll_event>,
    running: bool,
}

impl Reactor {
    pub fn new() -> io::Result<Self> {
        let epoll_fd = unsafe { libc::epoll_create1(0) };
        if epoll_fd < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Reactor {
            epoll_fd,
            handlers: HashMap::new(),
            events: vec![libc::epoll_event { events: 0, u64: 0 }; MAX_EVENTS],
            running: false,
        })
    }

    pub fn add_handler<F>(&mut self, fd: RawFd, handler: F) -> io::Result<()>
    where
        F: FnMut() -> io::Result<()> + 'static,
    {
        let mut event = libc::epoll_event {
            events: libc::EPOLLIN as u32,
            u64: fd as u64,
        };
        let res = unsafe {
            libc::epoll_ctl(
                self.epoll_fd,
                libc::EPOLL_CTL_ADD,
                fd,
                &mut event as *mut libc::epoll_event,
            )
        };
        if res < 0 {
            return Err(io::Error::last_os_error());
        }
        self.handlers.insert(fd, Box::new(handler));
        Ok(())
    }

    pub fn remove_handler(&mut self, fd: RawFd) -> io::Result<()> {
        let res = unsafe { libc::epoll_ctl(self.epoll_fd, libc::EPOLL_CTL_DEL, fd, ptr::null_mut()) };
        if res < 0 {
            return Err(io::Error::last_os_error());
        }
        self.handlers.remove(&fd);
        Ok(())
    }

    pub fn run(&mut self) -> io::Result<()> {
        self.running = true;
        while self.running {
            let nfds = unsafe {
                libc::epoll_wait(
                    self.epoll_fd,
                    self.events.as_mut_ptr(),
                    MAX_EVENTS as i32,
                    -1,
                )
            };
            if nfds < 0 {
                let err = io::Error::last_os_error();
                if err.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(err);
            }
            for i in 0..nfds {
                let event = unsafe { self.events.get_unchecked(i as usize) };
                let fd = event.u64 as RawFd;
                if let Some(handler) = self.handlers.get_mut(&fd) {
                    handler()?;
                }
            }
        }
        Ok(())
    }

    pub fn stop(&mut self) {
        self.running = false;
    }
}

impl Drop for Reactor {
    fn drop(&mut self) {
        unsafe { libc::close(self.epoll_fd) };
    }
}