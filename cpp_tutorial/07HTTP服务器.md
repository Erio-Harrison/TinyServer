HTTP是基于TCP实现的，理念就是我们在TCPserver部分说的那样，应用层是基于传输层实现的。

我们现在还拥有了管理数据库的类，接下来要做的就是基于HTTP协议，根据我们的服务需要，进行增删查改:

定义一个辅助函数 parse_form_data，用于解析 HTTP POST 请求的表单数据：

```bash
std::unordered_map<std::string, std::string> parse_form_data(const std::string& body) {
    std::unordered_map<std::string, std::string> data;
    std::istringstream stream(body);
    std::string pair;
    while (std::getline(stream, pair, '&')) {
        auto pos = pair.find('=');
        if (pos != std::string::npos) {
            std::string key = pair.substr(0, pos);
            std::string value = pair.substr(pos + 1);
            data[key] = value;
        }
    }
    return data;
}
```

handle_request 方法会处理接收到的 HTTP 请求。解析请求，记录日志，然后处理请求并发送响应:

```bash
void HttpServer::handle_request(int client_fd, const char* data, size_t len) {
    parser_.reset();
    if (parser_.parse(data, len)) {
        const HttpRequest& request = parser_.get_request();
        auto now = std::chrono::system_clock::now();
        auto now_c = std::chrono::system_clock::to_time_t(now);
        std::stringstream ss;
        ss << std::put_time(std::localtime(&now_c), "%F %T") 
           << " - Received request: " << request.method << " " << request.url << "\n";
        AsyncLog::getInstance().append(ss.str());
        
        std::string response = process_request(request);
        send_response(client_fd, response);
    } else {
        AsyncLog::getInstance().append("Bad request received\n");
        send_response(client_fd, "400 Bad Request", "text/plain");
    }
}
```
send_response 方法会发送 HTTP 响应。构造完整的 HTTP 响应，记录日志，然后发送给客户端：

```bash
void HttpServer::send_response(int client_fd, const std::string& response, const std::string& content_type) {
    std::ostringstream oss;
    oss << "HTTP/1.1 200 OK\r\n"
        << "Content-Type: " << content_type << "\r\n"
        << "Content-Length: " << response.length() << "\r\n"
        << "\r\n"
        << response;
    std::string full_response = oss.str();
    
    // 记录日志
    auto now = std::chrono::system_clock::now();
    auto now_c = std::chrono::system_clock::to_time_t(now);
    std::stringstream ss;
    ss << std::put_time(std::localtime(&now_c), "%F %T")
       << " - Sent response to client_fd " << client_fd << ": " << full_response.substr(0, 50) << "...\n";
    AsyncLog::getInstance().append(ss.str());

    send(client_fd, full_response.c_str(), full_response.length());
}
```

process_request 方法会根据请求 URL 处理不同的路由。包括静态文件处理和自定义路由处理：

```bash
std::string HttpServer::process_request(const HttpRequest& request) {
    // 记录处理请求日志
    auto now = std::chrono::system_clock::now();
    auto now_c = std::chrono::system_clock::to_time_t(now);
    std::stringstream ss;
    ss << std::put_time(std::localtime(&now_c), "%F %T")
       << " - Processing request for URL: " << request.url << "\n";
    AsyncLog::getInstance().append(ss.str());

    if (request.url.find("/static/") == 0) {
        return handle_static_file(request.url.substr(7));
    }

    auto handler = route_handlers_.find(request.url);
    if (handler != route_handlers_.end()) {
        return handler->second(request);
    } else {
        return "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\n\r\n404 Not Found";
    }
}
```
handle_register 方法会处理用户注册请求。验证请求方法和参数，检查用户名是否存在，然后尝试注册新用户：

```bash
std::string HttpServer::handle_register(const HttpRequest& request) {
    auto now = std::chrono::system_clock::now();
    auto now_c = std::chrono::system_clock::to_time_t(now);
    std::stringstream ss;

    if (request.method != "POST") {
        ss << std::put_time(std::localtime(&now_c), "%F %T")
           << " - Register attempt with invalid method: " << request.method << "\n";
        AsyncLog::getInstance().append(ss.str());
        return "HTTP/1.1 405 Method Not Allowed\r\nContent-Type: text/plain\r\n\r\nMethod Not Allowed";
    }

    auto form_data = parse_form_data(request.body);
    
    if (form_data.find("username") == form_data.end() || form_data.find("password") == form_data.end()) {
        ss << std::put_time(std::localtime(&now_c), "%F %T")
           << " - Register attempt with missing username or password\n";
        AsyncLog::getInstance().append(ss.str());
        return "HTTP/1.1 400 Bad Request\r\nContent-Type: text/plain\r\n\r\nMissing username or password";
    }

    std::string username = form_data["username"];
    std::string password = form_data["password"];

    if (username.empty() || password.empty()) {
        ss << std::put_time(std::localtime(&now_c), "%F %T")
           << " - Register attempt with empty username or password\n";
        AsyncLog::getInstance().append(ss.str());
        return "HTTP/1.1 400 Bad Request\r\nContent-Type: text/plain\r\n\r\nUsername and password cannot be empty";
    }

    // 检查用户名是否已存在
    auto conn = db_manager_.get_connection();
    std::string check_query = "SELECT * FROM users WHERE username = '" + username + "'";
    bool user_exists = !db_manager_.execute_query(check_query);
    
    if (user_exists) {
        db_manager_.release_connection(std::move(conn));
        ss << std::put_time(std::localtime(&now_c), "%F %T")
           << " - Register attempt with existing username: " << username << "\n";
        AsyncLog::getInstance().append(ss.str());
        return "HTTP/1.1 409 Conflict\r\nContent-Type: text/plain\r\n\r\nUsername already exists";
    }

    // 注册新用户
    std::string insert_query = "INSERT INTO users (username, password) VALUES ('" + username + "', '" + password + "')";
    bool success = db_manager_.execute_query(insert_query);
    db_manager_.release_connection(std::move(conn));

    if (success) {
        ss << std::put_time(std::localtime(&now_c), "%F %T")
           << " - User registered successfully: " << username << "\n";
        AsyncLog::getInstance().append(ss.str());
        return "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nUser registered successfully";
    } else {
        ss << std::put_time(std::localtime(&now_c), "%F %T")
           << " - Failed to register user: " << username << "\n";
        AsyncLog::getInstance().append(ss.str());
        return "HTTP/1.1 500 Internal Server Error\r\nContent-Type: text/plain\r\n\r\nFailed to register user";
    }
}
```

