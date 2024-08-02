#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_get_service_instances() {
        let registry = ServiceRegistry::new();
        let service_name = "test_service";

        // 注册两个服务实例
        registry.register_service(service_name, "127.0.0.1".to_string(), 8080);
        registry.register_service(service_name, "127.0.0.2".to_string(), 8081);

        // 获取服务实例列表
        let instances = registry.get_service_instances(service_name);
        assert_eq!(instances.len(), 2);
        assert!(instances.contains(&ServiceInstance {
            host: "127.0.0.1".to_string(),
            port: 8080,
        }));
        assert!(instances.contains(&ServiceInstance {
            host: "127.0.0.2".to_string(),
            port: 8081,
        }));
    }

    #[test]
    fn test_deregister_service() {
        let registry = ServiceRegistry::new();
        let service_name = "test_service";

        // 注册两个服务实例
        registry.register_service(service_name, "127.0.0.1".to_string(), 8080);
        registry.register_service(service_name, "127.0.0.2".to_string(), 8081);

        // 注销其中一个服务实例
        registry.deregister_service(service_name, "127.0.0.1", 8080);

        // 获取剩余的服务实例
        let instances = registry.get_service_instances(service_name);
        assert_eq!(instances.len(), 1);
        assert!(instances.contains(&ServiceInstance {
            host: "127.0.0.2".to_string(),
            port: 8081,
        }));

        // 再次注销同一个实例，列表应为空
        registry.deregister_service(service_name, "127.0.0.2", 8081);
        let instances = registry.get_service_instances(service_name);
        assert!(instances.is_empty());
    }

    #[test]
    fn test_get_service_instances_empty() {
        let registry = ServiceRegistry::new();

        // 获取一个不存在的服务实例列表
        let instances = registry.get_service_instances("non_existent_service");
        assert!(instances.is_empty());
    }

    #[test]
    fn test_register_and_deregister_same_instance() {
        let registry = ServiceRegistry::new();
        let service_name = "test_service";

        // 注册一个服务实例
        registry.register_service(service_name, "127.0.0.1".to_string(), 8080);
        // 再次注册同一个服务实例
        registry.register_service(service_name, "127.0.0.1".to_string(), 8080);

        // 确认该服务实例仅存在一次
        let instances = registry.get_service_instances(service_name);
        assert_eq!(instances.len(), 2);

        // 注销该服务实例
        registry.deregister_service(service_name, "127.0.0.1", 8080);
        let instances = registry.get_service_instances(service_name);
        assert!(instances.is_empty());
    }
}
