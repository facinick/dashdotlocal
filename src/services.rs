use crate::types::{Service, ServiceDiscovery, ServiceInfo, ServiceStatus};
use std::process::Command;
use std::str;
use tokio::net::TcpStream;
use tokio::time::{Duration, timeout};

pub struct MacServiceDiscovery;

impl ServiceDiscovery for MacServiceDiscovery {
    fn discover(&self) -> Vec<ServiceInfo> {
        let output = Command::new("lsof")
            .args(["-iTCP", "-sTCP:LISTEN", "-P", "-n"])
            .output();
        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("[DEBUG] Raw lsof output:\n{}", stdout);
            let mut infos = Vec::new();
            for line in stdout.lines().skip(1) {
                println!("[DEBUG] Parsing line: {}", line);
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() < 9 {
                    println!("[DEBUG] Skipping line (not enough columns): {}", line);
                    continue;
                }
                let process = Some(parts[0].to_string());
                let pid = parts[1].parse().ok();
                let user = Some(parts[2].to_string());
                let name = parts.get(parts.len().saturating_sub(2)).unwrap_or(&"");
                if let Some(port) = parse_port(name) {
                    println!("[DEBUG] Extracted port {} from: {}", port, name);
                    infos.push(ServiceInfo {
                        port,
                        process: process.clone(),
                        pid,
                        user: user.clone(),
                    });
                } else {
                    println!("[DEBUG] Could not extract port from: {}", name);
                }
            }
            infos
        } else {
            println!("[DEBUG] Failed to run lsof");
            Vec::new()
        }
    }
}

fn parse_port(name: &str) -> Option<u16> {
    // Example: "TCP *:3000 (LISTEN)" or "TCP 127.0.0.1:27017 (LISTEN)"
    let parts: Vec<&str> = name.split(':').collect();
    if parts.len() < 2 {
        return None;
    }
    let port_part = parts.last().unwrap();
    let port_str = port_part.split_whitespace().next()?;
    port_str.parse().ok()
}

pub async fn scan_services<D: ServiceDiscovery + ?Sized>(discovery: &D) -> Vec<Service> {
    let infos = discovery.discover();
    let checks = infos.iter().map(|info| async move {
        let addr = format!("127.0.0.1:{}", info.port);
        let status = match timeout(Duration::from_millis(200), TcpStream::connect(&addr)).await {
            Ok(Ok(_)) => ServiceStatus::Open,
            _ => ServiceStatus::Closed,
        };
        Service {
            port: info.port,
            status,
            process: info.process.clone(),
            pid: info.pid,
            user: info.user.clone(),
        }
    });
    futures::future::join_all(checks).await
}

pub async fn get_service<D: ServiceDiscovery + ?Sized>(
    discovery: &D,
    port: u16,
) -> Option<Service> {
    let infos = discovery.discover();
    let info = infos.into_iter().find(|i| i.port == port)?;
    let addr = format!("127.0.0.1:{}", port);
    let status = match timeout(Duration::from_millis(200), TcpStream::connect(&addr)).await {
        Ok(Ok(_)) => ServiceStatus::Open,
        _ => ServiceStatus::Closed,
    };
    Some(Service {
        port,
        status,
        process: info.process,
        pid: info.pid,
        user: info.user,
    })
}
