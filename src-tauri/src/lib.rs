use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use tauri::{Manager, Emitter, AppHandle};

mod commands;
mod models;
mod services;

use commands::server::{ActiveRcon, AutoUpdateState};
use services::config_service::ConfigService;
use services::log_service::LogService;
use services::process::{start_output_cache_maintenance, ProcessManager};
use services::rcon_client::RconClient;
use services::scheduler;
use services::system_monitor::SystemMonitor;

// 构建托盘菜单
fn build_tray_menu(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // 获取当前自动托管状态
    let config_state: tauri::State<Arc<Mutex<ConfigService>>> = app.state();
    let cfg = config_state.lock().unwrap_or_else(|e| e.into_inner());
    let settings = cfg.load_app_settings();
    let auto_hosting_enabled = settings.auto_update_hosting;
    drop(cfg);

    let hosting_label = if auto_hosting_enabled {
        "✓ 关闭托管"
    } else {
        "启动托管"
    };

    // 创建菜单项
    let show_i = tauri::menu::MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
    let sep1 = tauri::menu::PredefinedMenuItem::separator(app)?;
    let dashboard_i = tauri::menu::MenuItem::with_id(app, "dashboard", "仪表盘", true, None::<&str>)?;
    let server_i = tauri::menu::MenuItem::with_id(app, "server", "服务器", true, None::<&str>)?;
    let settings_i = tauri::menu::MenuItem::with_id(app, "settings", "设置", true, None::<&str>)?;
    let sep2 = tauri::menu::PredefinedMenuItem::separator(app)?;
    let auto_hosting_i = tauri::menu::MenuItem::with_id(app, "auto_hosting", hosting_label, true, None::<&str>)?;
    let sep3 = tauri::menu::PredefinedMenuItem::separator(app)?;
    let quit_i = tauri::menu::MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

    let menu = tauri::menu::Menu::with_items(app, &[
        &show_i,
        &sep1,
        &dashboard_i,
        &server_i,
        &settings_i,
        &sep2,
        &auto_hosting_i,
        &sep3,
        &quit_i,
    ])?;

    // 构建托盘图标
    let _tray = tauri::tray::TrayIconBuilder::with_id("main")
        .menu(&menu)
        .icon(app.default_window_icon().unwrap().clone())
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "dashboard" | "server" | "settings" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                        let _ = window.emit("navigate", event.id.as_ref());
                    }
                }
                "auto_hosting" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.emit("toggle-auto-hosting", ());
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                button_state: tauri::tray::MouseButtonState::Up,
                ..
            } = event {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}

