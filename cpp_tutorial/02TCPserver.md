TCP是传输层的的协议，我们先实现一个简单的TCP框架，之后可以继承这个TCP框架（应用层是基于传输层实现的），去实现应用层的协议（在TCP基础上,加上特定的处理方式），比如：HTTP 服务器、聊天服务器、游戏服务器、数据库服务器、文件传输服务器等等。

首先介绍下网络编程里常常用到的操作系统API：

```bash
1. 套接字管理

socket(int domain, int type, int protocol):
创建一个新的套接字。

- domain: 套接字协议族（如 AF_INET 表示 IPv4）。
- type: 套接字类型（如 SOCK_STREAM 表示 TCP，SOCK_DGRAM 表示 UDP）。
- protocol: 通常为 0（默认协议），可以指定具体协议。

bind(int sockfd, const struct sockaddr *addr, socklen_t addrlen):
将套接字与特定的本地地址和端口绑定。

- sockfd: 套接字文件描述符。
- addr: 本地地址结构。
- addrlen: 地址结构的长度。

listen(int sockfd, int backlog):
使套接字进入被动监听状态，等待连接。

- sockfd: 套接字文件描述符。
- backlog: 连接队列的最大长度。

accept(int sockfd, struct sockaddr *addr, socklen_t *addrlen):
接受一个传入的连接请求。

- sockfd: 监听套接字的文件描述符。
- addr: 客户端地址结构。
- addrlen: 地址结构的长度。

connect(int sockfd, const struct sockaddr *addr, socklen_t addrlen):
主动发起连接到指定的服务器。

- sockfd: 套接字文件描述符。
- addr: 服务器地址结构。
- addrlen: 地址结构的长度。

2. 数据传输

send(int sockfd, const void *buf, size_t len, int flags):
发送数据到已连接的套接字。

- sockfd: 套接字文件描述符。
- buf: 发送缓冲区指针。
- len: 发送数据的长度。
- flags: 可选的发送标志。

recv(int sockfd, void *buf, size_t len, int flags):
从已连接的套接字接收数据。

- sockfd: 套接字文件描述符。
- buf: 接收缓冲区指针。
- len: 接收数据的长度。
- flags: 可选的接收标志。

sendto(int sockfd, const void *buf, size_t len, int flags, const struct sockaddr *dest_addr, socklen_t addrlen):
发送数据到指定的目标地址，通常用于UDP。

- sockfd: 套接字文件描述符。
- buf: 发送缓冲区指针。
- len: 发送数据的长度。
- flags: 可选的发送标志。
- dest_addr: 目标地址结构。
- addrlen: 地址结构的长度。

recvfrom(int sockfd, void *buf, size_t len, int flags, struct sockaddr *src_addr, socklen_t *addrlen):
接收来自指定源地址的数据，通常用于UDP。

- sockfd: 套接字文件描述符。
- buf: 接收缓冲区指针。
- len: 接收数据的长度。
- flags: 可选的接收标志。
- src_addr: 源地址结构。
- addrlen: 地址结构的长度。

3. 套接字选项和配置

setsockopt(int sockfd, int level, int optname, const void *optval, socklen_t optlen):
设置套接字选项。

- sockfd: 套接字文件描述符。
- level: 选项所在的协议层（如 SOL_SOCKET）。
- optname: 选项名称（如 SO_REUSEADDR）。
- optval: 选项值指针。
- optlen: 选项值的长度。

getsockopt(int sockfd, int level, int optname, void *optval, socklen_t *optlen):
获取套接字选项。

- sockfd: 套接字文件描述符。
- level: 选项所在的协议层。
- optname: 选项名称。
- optval: 存储选项值的缓冲区。
- optlen: 缓冲区的长度。

fcntl(int fd, int cmd, ...):
操作文件描述符的属性，常用于设置套接字为非阻塞模式。

- fd: 文件描述符。
- cmd: 操作命令（如 F_GETFL 获取文件状态标志，F_SETFL 设置文件状态标志）。

4. 关闭连接

close(int sockfd):
关闭套接字，释放资源。
- sockfd: 套接字文件描述符。

5. 地址和主机转换

getaddrinfo(const char *node, const char *service, const struct addrinfo *hints, struct addrinfo **res):
获取地址信息。

- node: 主机名或IP地址。
- service: 服务名或端口号。
- hints: 指定查询的条件。
- res: 返回的结果。

freeaddrinfo(struct addrinfo *res):
释放由 getaddrinfo 分配的地址信息。

inet_ntop(int af, const void *src, char *dst, socklen_t size):
将网络字节序的二进制IP地址转换为点分十进制的字符串格式。

inet_pton(int af, const char *src, void *dst):
将点分十进制的IP地址字符串转换为网络字节序的二进制格式。
```

