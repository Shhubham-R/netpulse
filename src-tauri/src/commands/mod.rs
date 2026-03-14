use crate::db::{self, queries::{self, ConnectionRecord, HistoryFilter, AlertRecord, DnsQueryRecord}};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthPoint {
    pub timestamp: String,
    pub app_name: String,
    pub upload_kbps: f64,
    pub download_kbps: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub capture_interface: String,
    pub theme: String,
    pub trusted_countries: Vec<String>,
    pub alert_odd_hours: bool,
    pub alert_unknown_process: bool,
    pub alert_foreign_country: bool,
    pub data_retention_days: u32,
    pub notification_enabled: bool,
    pub start_minimized: bool,
    pub auto_start_capture: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            capture_interface: "auto".to_string(),
            theme: "dark".to_string(),
            trusted_countries: vec![
                "US".into(), "GB".into(), "DE".into(), "FR".into(),
                "CA".into(), "AU".into(), "NL".into(), "SE".into(),
                "JP".into(), "IN".into(),
            ],
            alert_odd_hours: true,
            alert_unknown_process: true,
            alert_foreign_country: true,
            data_retention_days: 30,
            notification_enabled: true,
            start_minimized: false,
            auto_start_capture: true,
        }
    }
}

#[tauri::command]
pub fn get_active_connections(db: State<'_, db::Database>) -> Result<Vec<ConnectionRecord>, String> {
    let conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;
    queries::get_active_connections(&conn)
}

#[tauri::command]
pub fn get_connection_history(
    db: State<'_, db::Database>,
    filter: HistoryFilter,
) -> Result<Vec<ConnectionRecord>, String> {
    let conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;
    queries::get_connection_history(&conn, &filter)
}

#[tauri::command]
pub fn get_bandwidth_series(
    db: State<'_, db::Database>,
    window_secs: u32,
) -> Result<Vec<BandwidthPoint>, String> {
    let conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;
    let cutoff = chrono::Utc::now()
        .checked_sub_signed(chrono::Duration::seconds(window_secs as i64))
        .unwrap_or_else(chrono::Utc::now)
        .to_rfc3339();

    let mut stmt = conn
        .prepare(
            "SELECT process_name, bytes_sent, bytes_recv, last_seen
             FROM connections
             WHERE last_seen >= ?1
             ORDER BY last_seen ASC",
        )
        .map_err(|e| format!("Query error: {e}"))?;

    let rows = stmt
        .query_map(rusqlite::params![cutoff], |row| {
            Ok(BandwidthPoint {
                app_name: row.get(0)?,
                upload_kbps: row.get::<_, i64>(1)? as f64 / 1024.0,
                download_kbps: row.get::<_, i64>(2)? as f64 / 1024.0,
                timestamp: row.get(3)?,
            })
        })
        .map_err(|e| format!("Query error: {e}"))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("Row error: {e}"))?);
    }
    Ok(results)
}

#[tauri::command]
pub fn get_alerts(
    db: State<'_, db::Database>,
    dismissed: bool,
) -> Result<Vec<AlertRecord>, String> {
    let conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;
    queries::get_alerts(&conn, dismissed)
}

#[tauri::command]
pub fn dismiss_alert(db: State<'_, db::Database>, id: i64) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;
    queries::dismiss_alert(&conn, id)
}

#[tauri::command]
pub fn block_app(db: State<'_, db::Database>, process_name: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;
    queries::block_app(&conn, &process_name)
}

#[tauri::command]
pub fn trust_app(db: State<'_, db::Database>, process_name: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;
    queries::trust_app(&conn, &process_name)
}

