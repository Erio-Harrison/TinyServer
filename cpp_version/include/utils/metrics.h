#pragma once

#include <string>
#include <unordered_map>
#include <mutex>
#include <atomic>

class Metrics {
public:
    static Metrics& instance();

    void increment_counter(const std::string& name);
    void set_gauge(const std::string& name, double value);
    void update_histogram(const std::string& name, double value);

    std::unordered_map<std::string, int64_t> get_counters();
    std::unordered_map<std::string, double> get_gauges();
    std::unordered_map<std::string, std::pair<double, int64_t>> get_histograms();

private:
    std::unordered_map<std::string, std::atomic<int64_t>> counters_;
    std::unordered_map<std::string, std::atomic<double>> gauges_;
    std::unordered_map<std::string, std::pair<std::atomic<double>, std::atomic<int64_t>>> histograms_;
    std::mutex mutex_;
};

#define INCREMENT_COUNTER(name) Metrics::instance().increment_counter(name)
#define SET_GAUGE(name, value) Metrics::instance().set_gauge(name, value)
#define UPDATE_HISTOGRAM(name, value) Metrics::instance().update_histogram(name, value)