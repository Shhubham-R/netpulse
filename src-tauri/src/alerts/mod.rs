pub mod rules;

use crate::capture::packet_parser::ConnectionEvent;
use crate::db;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: i64,
    pub connection_id: Option<i64>,
    pub rule_name: String,
    pub severity: String,
    pub message: String,
    pub process_name: Option<String>,
    pub triggered_at: String,
    pub dismissed: bool,
}

pub struct AlertEngine {
    known_processes: std::collections::HashSet<String>,
    first_seen_processes: std::collections::HashSet<String>,
    trusted_countries: Vec<String>,
}

impl AlertEngine {
    pub fn new() -> Self {
        Self {
            known_processes: Self::default_known_processes(),
            first_seen_processes: std::collections::HashSet::new(),
            trusted_countries: vec![
                "US".to_string(),
                "GB".to_string(),
                "DE".to_string(),
                "FR".to_string(),
                "CA".to_string(),
                "AU".to_string(),
                "NL".to_string(),
                "SE".to_string(),
                "JP".to_string(),
                "IN".to_string(),
            ],
        }
    }

    pub fn set_trusted_countries(&mut self, countries: Vec<String>) {
        self.trusted_countries = countries;
    }

    pub fn evaluate(
        &mut self,
        event: &ConnectionEvent,
        process_name: &str,
        country_code: &str,
        bytes_total: u64,
        is_blocked: bool,
    ) -> Vec<Alert> {
        let mut alerts = Vec::new();
        let now = chrono::Utc::now().to_rfc3339();

        // Rule 1: Odd hours (1am - 5am local time)
        if let Some(alert) = rules::check_odd_hours(process_name, &now) {
            alerts.push(alert);
        }

        // Rule 2: Unknown process
        if let Some(alert) = rules::check_unknown_process(process_name, &self.known_processes, &now)
        {
            alerts.push(alert);
        }

        // Rule 3: Foreign country
        if let Some(alert) =
            rules::check_foreign_country(process_name, country_code, &self.trusted_countries, &now)
        {
            alerts.push(alert);
        }

        // Rule 4: High volume from unknown process
        if let Some(alert) =
            rules::check_high_volume_unknown(process_name, bytes_total, &self.known_processes, &now)
        {
            alerts.push(alert);
        }

        // Rule 5: Tor exit node
        if let Some(alert) = rules::check_tor_exit_node(&event.dst_ip, process_name, &now) {
            alerts.push(alert);
        }

        // Rule 6: First connection
        if !self.first_seen_processes.contains(process_name) {
            self.first_seen_processes.insert(process_name.to_string());
            if let Some(alert) = rules::check_first_connection(process_name, &now) {
                alerts.push(alert);
            }
        }

        // Rule 7: Blocked process attempt
        if is_blocked {
            if let Some(alert) =
                rules::check_blocked_process(&event.dst_ip, process_name, &now)
            {
                alerts.push(alert);
            }
        }

        alerts
    }

    fn default_known_processes() -> std::collections::HashSet<String> {
        let procs = [
            "chrome",
            "chromium",
            "firefox",
            "firefox-esr",
            "brave",
            "opera",
            "vivaldi",
            "edge",
            "msedge",
            "safari",
            "spotify",
            "slack",
            "discord",
            "teams",
            "zoom",
            "code",
            "node",
            "npm",
            "cargo",
            "rustc",
            "git",
            "ssh",
            "sshd",
            "curl",
            "wget",
            "apt",
            "apt-get",
            "dpkg",
            "snap",
            "flatpak",
            "pip",
            "python",
            "python3",
            "java",
            "docker",
            "containerd",
            "systemd-resolve",
            "systemd-network",
            "NetworkManager",
            "avahi-daemon",
            "cups",
            "dropbox",
            "syncthing",
            "telegram",
            "signal",
            "thunderbird",
            "evolution",
            "steam",
            "lutris",
            "vscode",
            "codium",
            "postman",
            "insomnia",
            "dbeaver",
            "filezilla",
            "transmission",
            "qbittorrent",
            "vlc",
            "mpv",
            "kodi",
            "obs",
            "gimp",
            "inkscape",
            "blender",
            "libreoffice",
        ];

        procs.iter().map(|s| s.to_string()).collect()
    }
}

impl Default for AlertEngine {
    fn default() -> Self {
        Self::new()
    }
}
