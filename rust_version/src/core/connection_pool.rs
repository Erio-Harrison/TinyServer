use std::sync::{Arc, Mutex, Condvar};
use std::collections::VecDeque;

pub struct Connection {
    fd: i32,
}

impl Connection {
    pub fn new(fd: i32) -> Self {
        Connection { fd }
    }

    pub fn fd(&self) -> i32 {
        self.fd
    }
}

pub struct ConnectionPool {
    max_connections: usize,
    available_connections: Arc<(Mutex<VecDeque<Connection>>, Condvar)>,
    connection_factory: Box<dyn Fn() -> Connection + Send + Sync>,
}

impl ConnectionPool {
    pub fn new<F>(max_connections: usize, connection_factory: F) -> Self
    where
        F: Fn() -> Connection + Send + Sync + 'static,
    {
        ConnectionPool {
            max_connections,
            available_connections: Arc::new((Mutex::new(VecDeque::new()), Condvar::new())),
            connection_factory: Box::new(connection_factory),
        }
    }

    pub fn get_connection(&self) -> Connection {
        let (lock, cvar) = &*self.available_connections;
        let mut connections = lock.lock().unwrap();

        loop {
            if let Some(conn) = connections.pop_front() {
                return conn;
            }

            if connections.len() < self.max_connections {
                let conn = (self.connection_factory)();
                return conn;
            }

            connections = cvar.wait(connections).unwrap();
        }
    }

    pub fn release_connection(&self, conn: Connection) {
        let (lock, cvar) = &*self.available_connections;
        let mut connections = lock.lock().unwrap();
        connections.push_back(conn);
        cvar.notify_one();
    }
}