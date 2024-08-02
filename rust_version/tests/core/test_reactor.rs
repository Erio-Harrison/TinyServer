#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Write};
    use std::os::unix::io::{AsRawFd, FromRawFd};
    use std::sync::mpsc::{self, Sender};
    use std::sync::{Arc, Barrier};
    use std::thread;
    use std::time::Duration;

    // 创建一个读写管道，并返回读取端和写入端的文件描述符
    fn create_pipe() -> io::Result<(RawFd, RawFd)> {
        let mut fds = [0; 2];
        if unsafe { libc::pipe(fds.as_mut_ptr()) } == -1 {
            return Err(io::Error::last_os_error());
        }
        Ok((fds[0], fds[1]))
    }

    #[test]
    fn test_add_handler() {
        let (read_fd, write_fd) = create_pipe().expect("Failed to create pipe");

        let barrier = Arc::new(Barrier::new(2));
        let barrier_clone = Arc::clone(&barrier);

        let mut reactor = Reactor::new(1).expect("Failed to create reactor");

        // 添加事件处理器
        reactor
            .add_handler(read_fd, move || {
                let mut buffer = [0; 1];
                let mut file = unsafe { std::fs::File::from_raw_fd(read_fd) };
                file.read_exact(&mut buffer)?;
                assert_eq!(buffer[0], b'x');
                barrier_clone.wait(); // 让主线程知道已经执行了事件处理器
                Ok(())
            })
            .expect("Failed to add handler");

        // 启动事件循环
        thread::spawn(move || reactor.run().expect("Failed to run reactor"));

        // 写入数据以触发事件
        let mut file = unsafe { std::fs::File::from_raw_fd(write_fd) };
        file.write_all(b"x").expect("Failed to write data");

        // 等待事件处理器执行
        barrier.wait();

        // 停止事件循环
        reactor.stop();
    }

    #[test]
    fn test_remove_handler() {
        let (read_fd, write_fd) = create_pipe().expect("Failed to create pipe");

        let mut reactor = Reactor::new(1).expect("Failed to create reactor");

        // 添加事件处理器
        reactor
            .add_handler(read_fd, || {
                Ok(())
            })
            .expect("Failed to add handler");

        // 移除事件处理器
        reactor
            .remove_handler(read_fd)
            .expect("Failed to remove handler");

        // 启动事件循环
        let barrier = Arc::new(Barrier::new(2));
        let barrier_clone = Arc::clone(&barrier);

        thread::spawn(move || {
            // 写入数据
            let mut file = unsafe { std::fs::File::from_raw_fd(write_fd) };
            file.write_all(b"x").expect("Failed to write data");
            barrier_clone.wait(); // 等待主线程通知停止
        });

        // 确保事件处理器没有被调用
        thread::sleep(Duration::from_secs(1));

        // 停止事件循环
        reactor.stop();

        barrier.wait(); // 通知写线程停止
    }

    #[test]
    fn test_multiple_handlers() {
        let (read_fd1, write_fd1) = create_pipe().expect("Failed to create pipe");
        let (read_fd2, write_fd2) = create_pipe().expect("Failed to create pipe");

        let barrier = Arc::new(Barrier::new(3));
        let barrier_clone1 = Arc::clone(&barrier);
        let barrier_clone2 = Arc::clone(&barrier);

        let mut reactor = Reactor::new(2).expect("Failed to create reactor");

        // 添加第一个事件处理器
        reactor
            .add_handler(read_fd1, move || {
                let mut buffer = [0; 1];
                let mut file = unsafe { std::fs::File::from_raw_fd(read_fd1) };
                file.read_exact(&mut buffer)?;
                assert_eq!(buffer[0], b'x');
                barrier_clone1.wait(); // 让主线程知道已经执行了事件处理器
                Ok(())
            })
            .expect("Failed to add handler");

        // 添加第二个事件处理器
        reactor
            .add_handler(read_fd2, move || {
                let mut buffer = [0; 1];
                let mut file = unsafe { std::fs::File::from_raw_fd(read_fd2) };
                file.read_exact(&mut buffer)?;
                assert_eq!(buffer[0], b'y');
                barrier_clone2.wait(); // 让主线程知道已经执行了事件处理器
                Ok(())
            })
            .expect("Failed to add handler");

        // 启动事件循环
        thread::spawn(move || reactor.run().expect("Failed to run reactor"));

        // 写入数据以触发事件
        let mut file1 = unsafe { std::fs::File::from_raw_fd(write_fd1) };
        file1.write_all(b"x").expect("Failed to write data");

        let mut file2 = unsafe { std::fs::File::from_raw_fd(write_fd2) };
        file2.write_all(b"y").expect("Failed to write data");

        // 等待事件处理器执行
        barrier.wait();

        // 停止事件循环
        reactor.stop();
    }
}
