use tauri::{AppHandle, State, Manager};
use std::sync::{Arc, Mutex};
use crate::services::config_service::ConfigService;
use crate::models::config::AppSettings;

#[tauri::command]
pub fn should_show_close_dialog(
    config: State<'_, Arc<Mutex<ConfigService>>>,
) -> bool {
    let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
    let settings = cfg.load_app_settings();
    !settings.close_action_remembered
}

#[tauri::command]
pub fn save_close_preference(
    config: State<'_, Arc<Mutex<ConfigService>>>,
    close_to_tray: bool,
    remember: bool,
) -> Result<(), String> {
    let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
    let mut settings = cfg.load_app_settings();

    settings.close_to_tray = close_to_tray;
    settings.close_action_remembered = remember;

    cfg.save_app_settings(&settings)?;
    Ok(())
}

#[tauri::command]
pub fn get_close_preference(
    config: State<'_, Arc<Mutex<ConfigService>>>,
) -> AppSettings {
    let cfg = config.lock().unwrap_or_else(|e| e.into_inner());
    cfg.load_app_settings()
}

#[tauri::command]
pub fn hide_window_to_tray(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn show_window_from_tray(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn quit_app(app: AppHandle) {
    app.exit(0);
}
