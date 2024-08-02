#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    // 模拟一个简单的 ServiceInstance 结构体
    #[derive(Clone, Debug, PartialEq)]
    struct ServiceInstance {
        id: String,
    }

    impl ServiceInstance {
        fn new(id: &str) -> Self {
            ServiceInstance { id: id.to_string() }
        }
    }

    // 模拟一个简单的 ServiceRegistry 结构体
    struct ServiceRegistry {
        services: std::collections::HashMap<String, Vec<ServiceInstance>>,
    }

    impl ServiceRegistry {
        fn new() -> Self {
            ServiceRegistry {
                services: std::collections::HashMap::new(),
            }
        }

        fn register_service(&mut self, service_name: &str, instance: ServiceInstance) {
            self.services
                .entry(service_name.to_string())
                .or_insert_with(Vec::new)
                .push(instance);
        }

        fn get_service_instances(&self, service_name: &str) -> Vec<ServiceInstance> {
            self.services
                .get(service_name)
                .cloned()
                .unwrap_or_else(Vec::new)
        }
    }

    #[test]
    fn test_get_next_instance() {
        let mut registry = ServiceRegistry::new();
        let service_name = "test_service";

        let instance1 = ServiceInstance::new("instance1");
        let instance2 = ServiceInstance::new("instance2");

        registry.register_service(service_name, instance1.clone());
        registry.register_service(service_name, instance2.clone());

        let registry = Arc::new(registry);
        let load_balancer = LoadBalancer::new(Arc::clone(&registry));

        // 验证轮询负载均衡
        assert_eq!(load_balancer.get_next_instance(service_name), Some(instance1.clone()));
        assert_eq!(load_balancer.get_next_instance(service_name), Some(instance2.clone()));
        assert_eq!(load_balancer.get_next_instance(service_name), Some(instance1.clone()));
        assert_eq!(load_balancer.get_next_instance(service_name), Some(instance2.clone()));
    }

    #[test]
    fn test_get_next_instance_no_instances() {
        let registry = ServiceRegistry::new();
        let registry = Arc::new(registry);
        let load_balancer = LoadBalancer::new(Arc::clone(&registry));

        // 验证没有可用的服务实例时返回 None
        assert!(load_balancer.get_next_instance("non_existent_service").is_none());
    }

    #[test]
    fn test_round_robin_counter_wraps_around() {
        let mut registry = ServiceRegistry::new();
        let service_name = "test_service";

        let instance1 = ServiceInstance::new("instance1");
        let instance2 = ServiceInstance::new("instance2");

        registry.register_service(service_name, instance1.clone());
        registry.register_service(service_name, instance2.clone());

        let registry = Arc::new(registry);
        let load_balancer = LoadBalancer::new(Arc::clone(&registry));

        for _ in 0..100 {
            load_balancer.get_next_instance(service_name); // 模拟多次获取实例
        }

        // 验证计数器正确回绕
        assert_eq!(load_balancer.get_next_instance(service_name), Some(instance1.clone()));
    }
}
