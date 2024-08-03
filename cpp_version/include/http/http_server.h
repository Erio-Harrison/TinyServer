#ifndef HTTP_SERVER_H
#define HTTP_SERVER_H

#include "network/tcp_server.h"
#include "mysql/MySQLDatabase.h"
#include "http/http_parser.h"
#include "http/async_log.h"
#include <string>
#include <unordered_map>
#include <functional>

class HttpServer : public TcpServer {
public:
    HttpServer(Reactor& reactor, const std::string& ip, int port, MySQLDatabaseManager& db_manager);

private:
    void handle_request(int client_fd, const char* data, size_t len);
    void send_response(int client_fd, const std::string& response, const std::string& content_type = "text/html");
    std::string process_request(const HttpRequest& request);
    std::string handle_register(const HttpRequest& request);
    std::string handle_login(const HttpRequest& request);
    std::string handle_static_file(const std::string& path);

    MySQLDatabaseManager& db_manager_;
    std::unordered_map<std::string, std::function<std::string(const HttpRequest&)>> route_handlers_;
    HttpParser parser_;

    std::unordered_map<std::string, std::string> sessions_;
    std::string generate_session_id();
    void set_session_cookie(std::string& response, const std::string& session_id);
};

#endif