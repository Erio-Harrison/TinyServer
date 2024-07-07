#pragma once

#include "services/service_registry.h"
#include <memory>
#include <atomic>

class LoadBalancer {
public:
    explicit LoadBalancer(std::shared_ptr<ServiceRegistry> registry);

    ServiceInstance get_next_instance(const std::string& service_name);

private:
    std::shared_ptr<ServiceRegistry> registry_;
    std::atomic<size_t> round_robin_counter_{0};
};