use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ServiceStatus {
    Open,
    Closed,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Service {
    pub port: u16,
    pub status: ServiceStatus,
    pub process: Option<String>,
    pub pid: Option<u32>,
    pub user: Option<String>,
}

pub trait ServiceDiscovery: Send + Sync {
    fn discover(&self) -> Vec<ServiceInfo>;
}

#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub port: u16,
    pub process: Option<String>,
    pub pid: Option<u32>,
    pub user: Option<String>,
}
