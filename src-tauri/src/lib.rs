use std::sync::{Arc, Mutex};
use tauri::Manager;

mod models;
mod services;
mod commands;

use services::process::{start_output_cache_maintenance, ProcessManager};
use services::rcon_client::RconClient;
use services::config_service::ConfigService;
use services::log_service::LogService;
use services::scheduler;
use services::system_monitor::SystemMonitor;
use commands::server::{ActiveRcon, AutoUpdateState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let base_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    let config_service = ConfigService::new(base_dir.clone());
    config_service.ensure_directories();

    let log_service = LogService::new(base_dir.join("logs"));
    log_service.log_app("应用程序启动");

    let process_manager = ProcessManager::new(base_dir.join("logs"));

    let config_arc = Arc::new(Mutex::new(config_service));
    let process_arc = Arc::new(Mutex::new(process_manager));
    let rcon_arc = Arc::new(Mutex::new(RconClient::new()));
    let log_arc = Arc::new(Mutex::new(log_service));
    let system_arc = Arc::new(Mutex::new(SystemMonitor::new()));
    let active_rcon = ActiveRcon::from_config(&config_arc.lock().unwrap());
    let active_rcon_arc = Arc::new(Mutex::new(active_rcon));
    let auto_update_arc = Arc::new(Mutex::new(AutoUpdateState::default()));

    // 启动后台调度器
    scheduler::start_scheduler(
        Arc::clone(&config_arc),
        Arc::clone(&process_arc),
        Arc::clone(&rcon_arc),
        Arc::clone(&log_arc),
    );
    start_output_cache_maintenance(Arc::clone(&process_arc), Arc::clone(&log_arc));

    let monitor_process = Arc::clone(&process_arc);
    let monitor_rcon = Arc::clone(&rcon_arc);
    let monitor_config = Arc::clone(&config_arc);
    let monitor_log = Arc::clone(&log_arc);
    let monitor_active_rcon = Arc::clone(&active_rcon_arc);
    let monitor_auto_update = Arc::clone(&auto_update_arc);

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
            }
        }))
        .setup(move |app| {
            commands::server::start_auto_update_monitor(
                app.handle().clone(),
                Arc::clone(&monitor_process),
                Arc::clone(&monitor_rcon),
                Arc::clone(&monitor_config),
                Arc::clone(&monitor_log),
                Arc::clone(&monitor_active_rcon),
                Arc::clone(&monitor_auto_update),
            );
            Ok(())
        })
        .manage(process_arc)
        .manage(rcon_arc)
        .manage(config_arc)
        .manage(log_arc)
        .manage(system_arc)
        .manage(active_rcon_arc)
        .manage(auto_update_arc)
        .invoke_handler(tauri::generate_handler![
            commands::server::get_server_status,
            commands::server::get_server_output,
            commands::server::get_server_snapshot,
            commands::server::get_public_ip,
            commands::server::get_server_port,
            commands::server::start_server,
            commands::server::stop_server,
            commands::server::force_stop_server,
            commands::server::restart_server,
            commands::rcon::rcon_connect,
            commands::rcon::rcon_disconnect,
            commands::rcon::rcon_send,
            commands::rcon::rcon_poll,
            commands::rcon::rcon_status,
            commands::config::get_config,
            commands::config::get_app_settings,
            commands::config::set_auto_update_hosting,
            commands::config::save_config,
            commands::config::is_first_run,
            commands::config::save_wizard_config,
            commands::config::auto_detect_paths,
            commands::logs::read_log_file,
            commands::logs::list_log_dates,
            commands::update::run_update,
            commands::schedule::get_schedules,
            commands::schedule::save_schedules,
            commands::system::get_system_stats,
            commands::save::list_server_saves,
            commands::save::read_commands_dat,
            commands::save::save_commands_dat,
            commands::save::list_plugins,
            commands::save::open_plugin_config_dir,
            commands::save::load_plugin_notes,
            commands::save::save_plugin_notes,
            commands::save::read_rocket_rcon_config,
            commands::save::save_rocket_rcon_config,
            commands::save::read_workshop_config,
            commands::save::save_workshop_config,
            commands::save::load_workshop_mod_notes,
            commands::save::save_workshop_mod_notes,
            commands::save::open_url,
            commands::installer::download_steamcmd,
            commands::installer::download_server,
            commands::setup::detect_rocket_module,
            commands::setup::install_rocket_module,
            commands::setup::init_server_save,
            commands::setup::check_save_rocket_status,
            commands::updater::check_for_updates,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
