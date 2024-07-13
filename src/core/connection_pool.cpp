#include "core/connection_pool.h"

ConnectionPool::ConnectionPool(size_t max_connections, std::function<std::unique_ptr<Connection>()> connection_factory)
    : max_connections_(max_connections), connection_factory_(std::move(connection_factory)) {}

ConnectionPool::~ConnectionPool() {
    all_connections_.clear();
}

std::unique_ptr<Connection> ConnectionPool::get_connection() {
    std::unique_lock<std::mutex> lock(mutex_);
    
    while (available_connections_.empty() && all_connections_.size() < max_connections_) {
        all_connections_.push_back(connection_factory_());
        available_connections_.push(all_connections_.back().get());
    }

    cv_.wait(lock, [this] { return !available_connections_.empty(); });

    auto* conn = available_connections_.front();
    available_connections_.pop();

    return std::unique_ptr<Connection>(conn);
}

void ConnectionPool::release_connection(std::unique_ptr<Connection> conn) {
    std::lock_guard<std::mutex> lock(mutex_);
    available_connections_.push(conn.release());
    cv_.notify_one();
}