#include "network/tcp_server.h"
#include <sys/socket.h>
#include <sys/epoll.h>
#include <netinet/in.h>
#include <unistd.h>
#include <fcntl.h>
#include <stdexcept>
#include <cstring>
#include <iostream>

TcpServer::TcpServer(Reactor& reactor, const std::string& ip, int port)
    : reactor_(reactor), ip_(ip), port_(port), running_(false) {
    server_fd_ = socket(AF_INET, SOCK_STREAM, 0);
    if (server_fd_ == -1) {
        throw std::runtime_error("Failed to create socket");
    }

    int opt = 1;
    if (setsockopt(server_fd_, SOL_SOCKET, SO_REUSEADDR, &opt, sizeof(opt)) == -1) {
        close(server_fd_);
        throw std::runtime_error("Failed to set socket options");
    }

    sockaddr_in address;
    address.sin_family = AF_INET; // 设置地址族为 IPv4（AF_INET）。
    address.sin_addr.s_addr = INADDR_ANY;  // 这行代码是用来设置服务器监听的 IP 地址，INADDR_ANY表示服务器将在所有可用的网络接口上监听。
    address.sin_port = htons(port_); // 将端口号转换为网络字节序。

    if (bind(server_fd_, (struct sockaddr*)&address, sizeof(address)) == -1) {
        close(server_fd_);
        throw std::runtime_error("Failed to bind to port");
    }

    if (listen(server_fd_, SOMAXCONN) == -1) {
        close(server_fd_);
        throw std::runtime_error("Failed to listen on socket");
    }

    // 把我们的服务端设置为非阻塞模式
    int flags = fcntl(server_fd_, F_GETFL, 0);
    fcntl(server_fd_, F_SETFL, flags | O_NONBLOCK);
}

TcpServer::~TcpServer() {
    if (running_) {
        stop();
    }
    close(server_fd_);
}

void TcpServer::start() {
    running_ = true;
    reactor_.add_handler(server_fd_, EPOLLIN, [this](uint32_t events) {
        accept_connection();
    });
}

void TcpServer::stop() {
    running_ = false;
    reactor_.remove_handler(server_fd_);
}

void TcpServer::set_receive_handler(std::function<void(int, const char*, size_t)> handler) {
    receive_handler_ = std::move(handler);
}

void TcpServer::send(int client_fd, const char* data, size_t len) {
    ::send(client_fd, data, len, 0);
}

void TcpServer::handle_close(int client_fd) {
    reactor_.remove_handler(client_fd);
    close(client_fd);
}

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