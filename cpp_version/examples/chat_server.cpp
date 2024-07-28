#include "network/tcp_server.h"
#include "core/reactor.h"
#include "utils/logger.h"
#include "utils/metrics.h"
#include <unordered_map>
#include <string>
#include <iostream>

class ChatServer {
public:
    ChatServer(Reactor& reactor, const std::string& ip, int port)
        : server_(reactor, ip, port) {
        server_.set_connection_handler([this](int client_fd) { handle_connection(client_fd); });
        server_.set_receive_handler([this](int client_fd, const char* data, size_t len) { handle_message(client_fd, data, len); });
    }

    void start() {
        LOG_INFO("Starting chat server");
        server_.start();
    }

private:
    void handle_connection(int client_fd) {
        if (client_fd > 0) {
            LOG_INFO("New client connected: " + std::to_string(client_fd));
            clients_[client_fd] = "User" + std::to_string(client_fd);
            INCREMENT_COUNTER("connected_clients");
        } else {
            LOG_INFO("Client disconnected: " + std::to_string(-client_fd));
            clients_.erase(-client_fd);
            INCREMENT_COUNTER("disconnected_clients");
        }
    }

    void handle_message(int client_fd, const char* data, size_t len) {
        std::string message(data, len);
        LOG_INFO("Received message from " + clients_[client_fd] + ": " + message);
        INCREMENT_COUNTER("messages_received");
        UPDATE_HISTOGRAM("message_size", len);

        std::string broadcast = clients_[client_fd] + ": " + message;
        for (const auto& client : clients_) {
            if (client.first != client_fd) {
                server_.send(client.first, broadcast.c_str(), broadcast.length());
                INCREMENT_COUNTER("messages_sent");
            }
        }
    }

    TcpServer server_;
    std::unordered_map<int, std::string> clients_;
};

int main() {
    Logger::instance().set_log_level(LogLevel::INFO);
    Logger::instance().set_log_file("chat_server.log");

    Reactor reactor;
    ChatServer chat_server(reactor, "0.0.0.0", 8080);

    chat_server.start();

    LOG_INFO("Chat server started. Press Ctrl+C to exit.");
    reactor.run();

    return 0;
}