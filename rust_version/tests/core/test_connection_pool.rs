use std::sync::{Arc, atomic::{AtomicI32, Ordering}};
use std::thread;
use std::time::Duration;

#[test]
fn test_connection_pool_basic() {
    let counter = Arc::new(AtomicI32::new(0));
    let counter_clone = counter.clone();
    
    let pool = ConnectionPool::new(2, move || {
        let fd = counter_clone.fetch_add(1, Ordering::SeqCst);
        Connection::new(fd)
    });

    // 获取第一个连接
    let conn1 = pool.get_connection();
    assert_eq!(conn1.fd(), 0);

    // 获取第二个连接
    let conn2 = pool.get_connection();
    assert_eq!(conn2.fd(), 1);

    // 释放第一个连接
    pool.release_connection(conn1);

    // 再次获取连接应该得到已释放的连接
    let conn3 = pool.get_connection();
    assert_eq!(conn3.fd(), 0);
}

#[test]
fn test_connection_pool_max_connections() {
    let pool = Arc::new(ConnectionPool::new(1, || Connection::new(0)));
    let pool_clone = pool.clone();

    // 获取唯一的连接
    let conn = pool.get_connection();
    assert_eq!(conn.fd(), 0);

    // 在另一个线程中尝试获取连接
    let handle = thread::spawn(move || {
        let start = std::time::Instant::now();
        let conn2 = pool_clone.get_connection();
        let duration = start.elapsed();
        
        // 验证等待时间
        assert!(duration >= Duration::from_millis(100));
        assert_eq!(conn2.fd(), 0);
    });

    // 等待一段时间后释放连接
    thread::sleep(Duration::from_millis(100));
    pool.release_connection(conn);

    handle.join().unwrap();
}

#[test]
fn test_connection_pool_concurrent_access() {
    let pool = Arc::new(ConnectionPool::new(5, || {
        Connection::new(0)
    }));
    let mut handles = vec![];

    // 创建10个线程同时访问连接池
    for _ in 0..10 {
        let pool_clone = pool.clone();
        let handle = thread::spawn(move || {
            // 获取连接
            let conn = pool_clone.get_connection();
            // 模拟使用连接
            thread::sleep(Duration::from_millis(50));
            // 释放连接
            pool_clone.release_connection(conn);
        });
        handles.push(handle);
    }

    // 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }

    // 验证所有连接都被正确释放
    let (lock, _) = &*pool.available_connections;
    let connections = lock.lock().unwrap();
    assert!(connections.len() <= 5);
}

#[test]
fn test_connection_pool_stress() {
    let counter = Arc::new(AtomicI32::new(0));
    let pool = Arc::new(ConnectionPool::new(3, move || {
        Connection::new(counter.fetch_add(1, Ordering::SeqCst))
    }));
    let mut handles = vec![];

    // 创建20个线程，每个线程获取和释放连接多次
    for _ in 0..20 {
        let pool_clone = pool.clone();
        let handle = thread::spawn(move || {
            for _ in 0..50 {
                let conn = pool_clone.get_connection();
                thread::sleep(Duration::from_millis(1));
                pool_clone.release_connection(conn);
            }
        });
        handles.push(handle);
    }

    // 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }

    // 验证最终状态
    let (lock, _) = &*pool.available_connections;
    let connections = lock.lock().unwrap();
    assert!(connections.len() <= 3);
}

#[test]
fn test_connection_pool_fairness() {
    let pool = Arc::new(ConnectionPool::new(1, || Connection::new(0)));
    let mut handles = vec![];
    let completion_order = Arc::new(Mutex::new(Vec::new()));

    // 创建5个线程，记录它们完成的顺序
    for i in 0..5 {
        let pool_clone = pool.clone();
        let completion_order_clone = completion_order.clone();
        let handle = thread::spawn(move || {
            let _conn = pool_clone.get_connection();
            thread::sleep(Duration::from_millis(10));
            completion_order_clone.lock().unwrap().push(i);
            pool_clone.release_connection(_conn);
        });
        handles.push(handle);
        // 确保线程按顺序启动
        thread::sleep(Duration::from_millis(1));
    }

    // 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }

    // 验证完成顺序基本符合FIFO
    let order = completion_order.lock().unwrap();
    for i in 1..order.len() {
        assert!(order[i] > order[i-1]);
    }
}