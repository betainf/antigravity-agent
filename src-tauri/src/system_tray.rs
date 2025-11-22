//! 系统托盘管理模块
//!
//! 使用 Tauri 2.x 内置的系统托盘 API
//! 
//! 模块功能：
//! - 托盘图标加载与管理
//! - 动态菜单构建（账户切换、快速切换、子菜单）
//! - 菜单事件处理

use once_cell::sync::OnceCell;
use std::sync::{Arc, Mutex};
use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItem, SubmenuBuilder},
    tray::TrayIconBuilder,
    AppHandle, Manager,
};

/// 全局系统托盘管理器实例 - 使用 OnceCell 避免未定义行为
static SYSTEM_TRAY_MANAGER: OnceCell<Arc<Mutex<SystemTrayManager>>> = OnceCell::new();

/// 系统托盘管理器
pub struct SystemTrayManager {
    is_enabled: bool,
    app_handle: Option<AppHandle>,
    tray_icon: Option<tauri::tray::TrayIcon>,
    is_minimizing: bool, // 防止重入的标志
}

impl SystemTrayManager {
    /// 创建新的系统托盘管理器
    pub fn new() -> Self {
        Self {
            is_enabled: false,
            app_handle: None,
            tray_icon: None,
            is_minimizing: false,
        }
    }

    /// 初始化全局系统托盘管理器
    pub fn initialize_global(app_handle: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
        // 检查是否已经初始化
        if SYSTEM_TRAY_MANAGER.get().is_some() {
            return Ok(());
        }

        let mut manager = SystemTrayManager::new();
        manager.app_handle = Some(app_handle.clone());

        println!("📋 创建系统托盘图标");

        // 构建菜单
        let menu = build_tray_menu(app_handle)?;

        // 构建托盘图标
        let mut tray_builder = TrayIconBuilder::new()
            .menu(&menu)
            .tooltip("Antigravity Agent");

        // 加载托盘图标
        if let Some(icon) = load_tray_icon() {
            tray_builder = tray_builder.icon(icon);
        }

        // 创建托盘图标
        match tray_builder.build(app_handle) {
            Ok(tray) => {
                manager.tray_icon = Some(tray.clone());
                println!("✅ 系统托盘图标创建成功");

                // 设置菜单事件监听
                let app_handle_clone = app_handle.clone();
                tray.on_menu_event(move |app, event| {
                    handle_menu_event(app, event.id().as_ref(), &app_handle_clone);
                });
            }
            Err(e) => {
                println!("⚠️ 创建系统托盘图标失败: {}", e);
            }
        }

        // 使用 OnceCell 安全地设置全局实例
        let manager_arc = Arc::new(Mutex::new(manager));
        if SYSTEM_TRAY_MANAGER.set(manager_arc).is_err() {
            return Ok(());
        }

        println!("✅ 系统托盘管理器初始化成功");
        Ok(())
    }

    /// 重建托盘菜单
    pub fn rebuild_menu(app_handle: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(manager) = Self::get_global() {
            if let Ok(mgr) = manager.lock() {
                if let Some(ref tray) = mgr.tray_icon {
                    println!("🔄 重建托盘菜单");
                    let new_menu = build_tray_menu(app_handle)?;
                    tray.set_menu(Some(new_menu))?;
                    println!("✅ 托盘菜单已更新");
                }
            }
        }
        Ok(())
    }

    /// 获取全局系统托盘管理器
    pub fn get_global() -> Option<Arc<Mutex<SystemTrayManager>>> {
        SYSTEM_TRAY_MANAGER.get().map(Arc::clone)
    }

    /// 启用系统托盘功能
    pub fn enable(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.is_enabled = true;
        println!("✅ 系统托盘功能已启用");
        Ok(())
    }

    /// 禁用系统托盘功能
    pub fn disable(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.is_enabled = false;
        if self.tray_icon.take().is_some() {
            println!("🔴 系统托盘图标已移除");
        }
        println!("🔴 系统托盘功能已禁用");
        Ok(())
    }

    /// 检查系统托盘是否启用
    pub fn is_enabled(&self) -> bool {
        self.is_enabled
    }

    /// 最小化窗口到系统托盘
    pub fn minimize_to_tray(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_minimizing {
            println!("📋 已经在最小化到托盘的过程中，跳过重复调用");
            return Ok(());
        }

        if !self.is_enabled {
            return Err("系统托盘功能未启用".into());
        }

        self.is_minimizing = true;

        if let Some(app_handle) = &self.app_handle {
            if let Some(window) = app_handle.get_webview_window("main") {
                if let Err(e) = window.hide() {
                    self.is_minimizing = false;
                    return Err(format!("隐藏窗口失败: {}", e).into());
                }
                println!("📋 窗口已最小化到系统托盘");
            }
        }

        self.is_minimizing = false;
        Ok(())
    }

    /// 从系统托盘恢复窗口
    pub fn restore_from_tray(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(app_handle) = &self.app_handle {
            if let Some(window) = app_handle.get_webview_window("main") {
                window.show()?;
                window.set_focus()?;
                println!("📋 窗口已从系统托盘恢复");
            }
        }
        Ok(())
    }
}

// ============================================================================
// 图标加载功能
// ============================================================================

