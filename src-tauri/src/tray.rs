// ============================
// 系统托盘管理
// （对应 macOS NSStatusBar）
// ============================

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let quit_item = MenuItem::with_id(app, "quit", "Quit CodexBar", true, None::<&str>)?;
    let show_item = MenuItem::with_id(app, "show", "Open Dashboard", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let refresh_item = MenuItem::with_id(app, "refresh", "Refresh Now", true, None::<&str>)?;
    let settings_item = MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[&show_item, &refresh_item, &settings_item, &separator, &quit_item],
    )?;

    TrayIconBuilder::with_id("main")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => {
                app.exit(0);
            }
            "show" | "settings" => {
                toggle_window(app);
            }
            "refresh" => {
                // 触发前端刷新事件
                let _ = app.emit("tray-refresh", ());
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                toggle_window(app);
            }
        })
        .build(app)?;

    Ok(())
}

fn toggle_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            // 将窗口定位到托盘图标附近（右下角）
            position_window_near_tray(&window);
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

fn position_window_near_tray(window: &tauri::WebviewWindow) {
    // 获取屏幕尺寸，将窗口放在右下角
    if let Ok(monitor) = window.current_monitor() {
        if let Some(monitor) = monitor {
            let screen_size = monitor.size();
            let win_size = window.outer_size().unwrap_or(tauri::PhysicalSize::new(400, 600));

            let x = (screen_size.width as i32) - (win_size.width as i32) - 16;
            let y = (screen_size.height as i32) - (win_size.height as i32) - 60;

            let _ = window.set_position(tauri::PhysicalPosition::new(x.max(0), y.max(0)));
        }
    }
}

pub fn update_tray_title(app: &AppHandle, title: &str) {
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_tooltip(Some(title));
    }
}