// 重建托盘菜单的命令
#[tauri::command]
fn rebuild_tray_menu(app: AppHandle) -> Result<(), String> {
    // 获取当前自动托管状态
    let config_state: tauri::State<Arc<Mutex<ConfigService>>> = app.state();
    let cfg = config_state.lock().unwrap_or_else(|e| e.into_inner());
    let settings = cfg.load_app_settings();
    let auto_hosting_enabled = settings.auto_update_hosting;
    drop(cfg);

    let hosting_label = if auto_hosting_enabled {
        "✓ 关闭托管"
    } else {
        "启动托管"
    };

    // 创建菜单项
    let show_i = tauri::menu::MenuItem::with_id(&app, "show", "显示窗口", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let sep1 = tauri::menu::PredefinedMenuItem::separator(&app)
        .map_err(|e| e.to_string())?;
    let dashboard_i = tauri::menu::MenuItem::with_id(&app, "dashboard", "仪表盘", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let server_i = tauri::menu::MenuItem::with_id(&app, "server", "服务器", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let settings_i = tauri::menu::MenuItem::with_id(&app, "settings", "设置", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let sep2 = tauri::menu::PredefinedMenuItem::separator(&app)
        .map_err(|e| e.to_string())?;
    let auto_hosting_i = tauri::menu::MenuItem::with_id(&app, "auto_hosting", hosting_label, true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let sep3 = tauri::menu::PredefinedMenuItem::separator(&app)
        .map_err(|e| e.to_string())?;
    let quit_i = tauri::menu::MenuItem::with_id(&app, "quit", "退出", true, None::<&str>)
        .map_err(|e| e.to_string())?;

    let menu = tauri::menu::Menu::with_items(&app, &[
        &show_i,
        &sep1,
        &dashboard_i,
        &server_i,
        &settings_i,
        &sep2,
        &auto_hosting_i,
        &sep3,
        &quit_i,
    ]).map_err(|e| e.to_string())?;

    // 更新托盘菜单
    if let Some(tray) = app.tray_by_id("main") {
        tray.set_menu(Some(menu)).map_err(|e| e.to_string())?;
    }

    Ok(())
}

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
    let auto_update_stop = Arc::new(AtomicBool::new(false));

    // 启动后台调度器
    scheduler::start_scheduler(
        Arc::clone(&config_arc),
        Arc::clone(&process_arc),
        Arc::clone(&log_arc),
        Arc::clone(&auto_update_arc),
    );
    start_output_cache_maintenance(Arc::clone(&process_arc), Arc::clone(&log_arc));

    let monitor_process = Arc::clone(&process_arc);
    let monitor_config = Arc::clone(&config_arc);
    let monitor_log = Arc::clone(&log_arc);
    let monitor_active_rcon = Arc::clone(&active_rcon_arc);
    let monitor_auto_update = Arc::clone(&auto_update_arc);
    let monitor_stop = Arc::clone(&auto_update_stop);

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
            }
        }))
        .setup(move |app| {
            // 启动自动更新监控
            commands::server::start_auto_update_monitor(
                app.handle().clone(),
                Arc::clone(&monitor_process),
                Arc::clone(&monitor_config),
                Arc::clone(&monitor_log),
                Arc::clone(&monitor_active_rcon),
                Arc::clone(&monitor_auto_update),
                monitor_stop,
            );

            // 创建初始托盘菜单
            build_tray_menu(app)?;

            // 监听窗口关闭事件
            let window = app.get_webview_window("main").unwrap();
            let window_clone = window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window_clone.emit("close-requested", ());
                }
            });

            Ok(())
        })
        .manage(process_arc)
        .manage(rcon_arc)
        .manage(config_arc)
        .manage(log_arc)
        .manage(system_arc)
        .manage(active_rcon_arc)
        .manage(auto_update_arc)
        .manage(auto_update_stop)
        .invoke_handler(tauri::generate_handler![
            commands::server::get_server_status,
            commands::server::get_server_output,
            commands::server::get_server_snapshot,
            commands::server::send_server_command,
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
            commands::environment::check_runtime_environment,
            commands::environment::install_runtime_requirement,
            commands::logs::read_log_file,
            commands::logs::list_log_dates,
            commands::update::run_update,
            commands::schedule::get_schedules,
            commands::schedule::save_schedules,
            commands::system::get_system_stats,
            commands::save::list_server_saves,
            commands::save::read_commands_dat,
            commands::save::save_commands_dat,
            commands::save::read_game_config,
            commands::save::save_game_config,
            commands::save::list_plugins,
            commands::save::open_plugin_config_dir,
            commands::save::load_plugin_notes,
            commands::save::save_plugin_notes,
            commands::save::read_rocket_rcon_config,
            commands::save::save_rocket_rcon_config,
            commands::save::read_workshop_config,
            commands::save::save_workshop_config,
            commands::save::read_permissions_config,
            commands::save::save_permissions_config,
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
            commands::window::should_show_close_dialog,
            commands::window::save_close_preference,
            commands::window::get_close_preference,
            commands::window::hide_window_to_tray,
            commands::window::show_window_from_tray,
            commands::window::quit_app,
            rebuild_tray_menu,
            commands::window::show_window_from_tray,
            commands::window::quit_app,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
