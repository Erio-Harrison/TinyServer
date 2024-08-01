use criterion::{criterion_group, criterion_main, Criterion};
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use std::sync::atomic::{AtomicUsize, Ordering};

// 假设我们已经有了 TcpServer 的实现
use crate::network::TcpServer;

fn bench_tcp_server_throughput(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap();

    let mut group = c.benchmark_group("TcpServer Throughput");
    for &num_clients in &[1, 10, 100] {
        group.bench_function(format!("{} clients", num_clients), |b| {
            b.iter_custom(|iters| {
                runtime.block_on(async {
                    let server = Arc::new(TcpServer::new("127.0.0.1".to_string(), 8080));
                    let total_bytes_received = Arc::new(AtomicUsize::new(0));

                    let server_clone = Arc::clone(&server);
                    let total_bytes_clone = Arc::clone(&total_bytes_received);

                    server.set_receive_handler(move |_client_id, data| {
                        total_bytes_clone.fetch_add(data.len(), Ordering::SeqCst);
                    });

                    let server_handle = tokio::spawn(async move {
                        server_clone.start().await.unwrap();
                    });

                    // 给服务器一些时间来启动
                    tokio::time::sleep(Duration::from_millis(100)).await;

                    let mut clients = Vec::new();
                    for _ in 0..num_clients {
                        let stream = TcpStream::connect("127.0.0.1:8080").await.unwrap();
                        clients.push(stream);
                    }

                    let start = std::time::Instant::now();

                    for _ in 0..iters {
                        let message = b"Hello, Server!";
                        for client in &mut clients {
                            client.write_all(message).await.unwrap();
                        }
                    }

                    let duration = start.elapsed();

                    for mut client in clients {
                        client.shutdown().await.unwrap();
                    }

                    server.stop().await;
                    server_handle.await.unwrap();

                    duration
                })
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_tcp_server_throughput);
criterion_main!(benches);