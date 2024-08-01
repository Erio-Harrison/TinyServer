#pragma once

#include <functional>
#include <unordered_map>
#include <vector>
#include <memory>

class Reactor {
public:
    Reactor();
    ~Reactor();

    // 添加一个文件描述符和对应的处理函数
    void add_handler(int fd, uint32_t events, std::function<void()> handler);

    // 移除一个文件描述符的处理
    void remove_handler(int fd);

    // 运行事件循环
    void run();

    // 停止事件循环
    void stop();

    int get_epoll_fd();

private:
    int epoll_fd_;
    bool running_;
    std::unordered_map<int, std::function<void()>> handlers_;
    std::vector<struct epoll_event> events_;

    static constexpr int MAX_EVENTS = 10;
};