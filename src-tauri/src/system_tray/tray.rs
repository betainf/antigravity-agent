//! 系统托盘模块
//!
//! 使用 Tauri 2.9 内置的 tray API 实现后端控制托盘

use crate::app_settings::AppSettingsManager;
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::menu::{Menu, MenuBuilder, MenuItem};
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::{AppHandle, Emitter, Manager};

#[derive(Debug, Deserialize, Clone)]
pub struct TrayMenuLabels {
    pub show_main: String,
    pub quit: String,
}

impl Default for TrayMenuLabels {
    fn default() -> Self {
        Self {
            show_main: "Show Main Window".to_string(),
            quit: "Quit".to_string(),
        }
    }
}

/// 创建系统托盘（返回托盘实例）
pub fn create_tray_with_return(app: &AppHandle) -> Result<TrayIcon, String> {
    // 创建基础菜单（账户列表将由前端动态更新）
    let menu = create_basic_menu(app)?;

    // 构建托盘图标
    let tray = TrayIconBuilder::with_id("main")
        .menu(&menu)
        .on_menu_event(handle_tray_menu_event)
        .show_menu_on_left_click(true)
        .build(app)
        .map_err(|e| format!("创建系统托盘失败: {e}"))?;

    // 设置托盘图标
    if let Some(icon) = app.default_window_icon() {
        tray.set_icon(Some(icon.clone()))
            .map_err(|e| format!("设置托盘图标失败: {e}"))?;
    }

    Ok(tray)
}

/// 创建基础菜单（不含账户列表）
fn create_basic_menu(app: &AppHandle) -> Result<Menu<tauri::Wry>, String> {
    MenuBuilder::new(app)
        .item(
            &MenuItem::with_id(app, "show_main", "显示主窗口", true, None::<&str>)
                .map_err(|e| format!("创建显示主窗口菜单失败: {e}"))?,
        )
        .separator()
        .item(
            &MenuItem::with_id(app, "quit", "退出应用", true, None::<&str>)
                .map_err(|e| format!("创建退出菜单失败: {e}"))?,
        )
        .build()
        .map_err(|e| format!("构建基础菜单失败: {e}"))
}

/// 处理托盘菜单事件
fn handle_tray_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    tracing::info!("处理托盘菜单事件: {}", event.id.0);

    match event.id.0.as_str() {
        id if id.starts_with("show_main") => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        id if id.starts_with("quit") => {
            tracing::info!("退出应用");
            app.exit(0);
        }
        // 账户切换事件
        account_id if account_id.starts_with("account#") => {
            // ID Format: account#{nonce}#{email}
            let parts: Vec<&str> = account_id.splitn(3, '#').collect();
            if parts.len() < 3 {
                tracing::warn!("无效的账户菜单ID: {}", account_id);
                return;
            }
            let account_email = parts[2];
            tracing::info!("请求切换到账户: {account_email}");

            // 发射事件到前端
            if let Err(e) = app.emit("tray-switch-account", account_email) {
                tracing::error!("发射账户切换事件失败: {e}");
            }
        }
        _ => {
            tracing::warn!("未处理的菜单事件: {}", event.id.0);
        }
    }
}

/// 更新托盘菜单（添加账户列表）
pub fn update_tray_menu(
    app: &AppHandle,
    accounts: Vec<String>,
    labels: Option<TrayMenuLabels>,
) -> Result<(), String> {
    // 检查托盘是否应该启用
    let settings_manager = app.state::<AppSettingsManager>();
    let settings = settings_manager.get_settings();

    if !settings.system_tray_enabled {
        tracing::info!("托盘已禁用，跳过菜单更新");
        return Ok(());
    }

    let Some(tray) = app.tray_by_id("main") else {
        return Err("未找到系统托盘".to_string());
    };

    // 使用默认或传入的标签
    let menu_labels = labels.unwrap_or_default();

    // 创建包含账户列表的完整菜单
    let mut menu_builder = MenuBuilder::new(app);

    // Generate unique nonce for this update to avoid ID collisions
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    // 显示主窗口
    menu_builder = menu_builder.item(
        &MenuItem::with_id(
            app,
            format!("show_main#{}", nonce),
            &menu_labels.show_main,
            true,
            None::<&str>,
        )
        .map_err(|e| format!("创建显示主窗口菜单失败: {e}"))?,
    );

    // 添加账户列表
    if !accounts.is_empty() {
        menu_builder = menu_builder.separator();

        for account in &accounts {
            let masked_email = mask_email(account);
            menu_builder = menu_builder.item(
                &MenuItem::with_id(
                    app,
                    format!("account#{}#{}", nonce, account),
                    &masked_email,
                    true,
                    None::<&str>,
                )
                .map_err(|e| format!("创建账户菜单失败: {e}"))?,
            );
        }
    }

    // 退出应用
    menu_builder = menu_builder.separator().item(
        &MenuItem::with_id(
            app,
            format!("quit#{}", nonce),
            &menu_labels.quit,
            true,
            None::<&str>,
        )
        .map_err(|e| format!("创建退出菜单失败: {e}"))?,
    );

    // 构建并设置新菜单
    let new_menu = menu_builder
        .build()
        .map_err(|e| format!("构建新菜单失败: {e}"))?;

    tray.set_menu(Some(new_menu))
        .map_err(|e| format!("设置托盘菜单失败: {e}"))?;

    tracing::info!("✅ 托盘菜单已更新，包含 {} 个账户", accounts.len());
    Ok(())
}

/// 邮箱打码函数
fn mask_email(email: &str) -> String {
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return email.to_string();
    }

    let (local_part, domain) = (parts[0], parts[1]);

    match local_part.len() {
        0 => email.to_string(),
        1 => format!("{}*@{}", &local_part[..1], domain),
        2 => format!("{}*@{}", &local_part[..1], domain),
        _ => format!(
            "{}***{}@{}",
            &local_part[..1],
            &local_part[local_part.len() - 1..],
            domain
        ),
    }
}
