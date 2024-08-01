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

我们的TCP server需要确定它的ip地址和端口，所以我们声明了`std::string ip_;`和`int port_;`成员变量。`Reactor& reactor_;`这个成员就是用于管理事件的。`int server_fd_;`是服务器的主套接字文件描述符。`bool running_;`用来管理server的启动和停止。