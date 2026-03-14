use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
}

struct CacheEntry {
    info: Option<ProcessInfo>,
    inserted_at: Instant,
}

pub struct ProcessMapper {
    cache: Arc<DashMap<(String, u16), CacheEntry>>,
    cache_ttl: std::time::Duration,
}

impl ProcessMapper {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            cache_ttl: std::time::Duration::from_secs(2),
        }
    }

    pub fn lookup(&self, ip: &str, port: u16) -> Option<ProcessInfo> {
        let key = (ip.to_string(), port);

        // Check cache first
        if let Some(entry) = self.cache.get(&key) {
            if entry.inserted_at.elapsed() < self.cache_ttl {
                return entry.info.clone();
            }
        }

        // Perform lookup
        let result = self.do_lookup(ip, port);

        // Cache result
        self.cache.insert(
            key,
            CacheEntry {
                info: result.clone(),
                inserted_at: Instant::now(),
            },
        );

        // Periodic cache cleanup (every ~100 lookups)
        if self.cache.len() > 5000 {
            self.cleanup_cache();
        }

        result
    }

    fn cleanup_cache(&self) {
        self.cache.retain(|_, v| v.inserted_at.elapsed() < self.cache_ttl * 5);
    }

    #[cfg(target_os = "linux")]
    fn do_lookup(&self, ip: &str, port: u16) -> Option<ProcessInfo> {
        // Build socket → inode map from /proc/net/tcp and /proc/net/udp
        let socket_inodes = self.get_socket_inodes(ip, port);

        if socket_inodes.is_empty() {
            return None;
        }

        // Scan /proc/[pid]/fd to find which PID owns the inode
        self.find_pid_for_inodes(&socket_inodes)
    }

    #[cfg(target_os = "windows")]
    fn do_lookup(&self, _ip: &str, _port: u16) -> Option<ProcessInfo> {
        // Windows implementation would use GetExtendedTcpTable / GetExtendedUdpTable
        // This requires the `windows` crate with Win32_NetworkManagement_IpHelper
        None
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    fn do_lookup(&self, _ip: &str, _port: u16) -> Option<ProcessInfo> {
        None
    }

    #[cfg(target_os = "linux")]
    fn get_socket_inodes(&self, ip: &str, port: u16) -> Vec<u64> {
        let mut inodes = Vec::new();
        let port_hex = format!("{:04X}", port);

        for path in &[
            "/proc/net/tcp",
            "/proc/net/tcp6",
            "/proc/net/udp",
            "/proc/net/udp6",
        ] {
            if let Ok(content) = std::fs::read_to_string(path) {
                for line in content.lines().skip(1) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() < 10 {
                        continue;
                    }

                    // local_address is in format HEXIP:HEXPORT
                    let local_addr = parts[1];
                    if let Some(lport) = local_addr.split(':').nth(1) {
                        if lport == port_hex {
                            if let Ok(inode) = parts[9].parse::<u64>() {
                                if inode != 0 {
                                    inodes.push(inode);
                                }
                            }
                        }
                    }

                    // Also check remote address
                    let remote_addr = parts[2];
                    if let Some(rport) = remote_addr.split(':').nth(1) {
                        if rport == port_hex {
                            // Check if IP matches
                            if let Some(rip_hex) = remote_addr.split(':').next() {
                                if let Some(parsed_ip) = hex_to_ip(rip_hex) {
                                    if parsed_ip == ip {
                                        if let Ok(inode) = parts[9].parse::<u64>() {
                                            if inode != 0 {
                                                inodes.push(inode);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        inodes.dedup();
        inodes
    }

    #[cfg(target_os = "linux")]
    fn find_pid_for_inodes(&self, target_inodes: &[u64]) -> Option<ProcessInfo> {
        let proc_dir = match std::fs::read_dir("/proc") {
            Ok(dir) => dir,
            Err(_) => return None,
        };

        for entry in proc_dir.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            // Only look at numeric directories (PIDs)
            let pid: u32 = match name_str.parse() {
                Ok(p) => p,
                Err(_) => continue,
            };

            let fd_dir = format!("/proc/{pid}/fd");
            let fd_entries = match std::fs::read_dir(&fd_dir) {
                Ok(dir) => dir,
                Err(_) => continue,
            };

            for fd_entry in fd_entries.flatten() {
                let link = match std::fs::read_link(fd_entry.path()) {
                    Ok(l) => l,
                    Err(_) => continue,
                };

                let link_str = link.to_string_lossy();
                if link_str.starts_with("socket:[") {
                    let inode_str = &link_str[8..link_str.len() - 1];
                    if let Ok(inode) = inode_str.parse::<u64>() {
                        if target_inodes.contains(&inode) {
                            let comm_path = format!("/proc/{pid}/comm");
                            let name = std::fs::read_to_string(&comm_path)
                                .unwrap_or_else(|_| "unknown".to_string())
                                .trim()
                                .to_string();

                            return Some(ProcessInfo { pid, name });
                        }
                    }
                }
            }
        }

        None
    }
}

#[cfg(target_os = "linux")]
fn hex_to_ip(hex: &str) -> Option<String> {
    if hex.len() == 8 {
        // IPv4 in little-endian hex
        let bytes = u32::from_str_radix(hex, 16).ok()?;
        let ip = std::net::Ipv4Addr::from(bytes.to_be());
        // /proc/net/tcp stores addresses in host byte order (little-endian on x86)
        let octets = [
            (bytes & 0xFF) as u8,
            ((bytes >> 8) & 0xFF) as u8,
            ((bytes >> 16) & 0xFF) as u8,
            ((bytes >> 24) & 0xFF) as u8,
        ];
        Some(format!("{}.{}.{}.{}", octets[0], octets[1], octets[2], octets[3]))
    } else if hex.len() == 32 {
        // IPv6
        let mut segments = Vec::new();
        for i in (0..32).step_by(8) {
            let chunk = &hex[i..i + 8];
            let val = u32::from_str_radix(chunk, 16).ok()?;
            segments.push(format!("{:04x}", (val >> 16) & 0xFFFF));
            segments.push(format!("{:04x}", val & 0xFFFF));
        }
        Some(segments.join(":"))
    } else {
        None
    }
}

impl Default for ProcessMapper {
    fn default() -> Self {
        Self::new()
    }
}
