#include "core/reactor.h"
#include <sys/epoll.h>
#include <unistd.h>
#include <stdexcept>
#include <iostream>

Reactor::Reactor() : running_(false), events_(MAX_EVENTS) {
    epoll_fd_ = epoll_create1(0);
    if (epoll_fd_ == -1) {
        throw std::runtime_error("Failed to create epoll file descriptor");
    }
}

Reactor::~Reactor() {
    close(epoll_fd_);
}

void Reactor::add_handler(int fd, uint32_t events, std::function<void()> handler) {
    epoll_event ev;
    ev.events = events;
    ev.data.fd = fd;

    if (epoll_ctl(epoll_fd_, EPOLL_CTL_ADD, fd, &ev) == -1) {
        throw std::runtime_error("Failed to add file descriptor to epoll");
    }

    handlers_[fd] = std::move(handler);
}

void Reactor::remove_handler(int fd) {
    auto it = handlers_.find(fd);
    if (it == handlers_.end()) {
        // 处理程序不在我们的映射中，可能已经被移除
        return;
    }

    if (epoll_ctl(epoll_fd_, EPOLL_CTL_DEL, fd, nullptr) == -1) {
        if (errno == EBADF) {
            // 文件描述符无效，可能已经被关闭
            // 我们只需要从 handlers_ 中移除它
            handlers_.erase(it);
        } else if (errno == ENOENT) {
            // 文件描述符不在 epoll 实例中
            // 这是一个意外情况，因为它在我们的 handlers_ 中
            // 我们应该记录这个情况，但仍然从 handlers_ 中移除它
            std::cerr << "Warning: File descriptor " << fd << " not found in epoll instance\n";
            handlers_.erase(it);
        } else {
            // 其他错误，可能需要更严重的处理
            throw std::runtime_error("Failed to remove file descriptor from epoll: ");
        }
    } else {
        // 成功从 epoll 实例中移除，现在从 handlers_ 中移除
        handlers_.erase(it);
    }
}

void Reactor::run() {
    running_ = true;
    while (running_) {
        int nfds = epoll_wait(epoll_fd_, events_.data(), MAX_EVENTS, -1);
        if (nfds == -1) {
            if (errno == EINTR) continue;  // 被信号中断，继续循环
            throw std::runtime_error("epoll_wait failed");
        }

        for (int n = 0; n < nfds; ++n) {
            int fd = events_[n].data.fd;
            auto it = handlers_.find(fd);
            if (it != handlers_.end()) {
                it->second();  // 调用处理函数
            }
        }
    }
}

void Reactor::stop() {
    running_ = false;
}

int Reactor::get_epoll_fd()
{
    return epoll_fd_;
}