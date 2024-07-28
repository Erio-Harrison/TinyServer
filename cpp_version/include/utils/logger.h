#pragma once

#include <string>
#include <fstream>
#include <mutex>
#include <memory>

enum class LogLevel {
    DEBUG,
    INFO,
    WARNING,
    ERROR,
    CRITICAL
};

class Logger {
public:
    static Logger& instance();

    void set_log_level(LogLevel level);
    void set_log_file(const std::string& filename);

    void debug(const std::string& message);
    void info(const std::string& message);
    void warning(const std::string& message);
    void error(const std::string& message);
    void critical(const std::string& message);

private:
    Logger();
    ~Logger();

    void log(LogLevel level, const std::string& message);

    LogLevel current_level_;
    std::ofstream log_file_;
    std::mutex mutex_;
};

#define LOG_DEBUG(message) Logger::instance().debug(message)
#define LOG_INFO(message) Logger::instance().info(message)
#define LOG_WARNING(message) Logger::instance().warning(message)
#define LOG_ERROR(message) Logger::instance().error(message)
#define LOG_CRITICAL(message) Logger::instance().critical(message)