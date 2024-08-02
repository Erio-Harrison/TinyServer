#include "../include/network/tcp_server.h"
#include <string>
#include <unordered_map>
#include <functional>
#include <sstream>

class HttpServer : public TcpServer {
public:
    using RequestHandler = std::function<void(const std::string&, const std::string&, const std::unordered_map<std::string, std::string>&, std::string&)>;

    HttpServer(Reactor& reactor, const std::string& ip, int port);

    void add_route(const std::string& path, RequestHandler handler);

private:
    void handle_request(int client_fd, const char* data, size_t len);
    void parse_request(const std::string& request, std::string& method, std::string& path, std::unordered_map<std::string, std::string>& headers);
    void send_response(int client_fd, const std::string& status, const std::string& content_type, const std::string& body);

    std::unordered_map<std::string, RequestHandler> routes_;
};

HttpServer::HttpServer(Reactor& reactor, const std::string& ip, int port)
    : TcpServer(reactor, ip, port) {
    set_receive_handler([this](int client_fd, const char* data, size_t len) {
        handle_request(client_fd, data, len);
    });
}

void HttpServer::add_route(const std::string& path, RequestHandler handler) {
    routes_[path] = std::move(handler);
}

void HttpServer::handle_request(int client_fd, const char* data, size_t len) {
    std::string request(data, len);
    std::string method, path;
    std::unordered_map<std::string, std::string> headers;

    parse_request(request, method, path, headers);

    auto it = routes_.find(path);
    if (it != routes_.end()) {
        std::string response;
        it->second(method, path, headers, response);
        send_response(client_fd, "200 OK", "text/html", response);
    } else {
        send_response(client_fd, "404 Not Found", "text/plain", "404 Not Found");
    }
}

void HttpServer::parse_request(const std::string& request, std::string& method, std::string& path, std::unordered_map<std::string, std::string>& headers) {
    std::istringstream iss(request);
    std::string line;

    // Parse request line
    std::getline(iss, line);
    std::istringstream request_line(line);
    request_line >> method >> path;

    // Parse headers
    while (std::getline(iss, line) && line != "\r") {
        auto colon_pos = line.find(':');
        if (colon_pos != std::string::npos) {
            std::string key = line.substr(0, colon_pos);
            std::string value = line.substr(colon_pos + 1);
            headers[key] = value;
        }
    }
}

void HttpServer::send_response(int client_fd, const std::string& status, const std::string& content_type, const std::string& body) {
    std::ostringstream response;
    response << "HTTP/1.1 " << status << "\r\n";
    response << "Content-Type: " << content_type << "\r\n";
    response << "Content-Length: " << body.length() << "\r\n";
    response << "\r\n";
    response << body;

    send(client_fd, response.str().c_str(), response.str().length());
}

int main() {
    Reactor reactor;
    HttpServer server(reactor, "0.0.0.0", 8080);

    server.add_route("/", [](const std::string& method, const std::string& path, 
                             const std::unordered_map<std::string, std::string>& headers, 
                             std::string& response) {
        response = "<html><body><h1>Welcome to the HTTP Server!</h1></body></html>";
    });

    server.add_route("/hello", [](const std::string& method, const std::string& path, 
                                  const std::unordered_map<std::string, std::string>& headers, 
                                  std::string& response) {
        response = "<html><body><h1>Hello, World!</h1></body></html>";
    });

    server.start();
    reactor.run();

    return 0;
}