// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use log::LevelFilter;
use rusqlite::Connection;

/// Antigravity 清理模块
mod antigravity_cleanup;

/// Antigravity 备份模块
mod antigravity_backup;

/// Antigravity 恢复模块
mod antigravity_restore;

/// Antigravity 启动模块
mod antigravity_starter;

/// 窗口状态管理模块
mod window_state_manager;

/// 窗口事件处理模块
mod window_event_handler;

/// 系统托盘模块
mod system_tray;

/// 平台工具模块
mod platform_utils;

/// 常量定义模块
mod constants;

/// 配置管理器模块
mod config_manager;

/// 应用设置模块
mod app_settings;

/// 工具模块
mod utils;

/// Antigravity 路径配置模块
mod antigravity_path_config;

/// 命令模块
mod commands;

/// 路径处理模块
mod path_utils;

// 重新导出命令函数以保持 invoke_handler 兼容性
use crate::commands::{
    backup_and_restart_antigravity,
    backup_antigravity_current_account,
    backup_profile,
    clear_all_antigravity_data,
    clear_all_backups,
    clear_logs,
    collect_backup_contents,
    delete_backup,
    detect_antigravity_executable,  // 新增
    detect_antigravity_installation,  // 新增
    disable_system_tray,
    // tray_commands
    enable_system_tray,
    // 日志导出命令
    export_logs,
    find_antigravity_installations,
    get_antigravity_accounts,
    get_current_antigravity_info,
    get_log_content,
    get_log_info,
    get_recent_accounts,
    // platform_commands
    get_current_paths,  // 新增
    get_platform_info,
    get_system_tray_state,
    is_system_tray_enabled,
    toggle_system_tray,
    // process_commands
    kill_antigravity,
    is_antigravity_running,  // 新增
    list_antigravity_processes,  // 新增调试命令
    list_backups,
    minimize_to_tray,
    // 最后2个有依赖的函数
    restore_antigravity_account,
    restore_backup_files,
    restore_from_tray,
    restore_profile,
    save_antigravity_executable,  // 新增
    save_antigravity_path,  // 新增
    save_system_tray_state,
    start_antigravity,
    // account_commands (前5个零依赖函数)
    switch_antigravity_account,
    switch_to_antigravity_account,
    validate_antigravity_executable,  // 新增
    validate_antigravity_path,
};

#[derive(Debug, Serialize, Deserialize)]
struct ProfileInfo {
    name: String,
    source_path: String,
    backup_path: String,
    created_at: String,
    last_updated: String,
}

// Antigravity 账户信息结构
#[derive(Debug, Serialize, Deserialize)]
struct AntigravityAccount {
    id: String,
    name: String,
    email: String,
    api_key: String,
    profile_url: String,   // Base64 编码的头像
    user_settings: String, // 编码后的用户设置
    created_at: String,
    last_switched: String,
}

// 导入系统托盘管理器

#[derive(Debug, Serialize, Deserialize)]
struct AppState {
    profiles: HashMap<String, ProfileInfo>,
    config_dir: PathBuf,
    antigravity_accounts: HashMap<String, AntigravityAccount>,
    current_account_id: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        // 智能检测配置目录，确保跨平台兼容性
        let config_dir = if cfg!(windows) {
            // Windows: 优先使用 APPDATA 环境变量
            std::env::var_os("APPDATA")
                .map(|appdata| PathBuf::from(appdata).join(".antigravity-agent"))
                .or_else(|| {
                    // 备用方案：通过用户主目录构建 AppData\Roaming 路径
                    dirs::home_dir().map(|home| {
                        home.join("AppData")
                            .join("Roaming")
                            .join(".antigravity-agent")
                    })
                })
                .or_else(|| {
                    // 最后备用：使用系统标准配置目录
                    dirs::config_dir().map(|config| config.join(".antigravity-agent"))
                })
                .unwrap_or_else(|| PathBuf::from(".antigravity-agent"))
        } else {
            // macOS/Linux: 使用标准配置目录
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".antigravity-agent")
        };

        // 确保配置目录存在
        fs::create_dir_all(&config_dir)
            .map_err(|e| eprintln!("警告：无法创建配置目录 {:?}: {}", config_dir, e))
            .ok();

        Self {
            profiles: HashMap::new(),
            config_dir,
            antigravity_accounts: HashMap::new(),
            current_account_id: None,
        }
    }
}

