mod service_registry;
mod load_balancer;

pub use service_registry::{ServiceInstance, ServiceRegistry};
pub use load_balancer::LoadBalancer;