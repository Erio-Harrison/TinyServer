#include "utils/metrics.h"

Metrics& Metrics::instance() {
    static Metrics instance;
    return instance;
}

void Metrics::increment_counter(const std::string& name) {
    counters_[name]++;
}

void Metrics::set_gauge(const std::string& name, double value) {
    gauges_[name].store(value);
}

void Metrics::update_histogram(const std::string& name, double value) {
    auto& histogram = histograms_[name];
    double old_value = histogram.first.load(std::memory_order_relaxed);
    double new_value;
    do {
        new_value = old_value + value;
    } while (!histogram.first.compare_exchange_weak(old_value, new_value, 
                                                    std::memory_order_relaxed,
                                                    std::memory_order_relaxed));
    histogram.second++;
}

std::unordered_map<std::string, int64_t> Metrics::get_counters() {
    std::unordered_map<std::string, int64_t> result;
    for (const auto& pair : counters_) {
        result[pair.first] = pair.second.load();
    }
    return result;
}

std::unordered_map<std::string, double> Metrics::get_gauges() {
    std::unordered_map<std::string, double> result;
    for (const auto& pair : gauges_) {
        result[pair.first] = pair.second.load();
    }
    return result;
}

std::unordered_map<std::string, std::pair<double, int64_t>> Metrics::get_histograms() {
    std::unordered_map<std::string, std::pair<double, int64_t>> result;
    for (const auto& pair : histograms_) {
        result[pair.first] = {pair.second.first.load(), pair.second.second.load()};
    }
    return result;
}