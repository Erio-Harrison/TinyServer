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

这个系统的核心数据结构是这里的双缓冲区设计：

```bash
typedef std::vector<std::string> Buffer;
typedef std::unique_ptr<Buffer> BufferPtr;

BufferPtr currentBuffer_;     // 当前缓冲区
BufferPtr nextBuffer_;        // 备用缓冲区
std::vector<BufferPtr> buffers_; // 待写入的缓冲区列表
```

这是我的设计，跟着做的朋友可以用自己喜欢的数据结构。

在追加内容的时候：我们应当使用锁保护共享资源，在缓冲区未满时直接写入，缓冲区满时进行缓冲区切换，最后通知后台线程处理：

```bash
void AsyncLog::append(const std::string& log) {
    std::lock_guard<std::mutex> lock(mutex_);
    if (currentBuffer_->size() < BUFFER_SIZE) {
        currentBuffer_->push_back(log);
    } else {
        // 当前缓冲区满，切换到下一个
        buffers_.push_back(std::move(currentBuffer_));
        if (nextBuffer_) {
            currentBuffer_ = std::move(nextBuffer_);
        } else {
            currentBuffer_.reset(new Buffer);
        }
        currentBuffer_->push_back(log);
        cond_.notify_one();
    }
}
```

然后就是主线程了，注意看这里：

```bash
// 在 threadFunc() 中的批量写入逻辑
for (const auto& buffer : buffersToWrite) {
    for (const std::string& msg : *buffer) {
        logFile_ << msg;
    }
}
```

这里不是每条日志都立即写入文件，而是将多条日志收集在缓冲区中，然后一次性写入。这样减少了 I/O 操作的次数，提高了效率。

```bash
if (!newBuffer1) {
    newBuffer1 = std::move(buffersToWrite.back());
    buffersToWrite.pop_back();
    newBuffer1->clear();
}

if (!newBuffer2) {
    newBuffer2 = std::move(buffersToWrite.back());
    buffersToWrite.pop_back();
    newBuffer2->clear();
}
```

这段代码不是简单地删除用过的缓冲区，而是将其清空后重新使用，避免了频繁的内存分配和释放。

```bash
if (logFile_.tellp() > 64 * 1024 * 1024) {
    // 日志文件超过64MB，轮换日志文件
    logFile_.close();
    // 这里可以添加日志文件轮换逻辑
    logFile_.open("server.log", std::ios::app);
}
```

这里通过检查文件大小并在超过阈值时进行轮换，防止单个日志文件过大。

```bash
if (buffersToWrite.size() > 25) {
    // 日志堆积过多，丢弃一些旧日志
    buffersToWrite.erase(buffersToWrite.begin() + 2, buffersToWrite.end());
}
```

这里是一个防止日志文件堆积的保护机制，当待写入的缓冲区数量过多时（超过25个），会丢弃部分旧日志，这是一种保护机制，防止因日志写入速度跟不上而导致内存耗尽。