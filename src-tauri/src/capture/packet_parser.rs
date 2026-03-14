use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionEvent {
    pub src_ip: String,
    pub src_port: u16,
    pub dst_ip: String,
    pub dst_port: u16,
    pub protocol: String,
    pub payload_len: u32,
    pub timestamp: String,
    pub is_outbound: bool,
    pub tcp_flags: Option<TcpFlags>,
    pub dns_payload: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpFlags {
    pub syn: bool,
    pub ack: bool,
    pub fin: bool,
    pub rst: bool,
}

pub fn parse_packet(data: &[u8], ts: libc::timeval) -> Option<ConnectionEvent> {
    let timestamp = {
        let secs = ts.tv_sec as i64;
        let nsecs = (ts.tv_usec as u32) * 1000;
        chrono::DateTime::from_timestamp(secs, nsecs)
            .unwrap_or_else(|| chrono::Utc::now().into())
            .to_rfc3339()
    };

    // Try parsing as Ethernet first
    if let Some(eth) = EthernetPacket::new(data) {
        match eth.get_ethertype() {
            EtherTypes::Ipv4 => {
                return parse_ipv4(eth.payload(), &timestamp);
            }
            EtherTypes::Ipv6 => {
                return parse_ipv6(eth.payload(), &timestamp);
            }
            _ => {}
        }
    }

    // Try raw IP (e.g., on "any" interface or loopback with SLL header)
    if data.len() > 16 {
        // Linux cooked capture (SLL) header is 16 bytes
        let protocol_type = u16::from_be_bytes([data[14], data[15]]);
        if protocol_type == 0x0800 {
            return parse_ipv4(&data[16..], &timestamp);
        } else if protocol_type == 0x86DD {
            return parse_ipv6(&data[16..], &timestamp);
        }
    }

    // Try direct IPv4 parse
    if !data.is_empty() && (data[0] >> 4) == 4 {
        return parse_ipv4(data, &timestamp);
    }

    None
}

fn parse_ipv4(data: &[u8], timestamp: &str) -> Option<ConnectionEvent> {
    let ipv4 = Ipv4Packet::new(data)?;
    let src_ip = IpAddr::V4(ipv4.get_source());
    let dst_ip = IpAddr::V4(ipv4.get_destination());
    let payload = ipv4.payload();
    let total_len = ipv4.get_total_length() as u32;

    let is_outbound = is_local_ip(&src_ip);

    match ipv4.get_next_level_protocol() {
        IpNextHeaderProtocols::Tcp => {
            let tcp = TcpPacket::new(payload)?;
            let flags = TcpFlags {
                syn: tcp.get_flags() & 0x02 != 0,
                ack: tcp.get_flags() & 0x10 != 0,
                fin: tcp.get_flags() & 0x01 != 0,
                rst: tcp.get_flags() & 0x04 != 0,
            };

            Some(ConnectionEvent {
                src_ip: src_ip.to_string(),
                src_port: tcp.get_source(),
                dst_ip: dst_ip.to_string(),
                dst_port: tcp.get_destination(),
                protocol: "TCP".to_string(),
                payload_len: total_len.saturating_sub(
                    (ipv4.get_header_length() as u32 * 4) + (tcp.get_data_offset() as u32 * 4),
                ),
                timestamp: timestamp.to_string(),
                is_outbound,
                tcp_flags: Some(flags),
                dns_payload: None,
            })
        }
        IpNextHeaderProtocols::Udp => {
            let udp = UdpPacket::new(payload)?;
            let dns_payload = if udp.get_source() == 53 || udp.get_destination() == 53 {
                Some(udp.payload().to_vec())
            } else {
                None
            };

            Some(ConnectionEvent {
                src_ip: src_ip.to_string(),
                src_port: udp.get_source(),
                dst_ip: dst_ip.to_string(),
                dst_port: udp.get_destination(),
                protocol: "UDP".to_string(),
                payload_len: udp.get_length() as u32,
                timestamp: timestamp.to_string(),
                is_outbound,
                tcp_flags: None,
                dns_payload,
            })
        }
        _ => None,
    }
}

fn parse_ipv6(data: &[u8], timestamp: &str) -> Option<ConnectionEvent> {
    let ipv6 = Ipv6Packet::new(data)?;
    let src_ip = IpAddr::V6(ipv6.get_source());
    let dst_ip = IpAddr::V6(ipv6.get_destination());
    let payload = ipv6.payload();
    let payload_len = ipv6.get_payload_length() as u32;

    let is_outbound = is_local_ip(&src_ip);

    match ipv6.get_next_header() {
        IpNextHeaderProtocols::Tcp => {
            let tcp = TcpPacket::new(payload)?;
            let flags = TcpFlags {
                syn: tcp.get_flags() & 0x02 != 0,
                ack: tcp.get_flags() & 0x10 != 0,
                fin: tcp.get_flags() & 0x01 != 0,
                rst: tcp.get_flags() & 0x04 != 0,
            };

            Some(ConnectionEvent {
                src_ip: src_ip.to_string(),
                src_port: tcp.get_source(),
                dst_ip: dst_ip.to_string(),
                dst_port: tcp.get_destination(),
                protocol: "TCP".to_string(),
                payload_len,
                timestamp: timestamp.to_string(),
                is_outbound,
                tcp_flags: Some(flags),
                dns_payload: None,
            })
        }
        IpNextHeaderProtocols::Udp => {
            let udp = UdpPacket::new(payload)?;
            let dns_payload = if udp.get_source() == 53 || udp.get_destination() == 53 {
                Some(udp.payload().to_vec())
            } else {
                None
            };

            Some(ConnectionEvent {
                src_ip: src_ip.to_string(),
                src_port: udp.get_source(),
                dst_ip: dst_ip.to_string(),
                dst_port: udp.get_destination(),
                protocol: "UDP".to_string(),
                payload_len,
                timestamp: timestamp.to_string(),
                is_outbound,
                tcp_flags: None,
                dns_payload,
            })
        }
        _ => None,
    }
}

fn is_local_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_private()
                || v4.is_loopback()
                || v4.is_link_local()
                || v4.octets()[0] == 100 && v4.octets()[1] >= 64 && v4.octets()[1] <= 127
        }
        IpAddr::V6(v6) => v6.is_loopback() || v6.segments()[0] == 0xfe80,
    }
}
