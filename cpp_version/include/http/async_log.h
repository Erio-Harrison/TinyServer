#pragma once
#include <string>
#include <vector>
#include <thread>
#include <mutex>
#include <condition_variable>
#include <atomic>
#include <fstream>

class AsyncLog {
public:
    static AsyncLog& getInstance() {
        static AsyncLog instance;
        return instance;
    }

    void append(const std::string& log);
    void stop();

private:
    AsyncLog();
    ~AsyncLog();
    AsyncLog(const AsyncLog&) = delete;
    AsyncLog& operator=(const AsyncLog&) = delete;

    void threadFunc();

    typedef std::vector<std::string> Buffer;
    typedef std::unique_ptr<Buffer> BufferPtr;

    BufferPtr currentBuffer_;
    BufferPtr nextBuffer_;
    std::vector<BufferPtr> buffers_;

    std::thread writeThread_;
    std::mutex mutex_;
    std::condition_variable cond_;
    std::atomic<bool> running_;
    std::ofstream logFile_;

    const size_t BUFFER_SIZE = 4000 * 1000;  // 4MB
};