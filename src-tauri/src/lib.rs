mod alerts;
mod capture;
mod commands;
mod db;
mod geo;
mod tray;

use capture::packet_parser::ConnectionEvent;
use crossbeam_channel::bounded;
use std::sync::{Arc, Mutex};
use tauri::Manager;

pub use db::Database;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_handle = app.handle().clone();

            // Initialize database
            let data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("Failed to get app data dir: {e}"))?;
            let database =
                Database::new(data_dir.clone()).map_err(|e| format!("DB init failed: {e}"))?;
            app.manage(database);

            // Set up system tray
            if let Err(e) = tray::setup_tray(&app_handle) {
                log::warn!("Failed to setup system tray: {e}");
            }

            // Initialize GeoIP
            let geo_lookup = Arc::new(geo::GeoIpLookup::new(data_dir.clone()));
            let dns_resolver = Arc::new(geo::dns_resolver::DnsResolver::new());
            let process_mapper = Arc::new(capture::process_mapper::ProcessMapper::new());
            let alert_engine = Arc::new(Mutex::new(alerts::AlertEngine::new()));

            // Start packet capture
            let (sender, receiver) = bounded::<Vec<ConnectionEvent>>(256);
            let capture_engine = capture::CaptureEngine::new(None);

            match capture_engine.start(sender) {
                Ok(_) => log::info!("Packet capture started"),
                Err(e) => {
                    log::warn!("Packet capture failed to start: {e}");
                    log::info!("The app will run without live capture. Check permissions.");
                    // Emit a status event to tell the frontend
                    let _ = app_handle.emit("capture-status", "error");
                }
            }

            // Spawn event processing thread
            let app_handle_clone = app_handle.clone();
            let geo_clone = Arc::clone(&geo_lookup);
            let dns_clone = Arc::clone(&dns_resolver);
            let mapper_clone = Arc::clone(&process_mapper);
            let alert_clone = Arc::clone(&alert_engine);

            std::thread::Builder::new()
                .name("event-processor".to_string())
                .spawn(move || {
                    process_events(
                        receiver,
                        app_handle_clone,
                        geo_clone,
                        dns_clone,
                        mapper_clone,
                        alert_clone,
                    );
                })
                .ok();

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_active_connections,
            commands::get_connection_history,
            commands::get_bandwidth_series,
            commands::get_alerts,
            commands::dismiss_alert,
            commands::block_app,
            commands::trust_app,
            commands::get_settings,
            commands::update_setting,
            commands::get_dns_queries,
            commands::export_session,
            commands::get_interfaces,
            commands::set_capture_interface,
            commands::get_blocked_apps,
            commands::get_trusted_apps,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn process_events(
    receiver: crossbeam_channel::Receiver<Vec<ConnectionEvent>>,
    app_handle: tauri::AppHandle,
    geo_lookup: Arc<geo::GeoIpLookup>,
    dns_resolver: Arc<geo::dns_resolver::DnsResolver>,
    process_mapper: Arc<capture::process_mapper::ProcessMapper>,
    alert_engine: Arc<Mutex<alerts::AlertEngine>>,
) {
    let mut total_upload: u64 = 0;
    let mut total_download: u64 = 0;
    let mut last_tray_update = std::time::Instant::now();
    let mut last_db_flush = std::time::Instant::now();
    let mut pending_records: Vec<db::queries::ConnectionRecord> = Vec::new();

    for batch in receiver.iter() {
        let mut frontend_events: Vec<serde_json::Value> = Vec::new();

        for event in &batch {
            // Process DNS packets
            if let Some(ref dns_payload) = event.dns_payload {
                if let Some(dns_entry) = dns_resolver.parse_dns_packet(
                    dns_payload,
                    &event.src_ip,
                    &event.dst_ip,
                    event.is_outbound,
                ) {
                    // Store DNS query for the frontend
                    let _ = app_handle.emit("dns-event", &dns_entry);
                }
            }

            // Map to process
            let (local_ip, local_port) = if event.is_outbound {
                (&event.src_ip, event.src_port)
            } else {
                (&event.dst_ip, event.dst_port)
            };

            let process_info = process_mapper.lookup(local_ip, local_port);
            let process_name = process_info
                .as_ref()
                .map(|p| p.name.clone())
                .unwrap_or_else(|| "unknown".to_string());
            let pid = process_info.as_ref().map(|p| p.pid);

            // GeoIP lookup for remote IP
            let remote_ip = if event.is_outbound {
                &event.dst_ip
            } else {
                &event.src_ip
            };

            let geo_result = if let Ok(ip) = remote_ip.parse() {
                geo_lookup.lookup(ip)
            } else {
                geo::GeoResult::unknown()
            };

            // Get hostname
            let hostname = dns_resolver.get_hostname(remote_ip);

            // Track bandwidth
            if event.is_outbound {
                total_upload += event.payload_len as u64;
            } else {
                total_download += event.payload_len as u64;
            }

            // Check alerts
            let bytes_total = event.payload_len as u64;
            let is_blocked = false; // Would check DB in production

            if let Ok(mut engine) = alert_engine.lock() {
                let alerts = engine.evaluate(
                    event,
                    &process_name,
                    &geo_result.country_code,
                    bytes_total,
                    is_blocked,
                );

                for alert in alerts {
                    let _ = app_handle.emit("alert-triggered", &alert);
                }
            }

            // Build frontend event
            let fe_event = serde_json::json!({
                "src_ip": event.src_ip,
                "src_port": event.src_port,
                "dst_ip": event.dst_ip,
                "dst_port": event.dst_port,
                "protocol": event.protocol,
                "payload_len": event.payload_len,
                "timestamp": event.timestamp,
                "is_outbound": event.is_outbound,
                "process_name": process_name,
                "pid": pid,
                "country_code": geo_result.country_code,
                "country_name": geo_result.country_name,
                "city": geo_result.city,
                "latitude": geo_result.latitude,
                "longitude": geo_result.longitude,
                "hostname": hostname,
            });

            frontend_events.push(fe_event);

            // Build DB record
            let now = chrono::Utc::now().to_rfc3339();
            let (bytes_sent, bytes_recv) = if event.is_outbound {
                (event.payload_len as u64, 0u64)
            } else {
                (0u64, event.payload_len as u64)
            };

            pending_records.push(db::queries::ConnectionRecord {
                id: 0,
                pid,
                process_name: process_name.clone(),
                protocol: event.protocol.clone(),
                src_port: if event.is_outbound {
                    event.src_port
                } else {
                    event.dst_port
                },
                dst_ip: remote_ip.to_string(),
                dst_host: hostname.clone(),
                country_code: Some(geo_result.country_code.clone()),
                city: Some(geo_result.city.clone()),
                bytes_sent,
                bytes_recv,
                first_seen: now.clone(),
                last_seen: now,
                is_blocked: false,
                is_trusted: false,
            });
        }

        // Emit batch to frontend
        if !frontend_events.is_empty() {
            let _ = app_handle.emit("traffic-event", &frontend_events);
        }

        // Emit speed stats
        let speed_event = serde_json::json!({
            "total_upload": total_upload,
            "total_download": total_download,
            "upload_speed": total_upload as f64 / last_tray_update.elapsed().as_secs_f64().max(1.0),
            "download_speed": total_download as f64 / last_tray_update.elapsed().as_secs_f64().max(1.0),
        });
        let _ = app_handle.emit("speed-update", &speed_event);

        // Update tray tooltip periodically
        if last_tray_update.elapsed() >= std::time::Duration::from_secs(2) {
            total_upload = 0;
            total_download = 0;
            last_tray_update = std::time::Instant::now();
        }

        // Flush to DB every 5 seconds
        if last_db_flush.elapsed() >= std::time::Duration::from_secs(5) && !pending_records.is_empty() {
            if let Some(db_state) = app_handle.try_state::<Database>() {
                if let Ok(conn) = db_state.conn.lock() {
                    for record in pending_records.drain(..) {
                        if let Err(e) = db::queries::upsert_connection(&conn, &record) {
                            log::warn!("Failed to flush connection to DB: {e}");
                        }
                    }
                }
            }
            last_db_flush = std::time::Instant::now();
        }
    }
}
