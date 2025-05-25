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
    pub protocol: Option<String>,
    pub local_address: Option<String>,
    pub fd: Option<String>,
    pub type_field: Option<String>,
    pub device: Option<String>,
    pub size_off: Option<String>,
    pub node: Option<String>,
    pub command_line: Option<String>,
    pub exe_path: Option<String>,
    pub start_time: Option<String>,
    pub ppid: Option<u32>,
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
    pub protocol: Option<String>,
    pub local_address: Option<String>,
    pub fd: Option<String>,
    pub type_field: Option<String>,
    pub device: Option<String>,
    pub size_off: Option<String>,
    pub node: Option<String>,
    pub command_line: Option<String>,
    pub exe_path: Option<String>,
    pub start_time: Option<String>,
    pub ppid: Option<u32>,
}
