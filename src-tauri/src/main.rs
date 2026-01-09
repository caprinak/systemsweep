// src-tauri/src/main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use desktop_cleaner_lib::commands;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            // Initialize database
            let app_dir = app.path().app_data_dir().expect("Failed to get app data dir");
            std::fs::create_dir_all(&app_dir)?;
            
            let db_path = app_dir.join("cleaner.db");
            desktop_cleaner_lib::database::init_database(&db_path)?;
            
            // Store db path in state
            app.manage(desktop_cleaner_lib::state::AppState::new(db_path));

            // Set up Tray Icon (Tauri 2.0 style)
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
            let scan_i = MenuItem::with_id(app, "scan", "Quick Scan", true, None::<&str>)?;
            
            let tray_menu = Menu::with_items(app, &[&show_i, &scan_i, &quit_i])?;
            
            let _tray = TrayIconBuilder::new()
                .menu(&tray_menu)
                .on_menu_event(move |app, event| {
                    match event.id.as_ref() {
                        "quit" => {
                            app.exit(0);
                        }
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "scan" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.emit("trigger-quick-scan", ());
                            }
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Scanner commands
            commands::scan_system,
            commands::scan_directory,
            commands::scan_duplicates,
            commands::scan_large_files,
            commands::scan_old_files,
            commands::scan_cache,
            commands::cancel_scan,
            commands::get_scan_progress,
            
            // Cleanup commands
            commands::delete_files,
            commands::delete_files_dry_run,
            commands::secure_delete,
            commands::move_to_trash,
            commands::restore_files,
            commands::get_restore_points,
            
            // Startup management
            commands::get_startup_items,
            commands::toggle_startup_item,
            commands::add_startup_item,
            commands::remove_startup_item,
            
            // System info
            commands::get_system_info,
            commands::get_disk_usage,
            commands::get_memory_usage,
            commands::get_running_processes,
            
            // Scheduler
            commands::schedule_cleanup,
            commands::get_scheduled_tasks,
            commands::remove_scheduled_task,
            
            // Configuration
            commands::get_config,
            commands::save_config,
            commands::reset_config,
            
            // History
            commands::get_cleanup_history,
            commands::clear_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