/// 加载托盘图标
fn load_tray_icon() -> Option<Image<'static>> {
    let tray_icon_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("icons")
        .join("tray-icon.png");

    if !tray_icon_path.exists() {
        println!("⚠️ 托盘图标文件不存在，使用默认图标");
        return None;
    }

    println!("📋 尝试加载托盘图标: {}", tray_icon_path.display());

    match std::fs::read(&tray_icon_path) {
        Ok(icon_data) => match image::load_from_memory(&icon_data) {
            Ok(img) => {
                let rgba_img = img.to_rgba8();
                let (width, height) = rgba_img.dimensions();
                let rgba_data = rgba_img.into_raw();
                let tauri_image = Image::new_owned(rgba_data, width as u32, height as u32);
                println!("✅ 托盘图标加载成功，尺寸: {}x{}", width, height);
                Some(tauri_image)
            }
            Err(e) => {
                println!("⚠️ 图像处理失败: {}", e);
                None
            }
        },
        Err(e) => {
            println!("⚠️ 读取图标文件失败: {}", e);
            None
        }
    }
}

// ============================================================================
// 菜单构建功能
// ============================================================================

/// 构建托盘菜单
fn build_tray_menu(
    app_handle: &AppHandle,
) -> Result<tauri::menu::Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    // 获取账户列表
    let recent_accounts = get_accounts_sync(app_handle, Some(2));
    let all_accounts = get_accounts_sync(app_handle, None);

    let mut menu_builder = MenuBuilder::new(app_handle);

    // 添加账户相关菜单
    if !all_accounts.is_empty() {
        // 快速切换（最近2个账户）
        if !recent_accounts.is_empty() {
            menu_builder = menu_builder.text("quick_switch_label", "快速切换");

            for account in &recent_accounts {
                let menu_id = format!("switch_account:{}", account);
                let item = MenuItem::with_id(
                    app_handle,
                    &menu_id,
                    format!("  {}", account),
                    true,
                    None::<&str>,
                )?;
                menu_builder = menu_builder.item(&item);
            }

            menu_builder = menu_builder.separator();
        }

        // 所有账户子菜单（超过2个时显示）
        if all_accounts.len() > 2 {
            let mut submenu_builder = SubmenuBuilder::new(app_handle, "所有账户");

            for account in &all_accounts {
                let menu_id = format!("switch_account:{}", account);
                let item = MenuItem::with_id(
                    app_handle,
                    &menu_id,
                    account,
                    true,
                    None::<&str>,
                )?;
                submenu_builder = submenu_builder.item(&item);
            }

            let submenu = submenu_builder.build()?;
            menu_builder = menu_builder.item(&submenu);
            menu_builder = menu_builder.separator();
        }

        // 刷新账户列表
        let refresh_item = MenuItem::with_id(
            app_handle,
            "refresh_accounts",
            "刷新账户列表",
            true,
            None::<&str>,
        )?;
        menu_builder = menu_builder.item(&refresh_item);
        menu_builder = menu_builder.separator();
    }

    // 窗口控制菜单
    let show_item = MenuItem::with_id(app_handle, "show", "显示窗口", true, None::<&str>)?;
    let hide_item = MenuItem::with_id(app_handle, "hide", "隐藏窗口", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app_handle, "quit", "退出应用", true, None::<&str>)?;

    menu_builder = menu_builder
        .item(&show_item)
        .separator()
        .item(&hide_item)
        .separator()
        .item(&quit_item);

    Ok(menu_builder.build()?)
}

/// 同步获取账户列表
fn get_accounts_sync(app_handle: &AppHandle, limit: Option<usize>) -> Vec<String> {
    let state = app_handle.state::<crate::AppState>();
    let result = tauri::async_runtime::block_on(async {
        crate::commands::backup_commands::get_recent_accounts(state, limit).await
    });

    result.unwrap_or_else(|e| {
        eprintln!("⚠️ 获取账户列表失败: {}", e);
        Vec::new()
    })
}

// ============================================================================
// 事件处理功能
// ============================================================================

/// 处理托盘菜单事件
fn handle_menu_event(app: &AppHandle, event_id: &str, app_handle_clone: &AppHandle) {
    println!("🖱️ 托盘菜单事件: {}", event_id);

    match event_id {
        "show" => handle_show_window(app),
        "hide" => handle_hide_window(app),
        "refresh_accounts" => handle_refresh_accounts(app_handle_clone),
        "quit" => handle_quit(app),
        id if id.starts_with("switch_account:") => handle_switch_account(app, id),
        _ => println!("🖱️ 未知菜单项: {}", event_id),
    }
}

/// 显示窗口
fn handle_show_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        println!("📋 菜单: 显示窗口");
    }
}

/// 隐藏窗口
fn handle_hide_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
        println!("📋 菜单: 隐藏窗口");
    }
}

/// 刷新账户列表
fn handle_refresh_accounts(app_handle_clone: &AppHandle) {
    println!("📋 菜单: 刷新账户列表");
    if let Err(e) = SystemTrayManager::rebuild_menu(app_handle_clone) {
        eprintln!("⚠️ 重建托盘菜单失败: {}", e);
    }
}

/// 退出应用
fn handle_quit(app: &AppHandle) {
    println!("📋 菜单: 退出应用");
    app.exit(0);
}

/// 切换账户
fn handle_switch_account(app: &AppHandle, menu_id: &str) {
    if let Some(account_name) = menu_id.strip_prefix("switch_account:") {
        println!("📋 菜单: 切换账户 -> {}", account_name);
        let account_name = account_name.to_string();
        let app_clone = app.clone();
        
        tauri::async_runtime::spawn(async move {
            match crate::commands::account_commands::switch_to_antigravity_account(account_name).await {
                Ok(msg) => {
                    println!("✅ 账户切换成功: {}", msg);
                    if let Err(e) = SystemTrayManager::rebuild_menu(&app_clone.app_handle()) {
                        eprintln!("⚠️ 重建托盘菜单失败: {}", e);
                    }
                }
                Err(e) => eprintln!("❌ 账户切换失败: {}", e),
            }
        });
    }
}
