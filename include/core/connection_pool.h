#pragma once

#include <vector>
#include <queue>
#include <memory>
#include <mutex>
#include <condition_variable>
#include <functional>

class Connection {
public:
    Connection(int fd) : fd_(fd) {}
    int fd() const { return fd_; }
    // 可以添加其他连接相关的方法

private:
    int fd_;
};

class ConnectionPool {
public:
    ConnectionPool(size_t max_connections, std::function<std::unique_ptr<Connection>()> connection_factory);
    ~ConnectionPool();

    std::unique_ptr<Connection> get_connection();
    void release_connection(std::unique_ptr<Connection> conn);

private:
    std::vector<std::unique_ptr<Connection>> all_connections_;
    std::queue<Connection*> available_connections_;
    std::mutex mutex_;
    std::condition_variable cv_;
    size_t max_connections_;
    std::function<std::unique_ptr<Connection>()> connection_factory_;
};