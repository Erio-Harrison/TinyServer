use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::net::TcpStream;
use std::io::Write;
use std::thread;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use rust_version::{
    core::Reactor,
    network::TcpServer,
    utils::{Logger, LogLevel},
};

fn benchmark_tcp_server_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("TcpServer Throughput");
    
    for &num_clients in &[1, 10, 100] {
        group.bench_function(format!("{} clients", num_clients), |b| {
            // 设置日志级别
            let logger = Logger::instance();
            logger.set_log_level(LogLevel::ERROR);

            let reactor = Arc::new(Reactor::new().unwrap());
            let server = Arc::new(TcpServer::new(reactor.clone(), "127.0.0.1", 8080).unwrap());

            let total_bytes_received = Arc::new(AtomicUsize::new(0));

            {
                let total_bytes_received = total_bytes_received.clone();
                server.set_receive_handler(move |_, data| {
                    total_bytes_received.fetch_add(data.len(), Ordering::Relaxed);
                    Ok(())
                });
            }

            server.start();

            let server_thread = thread::spawn(move || {
                reactor.run().unwrap();
            });

            // 给服务器一些时间来启动
            thread::sleep(Duration::from_millis(100));

            let mut clients = Vec::new();
            for _ in 0..num_clients {
                let client = TcpStream::connect("127.0.0.1:8080").unwrap();
                client.set_nonblocking(true).unwrap();
                clients.push(client);
            }

            b.iter(|| {
                let message = black_box("Hello, Server!");
                for client in &mut clients {
                    client.write_all(message.as_bytes()).unwrap();
                }
            });

            // 清理
            for client in clients {
                drop(client);
            }

            reactor.stop();
            server_thread.join().unwrap();

            println!("Total bytes received: {}", total_bytes_received.load(Ordering::Relaxed));
        });
    }

    group.finish();
}

criterion_group!(benches, benchmark_tcp_server_throughput);
criterion_main!(benches);