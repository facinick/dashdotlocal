use crate::types::{Service, ServiceDiscovery, ServiceInfo, ServiceStatus};
use std::collections::HashSet;
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
                let fd = Some(parts[3].to_string());
                let type_field = Some(parts[4].to_string());
                let device = Some(parts[5].to_string());
                let size_off = Some(parts[6].to_string());
                let node = Some(parts[7].to_string());
                let name = parts.get(parts.len().saturating_sub(2)).unwrap_or(&"");
                let (protocol, local_address, port) = parse_protocol_address_port(name);
                let (command_line, exe_path, start_time, ppid) = if let Some(pid) = pid {
                    get_process_details(pid)
                } else {
                    (None, None, None, None)
                };
                if let Some(port) = port {
                    println!("[DEBUG] Extracted port {} from: {}", port, name);
                    infos.push(ServiceInfo {
                        port,
                        process: process.clone(),
                        pid,
                        user: user.clone(),
                        protocol,
                        local_address,
                        fd,
                        type_field,
                        device,
                        size_off,
                        node,
                        command_line,
                        exe_path,
                        start_time,
                        ppid,
                    });
                } else {
                    println!("[DEBUG] Could not extract port from: {}", name);
                }
            }
            // Deduplicate by (process, pid, port)
            let mut seen = HashSet::new();
            infos.retain(|info| {
                let key = (info.process.clone(), info.pid, info.port);
                seen.insert(key)
            });
            infos
        } else {
            println!("[DEBUG] Failed to run lsof");
            Vec::new()
        }
    }
}

fn parse_protocol_address_port(name: &str) -> (Option<String>, Option<String>, Option<u16>) {
    // Example: "TCP *:3000 (LISTEN)" or "TCP 127.0.0.1:27017 (LISTEN)"
    let mut protocol = None;
    let mut local_address = None;
    let mut port = None;
    let mut parts = name.split_whitespace();
    if let Some(proto_addr) = parts.next() {
        let mut proto_addr_parts = proto_addr.splitn(2, ' ');
        let proto_addr = proto_addr_parts.next().unwrap_or("");
        let mut proto_and_addr = proto_addr.splitn(2, '/');
        let proto = proto_and_addr.next().unwrap_or("");
        protocol = Some(proto.to_string());
        let addr = proto_and_addr.next().unwrap_or("");
        let addr = if addr.is_empty() { proto_addr } else { addr };
        let addr_parts: Vec<&str> = addr.split(':').collect();
        if addr_parts.len() >= 2 {
            local_address = Some(addr_parts[..addr_parts.len() - 1].join(":"));
            port = addr_parts.last().and_then(|p| p.parse().ok());
        }
    }
    (protocol, local_address, port)
}

fn get_process_details(pid: u32) -> (Option<String>, Option<String>, Option<String>, Option<u32>) {
    // Get full command line
    let command_line = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "command="])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string());
    // Get executable path
    let exe_path = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "comm="])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string());
    // Get start time
    let start_time = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "lstart="])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string());
    // Get parent PID
    let ppid = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "ppid="])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse().ok());
    (command_line, exe_path, start_time, ppid)
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
            protocol: info.protocol.clone(),
            local_address: info.local_address.clone(),
            fd: info.fd.clone(),
            type_field: info.type_field.clone(),
            device: info.device.clone(),
            size_off: info.size_off.clone(),
            node: info.node.clone(),
            command_line: info.command_line.clone(),
            exe_path: info.exe_path.clone(),
            start_time: info.start_time.clone(),
            ppid: info.ppid,
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
        protocol: info.protocol,
        local_address: info.local_address,
        fd: info.fd,
        type_field: info.type_field,
        device: info.device,
        size_off: info.size_off,
        node: info.node,
        command_line: info.command_line,
        exe_path: info.exe_path,
        start_time: info.start_time,
        ppid: info.ppid,
    })
}