我们的TCP server需要确定它的ip地址和端口，所以我们声明了`std::string ip_;`和`int port_;`成员变量。`Reactor& reactor_;`这个成员就是用于管理事件的，会在server类外面定义，所以我们声明成引用接收它。`int server_fd_;`是服务器的主套接字文件描述符。`bool running_;`管理server启动和停止的时候会用上。`std::function<void(int, const char*, size_t)> receive_handler_;`就是服务端接收到数据后的处理方式，我们为其声明一个`void set_receive_handler(std::function<void(int, const char*, size_t)> handler);`来修改它，参数分别是：客户端文件描述符、接收到的数据、数据长度。。

`static constexpr size_t BUFFER_SIZE = 1024`定义了接收数据时使用的缓冲区大小,它是一个静态常量，所有 TcpServer 实例共享这个值，我们会用在 handle_read() 方法中，决定一次读取的最大字节数。

`start()`和`stop()`分别用于server的启动和管理。剩下几个函数，我们会在实际实现的时候看它们的作用：

当TCPserver初始化的时候，会创建套接字: `server_fd_ = socket(AF_INET, SOCK_STREAM, 0);`，返回的文件描述符存到server_fd_。如果server_fd_等于-1,则证明创建套接字失败。`setsockopt(server_fd_, SOL_SOCKET, SO_REUSEADDR, &opt, sizeof(opt))`是在设置套接字选项，当服务器关闭后，操作系统会将该地址和端口标记为一段时间内不可用，这个选项可以覆盖这个行为，允许立即重用地址和端口。当 opt 为非零值（如1）时，表示开启该选项，当 opt 为0时，表示关闭该选项。

为我们的server创建地址结构，这里就是创建一个 sockaddr_in 结构体来存储服务器的地址信息。：

```bash
sockaddr_in address;
address.sin_family = AF_INET; // 设置地址族为 IPv4（AF_INET）。
address.sin_addr.s_addr = INADDR_ANY;  // 这行代码是用来设置服务器监听的 IP 地址，INADDR_ANY表示服务器将在所有可用的网络接口上监听。
address.sin_port = htons(port_); // 将端口号转换为网络字节序。
```

把套接字和指定的 IP 地址和端口绑定起来：`bind(server_fd_, (struct sockaddr*)&address, sizeof(address))`如果绑定失败，就关闭套接字，并抛出异常：

```bash
if (bind(server_fd_, (struct sockaddr*)&address, sizeof(address)) == -1) {
    close(server_fd_);
    throw std::runtime_error("Failed to bind to port");
}
```

然后使套接字进入监听状态，准备接收客户端连接,`SOMAXCONN` 是系统定义的最大挂起连接队列长度,如果监听失败，关闭套接字并抛出异常：

```bash
if (listen(server_fd_, SOMAXCONN) == -1) {
    close(server_fd_);
    throw std::runtime_error("Failed to listen on socket");
}
```

最后把我们的服务端设置为非阻塞模式，非阻塞模式允许套接字操作（如 accept）立即返回，而不是阻塞等待。添加 `O_NONBLOCK` 标志，就是将套接字设置为非阻塞模式。获取当前套接字的标志赋值给`flags`：

```bash
int flags = fcntl(server_fd_, F_GETFL, 0);
fcntl(server_fd_, F_SETFL, flags | O_NONBLOCK);
```

对应的析构函数会停止服务器（如果正在运行），然后关闭套接字：

```bash
TcpServer::~TcpServer() {
    if (running_) {
        stop();
    }
    close(server_fd_);
}
```

来看server的start()函数，首先肯定要把`running_`置为true，然后把server_fd_ 添加到 Reactor 中，并设置当有 EPOLLIN 事件（表示有新连接到来）时，调用 accept_connection() 函数。这样，只有在确实有新连接时，才会去执行接受连接的操作：

```bash
void TcpServer::start() {
    running_ = true;
    reactor_.add_handler(server_fd_, EPOLLIN, [this](uint32_t events) {
        accept_connection();
    });
}
```

