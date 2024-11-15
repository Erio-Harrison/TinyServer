use rust_version::core::reactor::Reactor;
use std::io::{self, Read, Write};
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

// 创建管道的辅助函数
fn create_pipe() -> io::Result<(RawFd, RawFd)> {
    let mut fds = [0; 2];
    if unsafe { libc::pipe(fds.as_mut_ptr()) } == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok((fds[0], fds[1]))
}

#[test]
fn test_reactor_creation() {
    let reactor = Reactor::new().expect("Failed to create reactor");
    assert!(reactor.get_epoll_fd() > 0);
}

#[test]
fn test_basic_event_handling() {
    let (read_fd, write_fd) = create_pipe().expect("Failed to create pipe");
    let barrier = Arc::new(Barrier::new(2));
    let barrier_clone = Arc::clone(&barrier);
    let received = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let received_clone = Arc::clone(&received);

    let mut reactor = Reactor::new().expect("Failed to create reactor");

    reactor.add_handler(
        read_fd,
        libc::EPOLLIN as u32,
        move |events| {
            if events & (libc::EPOLLIN as u32) != 0 {
                let mut buf = [0u8; 1];
                let mut file = unsafe { std::fs::File::from_raw_fd(read_fd) };
                if file.read_exact(&mut buf).is_ok() {
                    received_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                }
                barrier_clone.wait();
                let _ = file.into_raw_fd(); // 防止文件被关闭
            }
        },
    ).expect("Failed to add handler");

    // 启动 reactor 线程
    let reactor_thread = thread::spawn(move || {
        reactor.run().expect("Failed to run reactor");
    });

    // 发送数据
    {
        let mut file = unsafe { std::fs::File::from_raw_fd(write_fd) };
        file.write_all(b"x").expect("Failed to write");
        let _ = file.into_raw_fd(); // 防止文件被关闭
    }

    // 等待处理完成
    barrier.wait();

    assert!(received.load(std::sync::atomic::Ordering::SeqCst));

    // 清理
    unsafe {
        libc::close(read_fd);
        libc::close(write_fd);
    }
}

#[test]
fn test_multiple_events() {
    let (read_fd1, write_fd1) = create_pipe().expect("Failed to create first pipe");
    let (read_fd2, write_fd2) = create_pipe().expect("Failed to create second pipe");
    
    let barrier = Arc::new(Barrier::new(3));
    let received1 = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let received2 = Arc::new(std::sync::atomic::AtomicBool::new(false));
    
    let mut reactor = Reactor::new().expect("Failed to create reactor");

    {
        let barrier_clone = Arc::clone(&barrier);
        let received_clone = Arc::clone(&received1);
        reactor.add_handler(
            read_fd1,
            libc::EPOLLIN as u32,
            move |events| {
                if events & (libc::EPOLLIN as u32) != 0 {
                    let mut buf = [0u8; 1];
                    let mut file = unsafe { std::fs::File::from_raw_fd(read_fd1) };
                    if file.read_exact(&mut buf).is_ok() {
                        received_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                    }
                    barrier_clone.wait();
                    let _ = file.into_raw_fd();
                }
            },
        ).expect("Failed to add first handler");
    }

    {
        let barrier_clone = Arc::clone(&barrier);
        let received_clone = Arc::clone(&received2);
        reactor.add_handler(
            read_fd2,
            libc::EPOLLIN as u32,
            move |events| {
                if events & (libc::EPOLLIN as u32) != 0 {
                    let mut buf = [0u8; 1];
                    let mut file = unsafe { std::fs::File::from_raw_fd(read_fd2) };
                    if file.read_exact(&mut buf).is_ok() {
                        received_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                    }
                    barrier_clone.wait();
                    let _ = file.into_raw_fd();
                }
            },
        ).expect("Failed to add second handler");
    }

    // 启动 reactor 线程
    let reactor_thread = thread::spawn(move || {
        reactor.run().expect("Failed to run reactor");
    });

    // 发送数据到两个管道
    {
        let mut file1 = unsafe { std::fs::File::from_raw_fd(write_fd1) };
        let mut file2 = unsafe { std::fs::File::from_raw_fd(write_fd2) };
        file1.write_all(b"x").expect("Failed to write to first pipe");
        file2.write_all(b"y").expect("Failed to write to second pipe");
        let _ = file1.into_raw_fd();
        let _ = file2.into_raw_fd();
    }

    // 等待两个事件都处理完成
    barrier.wait();
    barrier.wait();

    assert!(received1.load(std::sync::atomic::Ordering::SeqCst));
    assert!(received2.load(std::sync::atomic::Ordering::SeqCst));

    // 清理
    unsafe {
        libc::close(read_fd1);
        libc::close(write_fd1);
        libc::close(read_fd2);
        libc::close(write_fd2);
    }
}

#[test]
fn test_handler_removal() {
    let (read_fd, write_fd) = create_pipe().expect("Failed to create pipe");
    let mut reactor = Reactor::new().expect("Failed to create reactor");

    // 添加处理器
    reactor.add_handler(
        read_fd,
        libc::EPOLLIN as u32,
        move |_| {
            panic!("Handler should not be called after removal");
        },
    ).expect("Failed to add handler");

    // 移除处理器
    reactor.remove_handler(read_fd).expect("Failed to remove handler");

    let reactor_thread = thread::spawn(move || {
        reactor.run().expect("Failed to run reactor");
    });

    // 发送数据
    {
        let mut file = unsafe { std::fs::File::from_raw_fd(write_fd) };
        file.write_all(b"x").expect("Failed to write");
        let _ = file.into_raw_fd();
    }

    // 等待一段时间确保没有触发处理器
    thread::sleep(Duration::from_millis(100));

    // 清理
    unsafe {
        libc::close(read_fd);
        libc::close(write_fd);
    }
}