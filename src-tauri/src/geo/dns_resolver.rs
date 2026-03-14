use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsEntry {
    pub query_name: String,
    pub query_type: String,
    pub resolver_ip: Option<String>,
    pub response_ips: Vec<String>,
    pub timestamp: String,
}

pub struct DnsResolver {
    /// Maps destination IP → resolved hostname
    hostname_cache: Arc<DashMap<String, String>>,
    /// Maps process_name → list of DNS queries it made
    process_dns_map: Arc<DashMap<String, Vec<DnsEntry>>>,
}

impl DnsResolver {
    pub fn new() -> Self {
        Self {
            hostname_cache: Arc::new(DashMap::new()),
            process_dns_map: Arc::new(DashMap::new()),
        }
    }

    /// Parse a DNS packet payload to extract query/response information
    pub fn parse_dns_packet(
        &self,
        payload: &[u8],
        src_ip: &str,
        dst_ip: &str,
        is_outbound: bool,
    ) -> Option<DnsEntry> {
        if payload.len() < 12 {
            return None;
        }

        // DNS Header parsing
        let flags = u16::from_be_bytes([payload[2], payload[3]]);
        let is_response = (flags & 0x8000) != 0;
        let qd_count = u16::from_be_bytes([payload[4], payload[5]]);
        let an_count = u16::from_be_bytes([payload[6], payload[7]]);

        if qd_count == 0 {
            return None;
        }

        // Parse query name
        let (query_name, mut offset) = parse_dns_name(payload, 12)?;
        if offset + 4 > payload.len() {
            return None;
        }

        let query_type_num = u16::from_be_bytes([payload[offset], payload[offset + 1]]);
        let query_type = match query_type_num {
            1 => "A",
            28 => "AAAA",
            5 => "CNAME",
            15 => "MX",
            2 => "NS",
            12 => "PTR",
            16 => "TXT",
            6 => "SOA",
            _ => "OTHER",
        }
        .to_string();

        offset += 4; // Skip QTYPE and QCLASS

        let resolver_ip = if is_outbound {
            Some(dst_ip.to_string())
        } else {
            Some(src_ip.to_string())
        };

        // Parse answer records if this is a response
        let mut response_ips = Vec::new();
        if is_response {
            for _ in 0..an_count {
                if offset >= payload.len() {
                    break;
                }

                // Skip name (might be compressed)
                if offset < payload.len() && (payload[offset] & 0xC0) == 0xC0 {
                    offset += 2;
                } else {
                    let (_, new_offset) = match parse_dns_name(payload, offset) {
                        Some(r) => r,
                        None => break,
                    };
                    offset = new_offset;
                }

                if offset + 10 > payload.len() {
                    break;
                }

                let rtype = u16::from_be_bytes([payload[offset], payload[offset + 1]]);
                let rdlength =
                    u16::from_be_bytes([payload[offset + 8], payload[offset + 9]]) as usize;
                offset += 10;

                if offset + rdlength > payload.len() {
                    break;
                }

                match rtype {
                    1 if rdlength == 4 => {
                        // A record
                        let ip = format!(
                            "{}.{}.{}.{}",
                            payload[offset],
                            payload[offset + 1],
                            payload[offset + 2],
                            payload[offset + 3]
                        );
                        response_ips.push(ip.clone());
                        // Cache the hostname
                        self.hostname_cache.insert(ip, query_name.clone());
                    }
                    28 if rdlength == 16 => {
                        // AAAA record
                        let mut segments = Vec::new();
                        for i in (0..16).step_by(2) {
                            segments.push(format!(
                                "{:04x}",
                                u16::from_be_bytes([payload[offset + i], payload[offset + i + 1]])
                            ));
                        }
                        let ip = segments.join(":");
                        response_ips.push(ip.clone());
                        self.hostname_cache.insert(ip, query_name.clone());
                    }
                    _ => {}
                }

                offset += rdlength;
            }
        }

        let timestamp = chrono::Utc::now().to_rfc3339();

        Some(DnsEntry {
            query_name,
            query_type,
            resolver_ip,
            response_ips,
            timestamp,
        })
    }

    /// Get cached hostname for an IP address
    pub fn get_hostname(&self, ip: &str) -> Option<String> {
        self.hostname_cache.get(ip).map(|v| v.clone())
    }

    /// Record a DNS query for a process
    pub fn record_for_process(&self, process_name: &str, entry: DnsEntry) {
        self.process_dns_map
            .entry(process_name.to_string())
            .or_default()
            .push(entry);
    }

    /// Get DNS queries made by a process
    pub fn get_process_queries(&self, process_name: &str) -> Vec<DnsEntry> {
        self.process_dns_map
            .get(process_name)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    /// Perform async reverse DNS lookup
    pub async fn reverse_lookup(&self, ip: &str) -> Option<String> {
        // Check cache first
        if let Some(hostname) = self.hostname_cache.get(ip) {
            return Some(hostname.clone());
        }

        let ip_addr: IpAddr = match ip.parse() {
            Ok(addr) => addr,
            Err(_) => return None,
        };

        let resolver = match hickory_resolver::TokioAsyncResolver::tokio_from_system_conf() {
            Ok(r) => r,
            Err(_) => return None,
        };

        match resolver.reverse_lookup(ip_addr).await {
            Ok(lookup) => {
                if let Some(name) = lookup.iter().next() {
                    let hostname = name.to_string().trim_end_matches('.').to_string();
                    self.hostname_cache
                        .insert(ip.to_string(), hostname.clone());
                    Some(hostname)
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }
}

/// Parse a DNS domain name from packet data, handling compression
fn parse_dns_name(data: &[u8], start: usize) -> Option<(String, usize)> {
    let mut labels = Vec::new();
    let mut offset = start;
    let mut jumped = false;
    let mut return_offset = 0;
    let mut max_jumps = 20;

    loop {
        if offset >= data.len() || max_jumps == 0 {
            return None;
        }

        let len = data[offset] as usize;

        if len == 0 {
            if !jumped {
                return_offset = offset + 1;
            }
            break;
        }

        if (len & 0xC0) == 0xC0 {
            // Compression pointer
            if offset + 1 >= data.len() {
                return None;
            }
            if !jumped {
                return_offset = offset + 2;
            }
            offset = ((len & 0x3F) << 8 | data[offset + 1] as usize) as usize;
            jumped = true;
            max_jumps -= 1;
            continue;
        }

        offset += 1;
        if offset + len > data.len() {
            return None;
        }

        let label = String::from_utf8_lossy(&data[offset..offset + len]).to_string();
        labels.push(label);
        offset += len;
    }

    if labels.is_empty() {
        return None;
    }

    let name = labels.join(".");
    Some((name, if jumped { return_offset } else { return_offset }))
}

impl Default for DnsResolver {
    fn default() -> Self {
        Self::new()
    }
}
