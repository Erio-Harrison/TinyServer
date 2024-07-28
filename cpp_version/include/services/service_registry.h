#pragma once

#include <string>
#include <vector>
#include <unordered_map>
#include <mutex>
#include <memory>

struct ServiceInstance {
    std::string host;
    int port;
    // 可以添加其他元数据，如健康状态、负载等

    ServiceInstance(std::string h, int p) : host(std::move(h)), port(p) {}
};

class ServiceRegistry {
public:
    void register_service(const std::string& service_name, std::string host, int port);
    void deregister_service(const std::string& service_name, const std::string& host, int port);
    std::vector<ServiceInstance> get_service_instances(const std::string& service_name);

private:
    std::unordered_map<std::string, std::vector<ServiceInstance>> services_;
    std::mutex mutex_;
};