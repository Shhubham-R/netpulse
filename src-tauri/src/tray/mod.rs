use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState},
    AppHandle, Manager,
};

pub fn setup_tray(app: &AppHandle) -> Result<(), String> {
    let show_item = MenuItemBuilder::with_id("show", "Show NetPulse")
        .build(app)
        .map_err(|e| format!("Failed to build show menu item: {e}"))?;

    let pause_item = MenuItemBuilder::with_id("pause", "Pause Capture")
        .build(app)
        .map_err(|e| format!("Failed to build pause menu item: {e}"))?;

    let export_item = MenuItemBuilder::with_id("export", "Export Session")
        .build(app)
        .map_err(|e| format!("Failed to build export menu item: {e}"))?;

    let separator = MenuItemBuilder::with_id("sep", "──────────")
        .enabled(false)
        .build(app)
        .map_err(|e| format!("Failed to build separator: {e}"))?;

    let quit_item = MenuItemBuilder::with_id("quit", "Quit")
        .build(app)
        .map_err(|e| format!("Failed to build quit menu item: {e}"))?;

    let menu = MenuBuilder::new(app)
        .items(&[&show_item, &pause_item, &export_item, &separator, &quit_item])
        .build()
        .map_err(|e| format!("Failed to build tray menu: {e}"))?;

    let _tray = TrayIconBuilder::new()
        .tooltip("NetPulse — ↑ 0 KB/s ↓ 0 KB/s")
        .menu(&menu)
        .on_menu_event(move |app, event| {
            match event.id().as_ref() {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "pause" => {
                    let _ = app.emit("tray-action", "toggle_capture");
                }
                "export" => {
                    let _ = app.emit("tray-action", "export");
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)
        .map_err(|e| format!("Failed to build tray icon: {e}"))?;

    Ok(())
}

pub fn format_speed(bytes_per_sec: f64) -> String {
    if bytes_per_sec < 1024.0 {
        format!("{:.0} B/s", bytes_per_sec)
    } else if bytes_per_sec < 1024.0 * 1024.0 {
        format!("{:.1} KB/s", bytes_per_sec / 1024.0)
    } else if bytes_per_sec < 1024.0 * 1024.0 * 1024.0 {
        format!("{:.1} MB/s", bytes_per_sec / (1024.0 * 1024.0))
    } else {
        format!(
            "{:.2} GB/s",
            bytes_per_sec / (1024.0 * 1024.0 * 1024.0)
        )
    }
}
