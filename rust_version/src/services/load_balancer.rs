use super::service_registry::{ServiceInstance, ServiceRegistry};
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

pub struct LoadBalancer {
    registry: Arc<ServiceRegistry>,
    round_robin_counter: AtomicUsize,
}

impl LoadBalancer {
    pub fn new(registry: Arc<ServiceRegistry>) -> Self {
        LoadBalancer {
            registry,
            round_robin_counter: AtomicUsize::new(0),
        }
    }

    pub fn get_next_instance(&self, service_name: &str) -> Option<ServiceInstance> {
        let instances = self.registry.get_service_instances(service_name);
        if instances.is_empty() {
            None
        } else {
            let index = self.round_robin_counter.fetch_add(1, Ordering::Relaxed) % instances.len();
            Some(instances[index].clone())
        }
    }
}