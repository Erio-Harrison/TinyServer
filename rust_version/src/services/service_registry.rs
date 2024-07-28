use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct ServiceInstance {
    pub host: String,
    pub port: u16,
    // 可以添加其他元数据，如健康状态、负载等
}

pub struct ServiceRegistry {
    services: Arc<Mutex<HashMap<String, Vec<ServiceInstance>>>>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        ServiceRegistry {
            services: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn register_service(&self, service_name: &str, host: String, port: u16) {
        let mut services = self.services.lock().unwrap();
        let instances = services.entry(service_name.to_string()).or_insert_with(Vec::new);
        instances.push(ServiceInstance { host, port });
    }

    pub fn deregister_service(&self, service_name: &str, host: &str, port: u16) {
        let mut services = self.services.lock().unwrap();
        if let Some(instances) = services.get_mut(service_name) {
            instances.retain(|instance| !(instance.host == host && instance.port == port));
        }
    }

    pub fn get_service_instances(&self, service_name: &str) -> Vec<ServiceInstance> {
        let services = self.services.lock().unwrap();
        services.get(service_name).cloned().unwrap_or_default()
    }
}