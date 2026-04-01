mod commands;
mod config;
mod models;
mod providers;
mod tray;

use commands::AppState;
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志（默认 info 级别，可通过 RUST_LOG 环境变量覆盖）
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info"),
    )
    .init();

    let cfg = config::load_config();

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(AppState {
            config: Mutex::new(cfg),
        })
        .setup(|app| {
            tray::setup_tray(app.handle())?;
            if let Some(window) = app.get_webview_window("main") {
                window.hide().ok();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::save_config,
            commands::fetch_all_usage,
            commands::set_autostart,
            commands::refresh_single,
            commands::set_zhipu_usage_days,
        ])
        .run(tauri::generate_context!())
        .expect("error while running CodexBar");
}
