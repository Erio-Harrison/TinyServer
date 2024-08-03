#include "http/async_log.h"
#include <chrono>
#include <iomanip>

AsyncLog::AsyncLog()
    : currentBuffer_(new Buffer),
      nextBuffer_(new Buffer),
      running_(true)
{
    currentBuffer_->reserve(BUFFER_SIZE);
    nextBuffer_->reserve(BUFFER_SIZE);
    buffers_.reserve(16);
    logFile_.open("server.log", std::ios::app);
    writeThread_ = std::thread(&AsyncLog::threadFunc, this);
}

AsyncLog::~AsyncLog() {
    if (running_) {
        stop();
    }
}

void AsyncLog::append(const std::string& log) {
    std::lock_guard<std::mutex> lock(mutex_);
    if (currentBuffer_->size() < BUFFER_SIZE) {
        currentBuffer_->push_back(log);
    } else {
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

void AsyncLog::stop() {
    running_ = false;
    cond_.notify_one();
    writeThread_.join();
}

void AsyncLog::threadFunc() {
    BufferPtr newBuffer1(new Buffer);
    BufferPtr newBuffer2(new Buffer);
    newBuffer1->reserve(BUFFER_SIZE);
    newBuffer2->reserve(BUFFER_SIZE);
    std::vector<BufferPtr> buffersToWrite;
    buffersToWrite.reserve(16);

    while (running_) {
        {
            std::unique_lock<std::mutex> lock(mutex_);
            if (buffers_.empty()) {
                cond_.wait_for(lock, std::chrono::seconds(3));
            }
            buffers_.push_back(std::move(currentBuffer_));
            currentBuffer_ = std::move(newBuffer1);
            buffersToWrite.swap(buffers_);
            if (!nextBuffer_) {
                nextBuffer_ = std::move(newBuffer2);
            }
        }

        if (buffersToWrite.size() > 25) {
            // 日志堆积过多，丢弃一些旧日志
            buffersToWrite.erase(buffersToWrite.begin() + 2, buffersToWrite.end());
        }

        for (const auto& buffer : buffersToWrite) {
            for (const std::string& msg : *buffer) {
                logFile_ << msg;
            }
        }

        if (logFile_.tellp() > 64 * 1024 * 1024) {
            // 日志文件超过64MB，轮换日志文件
            logFile_.close();
            // 这里可以添加日志文件轮换逻辑
            logFile_.open("server.log", std::ios::app);
        }

        logFile_.flush();

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

        buffersToWrite.clear();
    }

    // 确保所有剩余的日志都被写入
    for (const auto& buffer : buffersToWrite) {
        for (const std::string& msg : *buffer) {
            logFile_ << msg;
        }
    }
    logFile_.flush();
}