#[tauri::command]
pub fn get_settings(db: State<'_, db::Database>) -> Result<Settings, String> {
    let conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;
    let all = queries::get_all_settings(&conn)?;

    Ok(Settings {
        capture_interface: all
            .get("capture_interface")
            .cloned()
            .unwrap_or_else(|| "auto".to_string()),
        theme: all
            .get("theme")
            .cloned()
            .unwrap_or_else(|| "dark".to_string()),
        trusted_countries: all
            .get("trusted_countries")
            .and_then(|v| serde_json::from_str(v).ok())
            .unwrap_or_else(|| Settings::default().trusted_countries),
        alert_odd_hours: all
            .get("alert_odd_hours")
            .map(|v| v == "true")
            .unwrap_or(true),
        alert_unknown_process: all
            .get("alert_unknown_process")
            .map(|v| v == "true")
            .unwrap_or(true),
        alert_foreign_country: all
            .get("alert_foreign_country")
            .map(|v| v == "true")
            .unwrap_or(true),
        data_retention_days: all
            .get("data_retention_days")
            .and_then(|v| v.parse().ok())
            .unwrap_or(30),
        notification_enabled: all
            .get("notification_enabled")
            .map(|v| v == "true")
            .unwrap_or(true),
        start_minimized: all
            .get("start_minimized")
            .map(|v| v == "true")
            .unwrap_or(false),
        auto_start_capture: all
            .get("auto_start_capture")
            .map(|v| v == "true")
            .unwrap_or(true),
    })
}

#[tauri::command]
pub fn update_setting(
    db: State<'_, db::Database>,
    key: String,
    value: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;
    queries::set_setting(&conn, &key, &value)
}

#[tauri::command]
pub fn get_dns_queries(
    db: State<'_, db::Database>,
    pid: Option<u32>,
) -> Result<Vec<DnsQueryRecord>, String> {
    let conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;
    queries::get_dns_queries(&conn, pid)
}

#[tauri::command]
pub fn export_session(
    db: State<'_, db::Database>,
    format: String,
) -> Result<String, String> {
    let conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;

    let connections = queries::get_connection_history(
        &conn,
        &HistoryFilter {
            process_name: None,
            country_code: None,
            hostname: None,
            start_date: None,
            end_date: None,
            limit: Some(10000),
            offset: None,
        },
    )?;

    let export_dir = dirs_next::download_dir()
        .or_else(dirs_next::home_dir)
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("netpulse_export_{timestamp}");

    let file_path = match format.as_str() {
        "csv" => {
            let path = export_dir.join(format!("{filename}.csv"));
            let mut wtr = csv::Writer::from_path(&path)
                .map_err(|e| format!("Failed to create CSV file: {e}"))?;

            wtr.write_record([
                "ID", "PID", "Process", "Protocol", "SrcPort", "DstIP", "DstHost",
                "Country", "City", "BytesSent", "BytesRecv", "FirstSeen", "LastSeen",
                "Blocked", "Trusted",
            ])
            .map_err(|e| format!("CSV write error: {e}"))?;

            for c in &connections {
                wtr.write_record([
                    c.id.to_string(),
                    c.pid.map(|p| p.to_string()).unwrap_or_default(),
                    c.process_name.clone(),
                    c.protocol.clone(),
                    c.src_port.to_string(),
                    c.dst_ip.clone(),
                    c.dst_host.clone().unwrap_or_default(),
                    c.country_code.clone().unwrap_or_default(),
                    c.city.clone().unwrap_or_default(),
                    c.bytes_sent.to_string(),
                    c.bytes_recv.to_string(),
                    c.first_seen.clone(),
                    c.last_seen.clone(),
                    c.is_blocked.to_string(),
                    c.is_trusted.to_string(),
                ])
                .map_err(|e| format!("CSV write error: {e}"))?;
            }

            wtr.flush().map_err(|e| format!("CSV flush error: {e}"))?;
            path
        }
        _ => {
            // JSON format
            let path = export_dir.join(format!("{filename}.json"));
            let json = serde_json::to_string_pretty(&connections)
                .map_err(|e| format!("JSON serialization error: {e}"))?;
            std::fs::write(&path, json)
                .map_err(|e| format!("Failed to write JSON file: {e}"))?;
            path
        }
    };

    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn get_interfaces() -> Result<Vec<String>, String> {
    Ok(crate::capture::CaptureEngine::list_interfaces())
}

#[tauri::command]
pub fn set_capture_interface(name: String) -> Result<(), String> {
    // This will be handled through the app state
    log::info!("Capture interface set to: {name}");
    Ok(())
}

#[tauri::command]
pub fn get_blocked_apps(db: State<'_, db::Database>) -> Result<Vec<String>, String> {
    let conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;
    queries::get_blocked_apps(&conn)
}

#[tauri::command]
pub fn get_trusted_apps(db: State<'_, db::Database>) -> Result<Vec<String>, String> {
    let conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;
    queries::get_trusted_apps(&conn)
}
