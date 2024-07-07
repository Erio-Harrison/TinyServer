#include "services/load_balancer.h"

LoadBalancer::LoadBalancer(std::shared_ptr<ServiceRegistry> registry)
    : registry_(std::move(registry)) {}

ServiceInstance LoadBalancer::get_next_instance(const std::string& service_name) {
    auto instances = registry_->get_service_instances(service_name);
    if (instances.empty()) {
        throw std::runtime_error("No instances available for service: " + service_name);
    }

    // 简单的轮询策略
    size_t index = round_robin_counter_++ % instances.size();
    return instances[index];
}