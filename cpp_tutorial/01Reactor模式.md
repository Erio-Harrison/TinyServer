Reactor是一种常见的处理高并发场景的设计模式，它的核心原理是利用操作系统提供的I/O多路复用机制来分发事件。想象某个场景：我们可能会和很多不同客户端进行信息交互，这些信息没有涉及到什么复杂的计算，只是简单的订阅发布。如果我们频繁的创建新的`socket`连接,这个开销是很大的。所以我们可以只创建一个连接，然后根据事件的文件描述符将其分发给其他处理逻辑，来优化性能。

如果有很多客户端，并且事件的处理涉及到复杂计算，我们的优化方式是使用多线程Reactor模式，即将事件的分发和处理分离。主线程（或少量线程）专门负责事件的检测和分发，而事件处理交由线程池中的多个工作线程执行。这样可以充分利用多核CPU的优势，提高系统的并发处理能力。

常见的操作系统接口包括：

**Unix/Linux:**
select：监听多个文件描述符的状态变化。
poll：与select类似，但使用不同的数据结构。
epoll：Linux特有的高效事件通知机制，适用于处理大量并发连接。

**Windows:**
WSAAsyncSelect：用于异步网络I/O事件通知。
WSAEventSelect：类似于select，但使用事件对象。
IOCP（I/O Completion Ports）：一种高效的I/O完成端口机制，适合高并发环境。

我们项目里使用的是epoll，首先来介绍一下相关的系统调用：

1. 首先是创建：`epoll_create1(0)`,它会创建一个新的 epoll 实例，返回一个文件描述符，用于后续的 epoll 操作。这里的文件描述符是一个整数，项目当中是用`int epoll_fd_`来记住它。

2. 然后是`epoll_ctl(epoll_fd, op, fd, &event)`，它用于向 epoll 实例添加、修改或删除监视的文件描述符，op 可以是`EPOLL_CTL_ADD`, `EPOLL_CTL_MOD`, 或 `EPOLL_CTL_DEL`。`event` 结构体包含要监视的事件类型和用户数据，如果不需要，则设定为`nullptr`（比如说删除操作）。

3. `epoll_wait(epoll_fd, events, maxevents, timeout)`会等待 epoll 实例上的事件，返回就绪的文件描述符数量，然后将就绪的事件填充到 events 数组中。其中，`epoll_fd` 是 epoll 实例的文件描述符。`events` 是一个数组，用于存储就绪的事件。`maxevents` 是 events 数组的大小，即最多可以存储多少个事件。`timeout` 参数表示等待的超时时间。如果设置为 -1，说明该函数会一直阻塞，直到有事件就绪才返回。它会和`epoll_ctl(epoll_fd, op, fd, &event)`配合完成事件的分发，整个管理过程是由操作系统完成的。

4. `close(epoll_fd)`就是关闭 epoll 实例，释放资源

5. epoll 事件类型：

- EPOLLIN：表示对应的文件描述符可以读（包括对端SOCKET正常关闭）
- EPOLLOUT：表示对应的文件描述符可以写
- EPOLLRDHUP：表示套接字的一端已经关闭，或者半关闭
- EPOLLPRI：表示对应的文件描述符有紧急的数据可读
- EPOLLERR：表示对应的文件描述符发生错误
- EPOLLHUP：表示对应的文件描述符被挂断
- EPOLLET：将 EPOLL 设为边缘触发(Edge Triggered)模式（默认为水平触发）
- EPOLLONESHOT：只监听一次事件，当监听完这次事件之后，如果还需要继续监听这个 socket 的话，需要再次把这个 socket 加入到 EPOLL 队列里

6. 错误处理：检查系统调用的返回值，如果为 -1 表示出错，使用 `errno` 获取具体的错误信息

来看`Reactor`类的实现：

我们会用`int epoll_fd_`来表示文件描述符，用`bool running_`来代表这个类是否正在工作，`std::unordered_map<int, std::function<void()>> handlers_`：每一个文件描述符会对应一个处理函数（用于处理事件）,`std::vector<struct epoll_event> events_`。

在初始化的时候，会创建epoll实例，假如创建失败，则throw错误

```bash
Reactor::Reactor() : running_(false), events_(MAX_EVENTS) {
    epoll_fd_ = epoll_create1(0);
    if (epoll_fd_ == -1) {
        throw std::runtime_error("Failed to create epoll file descriptor");
    }
}
```
析构的时候会释放这个epoll实例：

```bash
Reactor::~Reactor() {
    close(epoll_fd_);
}
```

我们会用`void add_handler(int fd, uint32_t events, std::function<void()> handler);`添加一个文件描述符和对应的处理函数，整个过程很简单，三歩：1.创建 epoll_event 结构体，设置事件类型和文件描述符。2.使用 epoll_ctl 将文件描述符添加到 epoll 实例。3.将处理函数存储在 handlers_ 映射中：

```bash
void Reactor::add_handler(int fd, uint32_t events, std::function<void()> handler) {
    epoll_event ev;
    ev.events = events;
    ev.data.fd = fd;

    if (epoll_ctl(epoll_fd_, EPOLL_CTL_ADD, fd, &ev) == -1) {
        throw std::runtime_error("Failed to add file descriptor to epoll");
    }

    handlers_[fd] = std::move(handler);
}
```

相对应的删除操作：`remove_handler`有两歩：1.使用 epoll_ctl 从 epoll 实例中移除文件描述符。2. 从 handlers_ 映射中移除相应的处理函数：

```bash
void Reactor::remove_handler(int fd) {
    if (epoll_ctl(epoll_fd_, EPOLL_CTL_DEL, fd, nullptr) == -1) {
        throw std::runtime_error("Failed to remove file descriptor from epoll");
    }

    handlers_.erase(fd);
}
```

在`Reactor`模式下，会启动一个线程持续监听并且进行相应函数的处理操作，线程启动的时候首先会把running_变为true。然后进入while循环，`int nfds = epoll_wait(epoll_fd_, events_.data(), MAX_EVENTS, -1);`是关键，它会阻塞等待直到事件发生。`int nfds = epoll_wait(epoll_fd_, events_.data(), MAX_EVENTS, -1);`里nfds返回准备好的文件描述符数量，如果它等于-1则表示出错了。这里特别处理了 EINTR 错误（被信号中断），如果是这种情况，就继续循环。其他错误则抛出异常。


