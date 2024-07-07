#include "services/service_registry.h"
#include <algorithm>

void ServiceRegistry::register_service(const std::string& service_name, std::string host, int port) {
    std::lock_guard<std::mutex> lock(mutex_);
    auto& instances = services_[service_name];
    instances.emplace_back(std::move(host), port);
}

void ServiceRegistry::deregister_service(const std::string& service_name, const std::string& host, int port) {
    std::lock_guard<std::mutex> lock(mutex_);
    auto it = services_.find(service_name);
    if (it != services_.end()) {
        auto& instances = it->second;
        instances.erase(
            std::remove_if(instances.begin(), instances.end(),
                [&](const ServiceInstance& instance) {
                    return instance.host == host && instance.port == port;
                }),
            instances.end());
    }
}

std::vector<ServiceInstance> ServiceRegistry::get_service_instances(const std::string& service_name) {
    std::lock_guard<std::mutex> lock(mutex_);
    auto it = services_.find(service_name);
    if (it != services_.end()) {
        return it->second;
    }
    return {};
}