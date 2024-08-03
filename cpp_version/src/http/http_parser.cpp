#include "http/http_parser.h"

HttpParser::HttpParser() : state_(HttpParserState::METHOD) {}

void HttpParser::reset() {
    state_ = HttpParserState::METHOD;
    request_ = HttpRequest();
    current_header_key_.clear();
}

bool HttpParser::parse(const char* data, size_t len) {
    for (size_t i = 0; i < len; ++i) {
        char c = data[i];
        switch (state_) {
            case HttpParserState::METHOD:
                if (c == ' ') {
                    state_ = HttpParserState::URL;
                } else {
                    request_.method += c;
                }
                break;
            case HttpParserState::URL:
                if (c == ' ') {
                    state_ = HttpParserState::PROTOCOL;
                } else {
                    request_.url += c;
                }
                break;
            case HttpParserState::PROTOCOL:
                if (c == '\r') {
                    // Ignore
                } else if (c == '\n') {
                    state_ = HttpParserState::HEADER_KEY;
                } else {
                    request_.protocol += c;
                }
                break;
            case HttpParserState::HEADER_KEY:
                if (c == ':') {
                    state_ = HttpParserState::HEADER_VALUE;
                } else if (c == '\r') {
                    // Ignore
                } else if (c == '\n') {
                    state_ = HttpParserState::BODY;
                } else {
                    current_header_key_ += c;
                }
                break;
            case HttpParserState::HEADER_VALUE:
                if (c == '\r') {
                    // Ignore
                } else if (c == '\n') {
                    request_.headers[current_header_key_] = current_header_key_;
                    current_header_key_.clear();
                    state_ = HttpParserState::HEADER_KEY;
                } else if (c != ' ' || !current_header_key_.empty()) {
                    current_header_key_ += c;
                }
                break;
            case HttpParserState::BODY:
                request_.body += c;
                break;
        }
    }
    return true;
}