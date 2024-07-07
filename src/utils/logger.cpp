#include "utils/logger.h"
#include <iostream>
#include <chrono>
#include <iomanip>
#include <thread>

Logger& Logger::instance() {
    static Logger instance;
    return instance;
}

Logger::Logger() : current_level_(LogLevel::INFO) {}

Logger::~Logger() {
    if (log_file_.is_open()) {
        log_file_.close();
    }
}

void Logger::set_log_level(LogLevel level) {
    current_level_ = level;
}

void Logger::set_log_file(const std::string& filename) {
    std::lock_guard<std::mutex> lock(mutex_);
    if (log_file_.is_open()) {
        log_file_.close();
    }
    log_file_.open(filename, std::ios::app);
}

void Logger::debug(const std::string& message) {
    log(LogLevel::DEBUG, message);
}

void Logger::info(const std::string& message) {
    log(LogLevel::INFO, message);
}

void Logger::warning(const std::string& message) {
    log(LogLevel::WARNING, message);
}

void Logger::error(const std::string& message) {
    log(LogLevel::ERROR, message);
}

void Logger::critical(const std::string& message) {
    log(LogLevel::CRITICAL, message);
}

void Logger::log(LogLevel level, const std::string& message) {
    if (level < current_level_) {
        return;
    }

    std::lock_guard<std::mutex> lock(mutex_);

    auto now = std::chrono::system_clock::now();
    auto now_c = std::chrono::system_clock::to_time_t(now);
    auto now_ms = std::chrono::duration_cast<std::chrono::milliseconds>(now.time_since_epoch()) % 1000;

    std::stringstream ss;
    ss << std::put_time(std::localtime(&now_c), "%Y-%m-%d %H:%M:%S")
       << '.' << std::setfill('0') << std::setw(3) << now_ms.count()
       << " [" << std::this_thread::get_id() << "] ";

    switch (level) {
        case LogLevel::DEBUG: ss << "DEBUG"; break;
        case LogLevel::INFO: ss << "INFO"; break;
        case LogLevel::WARNING: ss << "WARNING"; break;
        case LogLevel::ERROR: ss << "ERROR"; break;
        case LogLevel::CRITICAL: ss << "CRITICAL"; break;
    }

    ss << ": " << message;

    std::cout << ss.str() << std::endl;
    if (log_file_.is_open()) {
        log_file_ << ss.str() << std::endl;
    }
}