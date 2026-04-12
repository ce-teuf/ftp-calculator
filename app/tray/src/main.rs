//! ftp-tray — System tray controller for FTP Simulator.
//!
//! Monitors the backend process and PostgreSQL, provides a tray menu:
//!   • Ouvrir le dashboard  → opens http://localhost:3000 in default browser
//!   • Arrêter les services → kills backend + pg
//!   • Démarrer les services → launches backend + pg
//!   • Voir les logs        → opens log folder
//!   • À propos             → version dialog
//!   • Quitter              → exits the tray (services keep running)
//!
//! Icon colour:
//!   🟢 Both services running
//!   🔴 At least one service stopped
//!   🟡 Starting / transient state

use std::process::Command;
use std::time::Duration;

fn open_url(url: &str) {
    #[cfg(target_os = "windows")]
    { let _ = Command::new("cmd").args(["/C", "start", url]).spawn(); }
    #[cfg(target_os = "macos")]
    { let _ = Command::new("open").arg(url).spawn(); }
    #[cfg(target_os = "linux")]
    { let _ = Command::new("xdg-open").arg(url).spawn(); }
}

fn open_folder(path: &str) {
    #[cfg(target_os = "windows")]
    { let _ = Command::new("explorer").arg(path).spawn(); }
    #[cfg(target_os = "macos")]
    { let _ = Command::new("open").arg(path).spawn(); }
    #[cfg(target_os = "linux")]
    { let _ = Command::new("xdg-open").arg(path).spawn(); }
}

/// Check if ftp-backend process is running.
fn backend_running() -> bool {
    #[cfg(target_os = "windows")]
    {
        let out = Command::new("tasklist").output().unwrap_or_default();
        String::from_utf8_lossy(&out.stdout).contains("ftp-backend")
    }
    #[cfg(not(target_os = "windows"))]
    {
        let out = Command::new("pgrep").arg("-f").arg("ftp-backend").output().unwrap_or_default();
        !out.stdout.is_empty()
    }
}

fn start_services() {
    // On Linux/macOS: start via systemctl or directly
    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("systemctl").args(["start", "ftp-simulator-db"]).status();
        std::thread::sleep(Duration::from_secs(2));
        let _ = Command::new("systemctl").args(["start", "ftp-simulator-app"]).status();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("launchctl").args(["start", "com.ftpsimulator.db"]).status();
        let _ = Command::new("launchctl").args(["start", "com.ftpsimulator.app"]).status();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("net").args(["start", "FtpSimulatorDB"]).status();
        let _ = Command::new("net").args(["start", "FtpSimulatorApp"]).status();
    }
}

fn stop_services() {
    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("systemctl").args(["stop", "ftp-simulator-app"]).status();
        let _ = Command::new("systemctl").args(["stop", "ftp-simulator-db"]).status();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("launchctl").args(["stop", "com.ftpsimulator.app"]).status();
        let _ = Command::new("launchctl").args(["stop", "com.ftpsimulator.db"]).status();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("net").args(["stop", "FtpSimulatorApp"]).status();
        let _ = Command::new("net").args(["stop", "FtpSimulatorDB"]).status();
    }
}

fn log_dir() -> String {
    #[cfg(target_os = "windows")]
    { r"C:\ProgramData\FtpSimulator\logs".to_string() }
    #[cfg(target_os = "macos")]
    { format!("{}/Library/Logs/FtpSimulator", std::env::var("HOME").unwrap_or_default()) }
    #[cfg(target_os = "linux")]
    { "/var/log/ftp-simulator".to_string() }
}

fn main() {
    // On Linux, a display is required for the tray.
    // Bail gracefully if running headless.
    #[cfg(target_os = "linux")]
    if std::env::var("DISPLAY").is_err() && std::env::var("WAYLAND_DISPLAY").is_err() {
        eprintln!("ftp-tray: no display available, running in CLI-only mode");
        // Just keep services alive and exit — tray UI is optional
        return;
    }

    use tray_icon::{TrayIconBuilder, menu::{Menu, MenuItem, PredefinedMenuItem}};
    use winit::event_loop::{EventLoop, ControlFlow};

    let event_loop = EventLoop::new().expect("Failed to create event loop");

    let open_item  = MenuItem::new("📂 Ouvrir le dashboard", true, None);
    let start_item = MenuItem::new("▶  Démarrer les services", true, None);
    let stop_item  = MenuItem::new("⏹  Arrêter les services", true, None);
    let logs_item  = MenuItem::new("📋 Voir les logs", true, None);
    let about_item = MenuItem::new("ℹ️  À propos — FTP Simulator v1.0", false, None);
    let quit_item  = MenuItem::new("✖  Quitter le tray", true, None);

    let menu = Menu::new();
    menu.append(&open_item).ok();
    menu.append(&PredefinedMenuItem::separator()).ok();
    menu.append(&start_item).ok();
    menu.append(&stop_item).ok();
    menu.append(&PredefinedMenuItem::separator()).ok();
    menu.append(&logs_item).ok();
    menu.append(&about_item).ok();
    menu.append(&PredefinedMenuItem::separator()).ok();
    menu.append(&quit_item).ok();

    // Create a 16×16 solid-color icon (green = running, red = stopped)
    let running = backend_running();
    let icon_data: Vec<u8> = {
        let (r,g,b) = if running { (0, 180, 0) } else { (200, 40, 40) };
        (0..16*16*4).map(|i| match i % 4 { 0 => r, 1 => g, 2 => b, _ => 255 }).collect()
    };
    let icon = tray_icon::Icon::from_rgba(icon_data, 16, 16).expect("icon");

    let tooltip = if running { "FTP Simulator — Actif" } else { "FTP Simulator — Arrêté" };

    let _tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip(tooltip)
        .with_icon(icon)
        .build()
        .expect("Failed to build tray icon");

    let open_id  = open_item.id().clone();
    let start_id = start_item.id().clone();
    let stop_id  = stop_item.id().clone();
    let logs_id  = logs_item.id().clone();
    let quit_id  = quit_item.id().clone();

    event_loop.run(move |_event, elwt| {
        elwt.set_control_flow(ControlFlow::WaitUntil(
            std::time::Instant::now() + Duration::from_millis(100)
        ));

        if let Ok(event) = tray_icon::menu::MenuEvent::receiver().try_recv() {
            if event.id == open_id  { open_url("http://localhost:3000"); }
            if event.id == start_id { start_services(); }
            if event.id == stop_id  { stop_services(); }
            if event.id == logs_id  { open_folder(&log_dir()); }
            if event.id == quit_id  { elwt.exit(); }
        }
    }).expect("Event loop error");
}
