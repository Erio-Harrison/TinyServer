#ifndef HTTP_PARSER_H
#define HTTP_PARSER_H

#include <string>
#include <unordered_map>

enum class HttpParserState {
    METHOD,
    URL,
    PROTOCOL,
    HEADER_KEY,
    HEADER_VALUE,
    BODY
};

struct HttpRequest {
    std::string method;
    std::string url;
    std::string protocol;
    std::unordered_map<std::string, std::string> headers;
    std::string body;
};

class HttpParser {
public:
    HttpParser();
    void reset();
    bool parse(const char* data, size_t len);
    const HttpRequest& get_request() const { return request_; }

private:
    HttpParserState state_;
    HttpRequest request_;
    std::string current_header_key_;
};

#endif