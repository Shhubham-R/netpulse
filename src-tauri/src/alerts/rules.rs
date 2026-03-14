use super::Alert;
use std::collections::HashSet;

/// Known Tor exit node IPs (small static sample — in production, use a downloaded list)
const TOR_EXIT_NODES: &[&str] = &[
    "185.220.101.1",
    "185.220.101.2",
    "185.220.101.3",
    "185.220.101.4",
    "185.220.101.33",
    "185.220.101.34",
    "185.220.101.35",
    "185.220.101.36",
    "185.220.102.240",
    "185.220.102.241",
    "185.220.102.242",
    "185.220.102.243",
    "185.220.102.244",
    "185.220.102.245",
    "192.42.116.16",
    "192.42.116.17",
    "204.85.191.30",
    "204.85.191.31",
    "176.10.99.200",
    "176.10.104.240",
    "109.70.100.1",
    "109.70.100.2",
    "109.70.100.3",
    "109.70.100.4",
    "109.70.100.5",
    "109.70.100.6",
];

/// Check if connection is during odd hours (1am - 5am local time)
pub fn check_odd_hours(process_name: &str, now: &str) -> Option<Alert> {
    let hour = chrono::Local::now().hour();
    if hour >= 1 && hour < 5 {
        Some(Alert {
            id: 0,
            connection_id: None,
            rule_name: "odd_hours".to_string(),
            severity: "warning".to_string(),
            message: format!(
                "Process '{}' is making network connections at {}:00 (odd hours: 1am-5am)",
                process_name, hour
            ),
            process_name: Some(process_name.to_string()),
            triggered_at: now.to_string(),
            dismissed: false,
        })
    } else {
        None
    }
}

use chrono::Timelike;

/// Check if process is unknown (not in common app list)
pub fn check_unknown_process(
    process_name: &str,
    known: &HashSet<String>,
    now: &str,
) -> Option<Alert> {
    if process_name == "unknown" || process_name.is_empty() {
        return Some(Alert {
            id: 0,
            connection_id: None,
            rule_name: "unknown_process".to_string(),
            severity: "warning".to_string(),
            message: "Unidentified process making network connections".to_string(),
            process_name: Some(process_name.to_string()),
            triggered_at: now.to_string(),
            dismissed: false,
        });
    }

    let name_lower = process_name.to_lowercase();
    if !known.contains(&name_lower) {
        Some(Alert {
            id: 0,
            connection_id: None,
            rule_name: "unknown_process".to_string(),
            severity: "info".to_string(),
            message: format!(
                "Uncommon process '{}' detected making network connections",
                process_name
            ),
            process_name: Some(process_name.to_string()),
            triggered_at: now.to_string(),
            dismissed: false,
        })
    } else {
        None
    }
}

/// Check if destination country is not in trusted list
pub fn check_foreign_country(
    process_name: &str,
    country_code: &str,
    trusted: &[String],
    now: &str,
) -> Option<Alert> {
    if country_code.is_empty() || country_code == "??" {
        return None;
    }

    if !trusted.iter().any(|c| c == country_code) {
        Some(Alert {
            id: 0,
            connection_id: None,
            rule_name: "foreign_country".to_string(),
            severity: "info".to_string(),
            message: format!(
                "Process '{}' connected to a server in {} (not in trusted country list)",
                process_name, country_code
            ),
            process_name: Some(process_name.to_string()),
            triggered_at: now.to_string(),
            dismissed: false,
        })
    } else {
        None
    }
}

/// Check for high data volume from unknown process
pub fn check_high_volume_unknown(
    process_name: &str,
    bytes_total: u64,
    known: &HashSet<String>,
    now: &str,
) -> Option<Alert> {
    let threshold = 10 * 1024 * 1024; // 10 MB
    let name_lower = process_name.to_lowercase();

    if bytes_total > threshold && !known.contains(&name_lower) {
        Some(Alert {
            id: 0,
            connection_id: None,
            rule_name: "high_volume_unknown".to_string(),
            severity: "danger".to_string(),
            message: format!(
                "Unknown process '{}' has transferred {:.1} MB of data",
                process_name,
                bytes_total as f64 / (1024.0 * 1024.0)
            ),
            process_name: Some(process_name.to_string()),
            triggered_at: now.to_string(),
            dismissed: false,
        })
    } else {
        None
    }
}

/// Check if destination is a known Tor exit node
pub fn check_tor_exit_node(dst_ip: &str, process_name: &str, now: &str) -> Option<Alert> {
    if TOR_EXIT_NODES.contains(&dst_ip) {
        Some(Alert {
            id: 0,
            connection_id: None,
            rule_name: "tor_exit_node".to_string(),
            severity: "danger".to_string(),
            message: format!(
                "Process '{}' connected to known Tor exit node {}",
                process_name, dst_ip
            ),
            process_name: Some(process_name.to_string()),
            triggered_at: now.to_string(),
            dismissed: false,
        })
    } else {
        None
    }
}

/// Check if this is the first time a process has made an outbound connection
pub fn check_first_connection(process_name: &str, now: &str) -> Option<Alert> {
    if process_name == "unknown" || process_name.is_empty() {
        return None;
    }

    Some(Alert {
        id: 0,
        connection_id: None,
        rule_name: "first_connection".to_string(),
        severity: "info".to_string(),
        message: format!(
            "First outbound connection detected from '{}'",
            process_name
        ),
        process_name: Some(process_name.to_string()),
        triggered_at: now.to_string(),
        dismissed: false,
    })
}

/// Check if a blocked process is attempting to connect
pub fn check_blocked_process(dst_ip: &str, process_name: &str, now: &str) -> Option<Alert> {
    Some(Alert {
        id: 0,
        connection_id: None,
        rule_name: "blocked_process_attempt".to_string(),
        severity: "danger".to_string(),
        message: format!(
            "Blocked process '{}' attempted connection to {}",
            process_name, dst_ip
        ),
        process_name: Some(process_name.to_string()),
        triggered_at: now.to_string(),
        dismissed: false,
    })
}
