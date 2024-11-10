use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::net::{TcpStream, SocketAddr};
use std::io::Write;
use rust_version::core::reactor::Reactor;
use rust_version::network::tcp_server::TcpServer;

fn tcp_server_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("TcpServer Throughput");
    
    for num_clients in [1, 10, 100] {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_clients),
            &num_clients,
            |b, &num_clients| {
                b.iter_custom(|iters| {

                    let total_bytes_received = Arc::new(AtomicUsize::new(0));
                    let bytes_received = Arc::clone(&total_bytes_received);
                    
                    let reactor = Reactor::new().expect("Failed to create reactor");
                    let mut server = TcpServer::new(reactor, "127.0.0.1", 8080)
                        .expect("Failed to create server");

                    server.set_receive_handler(move |_client_fd, _data, len| {
                        bytes_received.fetch_add(len, Ordering::SeqCst);
                    });

                    server.start().expect("Failed to start server");
                    
                    let mut reactor = server.get_reactor();
                    let server_thread = thread::spawn(move || {
                        reactor.run().expect("Reactor failed");
                    });

                    thread::sleep(Duration::from_millis(100));

                    let mut clients = Vec::with_capacity(num_clients);
                    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
                    
                    for _ in 0..num_clients {
                        let client = TcpStream::connect(addr).expect("Failed to connect");
                        clients.push(client);
                    }

                    let start = std::time::Instant::now();
                    let message = b"Hello, Server!";
                    
                    for _ in 0..iters {
                        for client in &mut clients {
                            client.write_all(message).expect("Failed to write");
                        }
                    }

                    let duration = start.elapsed();

                    drop(clients);
                    server.stop().expect("Failed to stop server");
                    
                    match server_thread.join() {
                        Ok(_) => println!("Server thread completed successfully"),
                        Err(e) => println!("Server thread error: {:?}", e),
                    }

                    println!(
                        "Processed {} bytes over {} iterations with {} clients",
                        total_bytes_received.load(Ordering::SeqCst),
                        iters,
                        num_clients
                    );

                    duration
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = tcp_server_throughput
);
criterion_main!(benches);