handle_login 方法会处理用户登录请求。验证请求方法和参数，然后尝试验证用户凭据：

```bash
std::string HttpServer::handle_login(const HttpRequest& request) {
    auto now = std::chrono::system_clock::now();
    auto now_c = std::chrono::system_clock::to_time_t(now);
    std::stringstream ss;

    if (request.method != "POST") {
        ss << std::put_time(std::localtime(&now_c), "%F %T")
           << " - Login attempt with invalid method: " << request.method << "\n";
        AsyncLog::getInstance().append(ss.str());
        return "HTTP/1.1 405 Method Not Allowed\r\nContent-Type: text/plain\r\n\r\nMethod Not Allowed";
    }

    auto form_data = parse_form_data(request.body);

    if (form_data.find("username") == form_data.end() || form_data.find("password") == form_data.end()) {
        ss << std::put_time(std::localtime(&now_c), "%F %T")
           << " - Login attempt with missing username or password\n";
        AsyncLog::getInstance().append(ss.str());
        return "HTTP/1.1 400 Bad Request\r\nContent-Type: text/plain\r\n\r\nMissing username or password";
    }

    std::string username = form_data["username"];
    std::string password = form_data["password"];

    if (username.empty() || password.empty()) {
        ss << std::put_time(std::localtime(&now_c), "%F %T")
           << " - Login attempt with empty username or password\n";
        AsyncLog::getInstance().append(ss.str());
        return "HTTP/1.1 400 Bad Request\r\nContent-Type: text/plain\r\n\r\nUsername and password cannot be empty";
    }

    auto conn = db_manager_.get_connection();
    std::string query = "SELECT * FROM users WHERE username = '" + username + "' AND password = '" + password + "'";
    auto result = db_manager_.execute_query(query);
    db_manager_.release_connection(std::move(conn));

    if (!result) {
        ss << std::put_time(std::localtime(&now_c), "%F %T")
           << " - Login successful: " << username << "\n";
        AsyncLog::getInstance().append(ss.str());
        return "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nLogin successful";
    } else {
        ss << std::put_time(std::localtime(&now_c), "%F %T")
           << " - Login failed: " << username << "\n";
        AsyncLog::getInstance().append(ss.str());
        return "HTTP/1.1 401 Unauthorized\r\nContent-Type: text/plain\r\n\r\nInvalid username or password";
    }
}
```

handle_static_file 方法会处理静态文件请求。读取文件内容，确定内容类型，然后构造响应：

```bash
std::string HttpServer::handle_static_file(const std::string& path) {
    auto now = std::chrono::system_clock::now();
    auto now_c = std::chrono::system_clock::to_time_t(now);
    std::stringstream ss;
    ss << std::put_time(std::localtime(&now_c), "%F %T")
       << " - Serving static file: " << path << "\n";
    AsyncLog::getInstance().append(ss.str());

    std::ifstream file(path, std::ios::binary);
    if (!file) {
        ss << std::put_time(std::localtime(&now_c), "%F %T")
           << " - Static file not found: " << path << "\n";
        AsyncLog::getInstance().append(ss.str());
        return "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\n\r\n404 Not Found";
    }
    std::string content((std::istreambuf_iterator<char>(file)), std::istreambuf_iterator<char>());
    
    // 根据文件扩展名决定 Content-Type
    std::string content_type = "application/octet-stream"; // 默认
    size_t dot_pos = path.find_last_of('.');
    if (dot_pos != std::string::npos) {
        std::string ext = path.substr(dot_pos + 1);
        if (ext == "html" || ext == "htm") content_type = "text/html";
        else if (ext == "css") content_type = "text/css";
        else if (ext == "js") content_type = "application/javascript";
        else if (ext == "jpg" || ext == "jpeg") content_type = "image/jpeg";
        else if (ext == "png") content_type = "image/png";
        else if (ext == "gif") content_type = "image/gif";
    }

    std::ostringstream oss;
    oss << "HTTP/1.1 200 OK\r\n"
        << "Content-Type: " << content_type << "\r\n"
        << "Content-Length: " << content.length() << "\r\n"
        << "\r\n"
        << content;
    return oss.str();
}
```