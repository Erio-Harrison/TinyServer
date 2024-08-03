这里我们用到一个叫做有限状态机的概念：解析器的核心是一个状态机。它定义了不同的状态（如METHOD, URL, HEADER_KEY等），并根据输入字符在这些状态之间转换。这种方法很适合处理具有明确格式的文本，如HTTP请求。

然后我们HTTP请求的基本结构是这样的：

```bash
METHOD URL HTTP-VERSION
Header1: Value1
Header2: Value2
...

Body
```

我们根据这个结构定义了下面这个枚举类：

```bash
enum class HttpParserState {
    METHOD,
    URL,
    PROTOCOL,
    HEADER_KEY,
    HEADER_VALUE,
    BODY
};
```
它的解析原理是：

```bash
METHOD URL PROTOCOL\r\n   (解析器在 METHOD、URL、PROTOCOL 状态间转换)
Header1: Value1\r\n       (在 HEADER_KEY 和 HEADER_VALUE 状态间转换)
Header2: Value2\r\n       (重复 HEADER_KEY 和 HEADER_VALUE)
\r\n                      (空行标志头部结束)
Body                      (进入 BODY 状态)
```

解析过程就是完成一次for循环和相应的处理：

```bash
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
```

日志系统是帮我们记录信息用的：