#pragma once
#include "../core/reactor.h"
#include <string>
#include <functional>

class TcpServer {
public:
    TcpServer(Reactor& reactor, const std::string& ip, int port);
    ~TcpServer();

    void start();
    void stop();
    
    // 设置数据接收处理函数
    void set_receive_handler(std::function<void(int, const char*, size_t)> handler);

    // 向客户端发送消息
    void send(int client_fd, const char* data, size_t len);

private:
    void accept_connection();
    void handle_read(int client_fd);
    void handle_close(int client_fd);

    Reactor& reactor_;
    std::string ip_;
    int port_;
    int server_fd_;
    bool running_;

    std::function<void(int, const char*, size_t)> receive_handler_;

    static constexpr size_t BUFFER_SIZE = 1024;
};