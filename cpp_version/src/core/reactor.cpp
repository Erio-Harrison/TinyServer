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

void Reactor::add_handler(int fd, std::function<void()> handler) {
    epoll_event ev;
    ev.events = EPOLLIN;
    ev.data.fd = fd;

    if (epoll_ctl(epoll_fd_, EPOLL_CTL_ADD, fd, &ev) == -1) {
        throw std::runtime_error("Failed to add file descriptor to epoll");
    }

    handlers_[fd] = std::move(handler);
}

void Reactor::remove_handler(int fd) {
    if (epoll_ctl(epoll_fd_, EPOLL_CTL_DEL, fd, nullptr) == -1) {
        throw std::runtime_error("Failed to remove file descriptor from epoll");
    }

    handlers_.erase(fd);
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