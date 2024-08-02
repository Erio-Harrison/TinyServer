#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};
    use std::thread;
    use std::time::Duration;

    // 创建一个简单的连接工厂，用于生成模拟的连接对象
    fn connection_factory() -> Connection {
        static mut COUNTER: i32 = 0;
        unsafe {
            COUNTER += 1;
            Connection::new(COUNTER)
        }
    }

    #[test]
    fn test_get_and_release_connection() {
        let pool = ConnectionPool::new(2, connection_factory);

        // 获取连接
        let conn1 = pool.get_connection();
        let conn2 = pool.get_connection();

        // 确保连接是不同的
        assert_ne!(conn1.fd(), conn2.fd());

        // 释放连接
        pool.release_connection(conn1);
        pool.release_connection(conn2);

        // 获取连接并确保是先前的连接
        let conn3 = pool.get_connection();
        assert_eq!(conn3.fd(), 1);
        let conn4 = pool.get_connection();
        assert_eq!(conn4.fd(), 2);

        // 释放连接
        pool.release_connection(conn3);
        pool.release_connection(conn4);
    }

    #[test]
    fn test_max_connections() {
        let pool = ConnectionPool::new(2, connection_factory);

        // 获取两个连接
        let conn1 = pool.get_connection();
        let conn2 = pool.get_connection();

        // 尝试获取第三个连接时，需要等待
        let pool_clone = pool.clone();
        let barrier = Arc::new(Barrier::new(2));
        let barrier_clone = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            barrier_clone.wait(); // 同步线程启动
            let conn3 = pool_clone.get_connection();
            assert_eq!(conn3.fd(), 3); // 检查连接是否正确
        });

        // 确保线程正在等待
        thread::sleep(Duration::from_secs(1));

        // 释放一个连接
        pool.release_connection(conn1);

        // 等待线程完成
        barrier.wait();
        handle.join().expect("Thread panicked");
    }

    #[test]
    fn test_concurrent_access() {
        let pool = Arc::new(ConnectionPool::new(5, connection_factory));
        let barrier = Arc::new(Barrier::new(6)); // 5线程 + 主线程
        let mut handles = Vec::new();

        for _ in 0..5 {
            let pool_clone = Arc::clone(&pool);
            let barrier_clone = Arc::clone(&barrier);

            let handle = thread::spawn(move || {
                barrier_clone.wait(); // 等待所有线程准备好
                let conn = pool_clone.get_connection();
                assert!(conn.fd() > 0); // 确保获取到了有效连接
                thread::sleep(Duration::from_millis(500)); // 模拟连接使用
                pool_clone.release_connection(conn); // 释放连接
            });

            handles.push(handle);
        }

        // 等待所有线程启动
        barrier.wait();

        // 等待所有线程完成
        for handle in handles {
            handle.join().expect("Thread panicked");
        }
    }
}
