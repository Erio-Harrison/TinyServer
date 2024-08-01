use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;

const BUFFER_SIZE: usize = 1024;

struct TcpServer {
    ip: String,
    port: u16,
    running: bool,
    connection_handler: Option<Arc<dyn Fn(i32) + Send + Sync>>,
    receive_handler: Option<Arc<dyn Fn(i32, &[u8]) + Send + Sync>>,
    clients: Arc<Mutex<HashMap<i32, TcpStream>>>,
}

impl TcpServer {
    pub fn new(ip: String, port: u16) -> Self {
        TcpServer {
            ip,
            port,
            running: false,
            connection_handler: None,
            receive_handler: None,
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!("{}:{}", self.ip, self.port);
        let listener = TcpListener::bind(&addr).await?;
        println!("Server listening on {}", addr);

        self.running = true;

        while self.running {
            let (socket, _) = listener.accept().await?;
            let client_id = socket.peer_addr()?.port() as i32;

            {
                let mut clients = self.clients.lock().await;
                clients.insert(client_id, socket);
            }

            if let Some(ref handler) = self.connection_handler {
                handler(client_id);
            }

            let clients_clone = Arc::clone(&self.clients);
            let receive_handler = self.receive_handler.clone();

            tokio::spawn(async move {
                Self::handle_client(client_id, clients_clone, receive_handler).await;
            });
        }

        Ok(())
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    pub fn set_connection_handler<F>(&mut self, handler: F)
    where
        F: Fn(i32) + Send + Sync + 'static,
    {
        self.connection_handler = Some(Arc::new(handler));
    }

    pub fn set_receive_handler<F>(&mut self, handler: F)
    where
        F: Fn(i32, &[u8]) + Send + Sync + 'static,
    {
        self.receive_handler = Some(Arc::new(handler));
    }

    pub async fn send(&self, client_id: i32, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let mut clients = self.clients.lock().await;
        if let Some(stream) = clients.get_mut(&client_id) {
            stream.write_all(data).await?;
        }
        Ok(())
    }

    async fn handle_client(
        client_id: i32,
        clients: Arc<Mutex<HashMap<i32, TcpStream>>>,
        receive_handler: Option<Arc<dyn Fn(i32, &[u8]) + Send + Sync>>,
    ) {
        let mut buffer = [0u8; BUFFER_SIZE];

        loop {
            let mut stream = {
                let clients = clients.lock().await;
                if let Some(stream) = clients.get(&client_id) {
                    stream.try_clone().unwrap()
                } else {
                    break;
                }
            };

            match stream.read(&mut buffer).await {
                Ok(0) => {
                    // Connection closed
                    break;
                }
                Ok(n) => {
                    if let Some(ref handler) = receive_handler {
                        handler(client_id, &buffer[..n]);
                    }
                }
                Err(_) => break,
            }
        }

        // Connection closed, remove client
        let mut clients = clients.lock().await;
        clients.remove(&client_id);
    }
}