`accept_connection()`是实际建立连接的流程，我们首先创建一个 `sockaddr_in 结构体`来存储客户端的地址信息，然后调用 `accept()` 函数来接受一个新的客户端连接。这个函数返回一个新的文件描述符 `client_fd`，用于与客户端通信。如果 accept() 失败（返回-1），检查错误类型。EAGAIN 和 EWOULDBLOCK 表示暂时没有新连接，这在非阻塞模式下是正常的。其他错误则打印错误信息。再设置客户端socket为非阻塞模式，把这个事件再传给Reactor去管理。如果是读事件（EPOLLIN），调用 `handle_read()`。
如果是关闭事件（EPOLLRDHUP 或 EPOLLHUP），调用 `handle_close()`：

```bash
void TcpServer::accept_connection() {
    sockaddr_in client_addr;
    socklen_t client_len = sizeof(client_addr);
    int client_fd = accept(server_fd_, (struct sockaddr*)&client_addr, &client_len);

    if (client_fd == -1) {
        if (errno != EAGAIN && errno != EWOULDBLOCK) {
            std::cerr << "Failed to accept connection" << std::endl;
        }
        return;
    }

    // 为客户端的socket设置非阻塞
    int flags = fcntl(client_fd, F_GETFL, 0);
    fcntl(client_fd, F_SETFL, flags | O_NONBLOCK);

    reactor_.add_handler(client_fd, EPOLLIN | EPOLLRDHUP, [this, client_fd](uint32_t events) {
        if (events & EPOLLIN) {
            handle_read(client_fd);
        }
        if (events & (EPOLLRDHUP | EPOLLHUP)) {
            handle_close(client_fd);
        }
    });
}
```

`handle_read(int client_fd)`负责从客户端套接字读取数据，它会创建一个缓冲区，然后用`read`函数从套接字里读取数据到buffer。如果读取成功，就对数据进行相应处理，否则关闭客户端连接:

```bash

void TcpServer::handle_read(int client_fd) {
    char buffer[BUFFER_SIZE];
    ssize_t bytes_read = read(client_fd, buffer, BUFFER_SIZE);

    if (bytes_read > 0) {
        if (receive_handler_) {
            receive_handler_(client_fd, buffer, bytes_read);
        }
    } else if (bytes_read == 0 || (bytes_read == -1 && errno != EAGAIN)) {
        handle_close(client_fd);
    }
}
```

`handle_close(int client_fd)` 负责清理工作，确保连接被正确关闭，并从事件循环中移除:

```bash
void TcpServer::handle_close(int client_fd) {
    reactor_.remove_handler(client_fd);
    close(client_fd);
}
```

到这里，我们可以测试一下server的吞吐量，编译运行下面这段代码：

```bash
#include "network/tcp_server.h"
#include "core/reactor.h"
#include <benchmark/benchmark.h>
#include <thread>
#include <vector>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <unistd.h>
#include <cstring>

static void BM_TcpServerThroughput(benchmark::State& state) {
    Reactor reactor;
    TcpServer server(reactor, "127.0.0.1", 8080);

    std::atomic<size_t> total_bytes_received(0);

    server.set_receive_handler([&](int client_fd, const char* data, size_t len) {
        total_bytes_received += len;
    });

    server.start();

    std::thread server_thread([&reactor]() {
        reactor.run();
    });

    // 给服务器一些时间来启动
    std::this_thread::sleep_for(std::chrono::milliseconds(100));

    std::vector<int> client_fds;
    for (int i = 0; i < state.range(0); ++i) {
        int client_fd = socket(AF_INET, SOCK_STREAM, 0);
        sockaddr_in addr;
        addr.sin_family = AF_INET;
        addr.sin_port = htons(8080);
        inet_pton(AF_INET, "127.0.0.1", &addr.sin_addr);
        connect(client_fd, (struct sockaddr*)&addr, sizeof(addr));
        client_fds.push_back(client_fd);
    }

    for (auto _ : state) {
        const char* message = "Hello, Server!";
        for (int client_fd : client_fds) {
            send(client_fd, message, strlen(message), 0);
        }
    }

    for (int client_fd : client_fds) {
        close(client_fd);
    }

    reactor.stop();
    server_thread.join();

    state.SetBytesProcessed(total_bytes_received.load());
}

BENCHMARK(BM_TcpServerThroughput)->Arg(1)->Arg(10)->Arg(100);

BENCHMARK_MAIN();
```