fn main() {
    println!("🚀 启动 Antigravity Agent");
    println!("🔧 [main] 开始初始化应用程序...");

    // 记录系统启动信息
    crate::utils::log_decorator::log_system_info();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(AppState::default())
        .setup(|app| {
            println!("🔧 [setup] 开始应用程序设置...");
            
            // 初始化应用设置管理器
            let app_handle = app.handle();
            app.manage(app_settings::AppSettingsManager::new(app_handle));
            
            // 初始化系统托盘管理器
            app.manage(system_tray::SystemTrayManager::new());

            // 初始化简单日志记录器
            println!("🔧 [setup] 初始化日志记录器...");
            let log_dir = dirs::config_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("antigravity-agent")
                .join("logs");
            fs::create_dir_all(&log_dir).ok();

            simple_logging::log_to_file(log_dir.join("antigravity-agent.log"), LevelFilter::Info)
                .ok();
            println!("✅ [setup] 日志记录器初始化完成");

            // 在 release 模式下禁用右键菜单
            #[cfg(not(debug_assertions))]
            {
                if let Some(window) = app.get_webview_window("main") {
                    // Tauri 2.x 中禁用上下文菜单需要通过eval执行JavaScript
                    let _ = window
                        .eval("window.addEventListener('contextmenu', e => e.preventDefault());");
                }
            }

            // 初始化系统托盘管理器
            println!("🔧 [setup] 开始初始化系统托盘管理器...");
            let system_tray = app.state::<system_tray::SystemTrayManager>();
            match system_tray.initialize(app.handle()) {
                Ok(_) => println!("✅ [setup] 系统托盘管理器初始化成功"),
                Err(e) => println!("⚠️ [setup] 系统托盘管理器初始化失败: {}", e),
            }

            // 初始化窗口事件处理器
            println!("🔧 [setup] 初始化窗口事件处理器...");
            if let Err(e) = window_event_handler::init_window_event_handler(app) {
                eprintln!("⚠️  窗口事件处理器初始化失败: {}", e);
            }
            println!("✅ [setup] 窗口事件处理器初始化完成");

            println!("✅ [setup] 应用程序设置完成");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            backup_profile,
            restore_profile,
            list_backups,
            get_recent_accounts,
            collect_backup_contents,
            restore_backup_files,
            delete_backup,
            clear_all_backups,
            // Antigravity 相关命令
            switch_antigravity_account,
            get_antigravity_accounts,
            get_current_antigravity_info,
            backup_antigravity_current_account,
            restore_antigravity_account,
            switch_to_antigravity_account,
            clear_all_antigravity_data,
            // 进程管理命令
            kill_antigravity,
            is_antigravity_running,  // 新增
            list_antigravity_processes,  // 新增调试命令
            start_antigravity,
            backup_and_restart_antigravity,
            // 平台支持命令
            get_platform_info,
            find_antigravity_installations,
            get_current_paths,  // 新增
            // 数据库路径相关
            validate_antigravity_path,
            detect_antigravity_installation,
            save_antigravity_path,
            // 可执行文件路径相关
            validate_antigravity_executable,
            detect_antigravity_executable,
            save_antigravity_executable,
            enable_system_tray,
            disable_system_tray,
            minimize_to_tray,
            restore_from_tray,
            is_system_tray_enabled,
            save_system_tray_state,
            get_system_tray_state,
            toggle_system_tray,
            export_logs,
            get_log_content,
            get_log_info,
            clear_logs
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
