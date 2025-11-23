//! 系统托盘管理模块
//!
//! 负责系统托盘的生命周期管理、菜单构建和事件处理。
//! 使用 AppSettingsManager 进行状态持久化。

use std::sync::Mutex;
use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItem, SubmenuBuilder},
    tray::{MouseButton, TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Wry,
};

use crate::app_settings::AppSettingsManager;

/// 系统托盘管理器
pub struct SystemTrayManager {
    /// 托盘图标实例（使用 Mutex 保护，支持内部可变性）
    tray_icon: Mutex<Option<TrayIcon>>,
}

// 强制实现 Send 和 Sync，因为 TrayIcon 只是一个句柄，且我们使用了 Mutex 进行保护
unsafe impl Send for SystemTrayManager {}
unsafe impl Sync for SystemTrayManager {}

impl SystemTrayManager {
    /// 创建新的管理器
    pub fn new() -> Self {
        Self {
            tray_icon: Mutex::new(None),
        }
    }

    /// 初始化系统托盘
    /// 在应用启动时调用，根据保存的设置决定是否显示托盘
    pub fn initialize(&self, app_handle: &AppHandle) -> Result<(), String> {
        let settings_manager = app_handle.state::<AppSettingsManager>();
        let settings = settings_manager.get_settings();

        if settings.system_tray_enabled {
            // 初始化时异步创建图标
            let app_handle_clone = app_handle.clone();
 
            // Actually SystemTrayManager is managed by Tauri which wraps it in Arc.
            // But here we are inside a method of SystemTrayManager.
            
            // We can just spawn a task to do the async work
            tauri::async_runtime::spawn(async move {
                let manager = app_handle_clone.state::<SystemTrayManager>();
                if let Err(e) = manager.create_tray_icon(&app_handle_clone).await {
                    eprintln!("初始化托盘图标失败: {}", e);
                }
            });
        }

        Ok(())
    }

    /// 启用系统托盘
    pub async fn enable(&self, app_handle: &AppHandle) -> Result<(), String> {
        // 1. 更新设置
        let settings_manager = app_handle.state::<AppSettingsManager>();
        settings_manager
            .update_settings(|s| s.system_tray_enabled = true)
            .map_err(|e| e.to_string())?;

        // 2. 创建图标（如果不存在）
        if !self.is_tray_created() {
            self.create_tray_icon(app_handle).await?;
        }

        Ok(())
    }

    /// 禁用系统托盘
    pub fn disable(&self, app_handle: &AppHandle) -> Result<(), String> {
        // 1. 更新设置
        let settings_manager = app_handle.state::<AppSettingsManager>();
        settings_manager
            .update_settings(|s| s.system_tray_enabled = false)
            .map_err(|e| e.to_string())?;

        // 2. 销毁图标
        self.destroy_tray_icon();

        Ok(())
    }

    /// 切换系统托盘状态
    pub async fn toggle(&self, app_handle: &AppHandle) -> Result<bool, String> {
        let settings_manager = app_handle.state::<AppSettingsManager>();
        let is_enabled = settings_manager.get_settings().system_tray_enabled;

        if is_enabled {
            self.disable(app_handle)?;
            Ok(false)
        } else {
            self.enable(app_handle).await?;
            Ok(true)
        }
    }

    /// 检查系统托盘是否应启用（基于设置）
    pub fn is_enabled_setting(&self, app_handle: &AppHandle) -> bool {
        app_handle
            .state::<AppSettingsManager>()
            .get_settings()
            .system_tray_enabled
    }

    /// 检查托盘图标是否已创建（运行时状态）
    pub fn is_tray_created(&self) -> bool {
        self.tray_icon.lock().unwrap().is_some()
    }

    /// 最小化窗口到托盘
    pub fn minimize_to_tray(&self, app_handle: &AppHandle) -> Result<(), String> {
        if let Some(window) = app_handle.get_webview_window("main") {
            window.hide().map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// 从托盘恢复窗口
    pub fn restore_from_tray(&self, app_handle: &AppHandle) -> Result<(), String> {
        if let Some(window) = app_handle.get_webview_window("main") {
            window.show().map_err(|e| e.to_string())?;
            window.set_focus().map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    // --- 内部辅助方法 ---

    /// 创建托盘图标
    async fn create_tray_icon(&self, app_handle: &AppHandle) -> Result<(), String> {
        // 1. 快速检查：如果已存在则直接返回
        {
            let tray_lock = self.tray_icon.lock().unwrap();
            if tray_lock.is_some() {
                return Ok(());
            }
        }

        // 2. 构建菜单（这是一个异步操作，不能持有锁）
        let menu = self.build_menu(app_handle).await.map_err(|e| e.to_string())?;
        
        // 3. 再次获取锁进行创建（双重检查）
        let mut tray_lock = self.tray_icon.lock().unwrap();
        if tray_lock.is_some() {
            return Ok(());
        }

        let mut builder = TrayIconBuilder::new()
            .menu(&menu)
            .tooltip("Antigravity Agent")
            .on_menu_event(|app, event| {
                let id = event.id.as_ref().to_string();
                let app_clone = app.clone();
                tauri::async_runtime::spawn(async move {
                    Self::handle_menu_event(&app_clone, &id).await;
                });
            })
            .on_tray_icon_event(|tray, event| {
                if let TrayIconEvent::Click {
                    button: MouseButton::Left,
                    ..
                } = event
                {
                    let app = tray.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            });

        if let Some(icon) = self.load_icon() {
            builder = builder.icon(icon);
        }

        let tray = builder.build(app_handle).map_err(|e| e.to_string())?;
        *tray_lock = Some(tray);

        println!("✅ 系统托盘图标已创建");
        Ok(())
    }

    /// 销毁托盘图标
    fn destroy_tray_icon(&self) {
        let mut tray_lock = self.tray_icon.lock().unwrap();
        if let Some(tray) = tray_lock.take() {
            // 显式隐藏图标，确保从系统托盘移除
            if let Err(e) = tray.set_visible(false) {
                eprintln!("隐藏托盘图标失败: {}", e);
            }
        }
        println!("✅ 系统托盘图标已销毁");
    }

    /// 加载图标资源
    fn load_icon(&self) -> Option<Image<'static>> {
        let icon_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("icons")
            .join("tray-icon.png");

        if icon_path.exists() {
            if let Ok(icon_data) = std::fs::read(icon_path) {
                if let Ok(image) = image::load_from_memory(&icon_data) {
                     let rgba = image.to_rgba8();
                     let (w, h) = rgba.dimensions();
                     return Some(Image::new_owned(rgba.into_raw(), w, h));
                }
            }
        }
        None
    }

    /// 构建托盘菜单
    async fn build_menu(&self, app_handle: &AppHandle) -> tauri::Result<tauri::menu::Menu<Wry>> {
        let mut menu_builder = MenuBuilder::new(app_handle);

        // 1. 获取账户列表
        let state = app_handle.state::<crate::AppState>();
        let recent_accounts = crate::commands::backup_commands::get_recent_accounts(state.clone(), Some(2)).await.unwrap_or_default();
        let all_accounts = crate::commands::backup_commands::get_recent_accounts(state.clone(), None).await.unwrap_or_default();

        // 2. 添加账户相关菜单
        if !all_accounts.is_empty() {
            // 快速切换（最近2个账户）
            if !recent_accounts.is_empty() {
                let label_item = MenuItem::new(app_handle, "快速切换", false, None::<&str>)?;
                menu_builder = menu_builder.item(&label_item);

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

        // 3. 窗口控制菜单
        let show_item = MenuItem::with_id(app_handle, "show", "显示窗口", true, None::<&str>)?;
        let hide_item = MenuItem::with_id(app_handle, "hide", "隐藏窗口", true, None::<&str>)?;
        let quit_item = MenuItem::with_id(app_handle, "quit", "退出应用", true, None::<&str>)?;

        menu_builder = menu_builder
            .item(&show_item)
            .separator()
            .item(&hide_item)
            .separator()
            .item(&quit_item);

        menu_builder.build()
    }

    /// 处理菜单事件
    async fn handle_menu_event(app: &AppHandle, event_id: &str) {
        match event_id {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "hide" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }
            "quit" => {
                app.exit(0);
            }
            "refresh_accounts" => {
                let system_tray = app.state::<SystemTrayManager>();
                if let Err(e) = system_tray.update_menu(app).await {
                    eprintln!("刷新托盘菜单失败: {}", e);
                }
            }
            id if id.starts_with("switch_account:") => {
                if let Some(account_name) = id.strip_prefix("switch_account:") {
                    println!("📋 菜单: 切换账户 -> {}", account_name);
                    let account_name = account_name.to_string();
                    
                    match crate::commands::account_commands::switch_to_antigravity_account(account_name).await {
                        Ok(msg) => {
                            println!("✅ 账户切换成功: {}", msg);
                            let system_tray = app.state::<SystemTrayManager>();
                            if let Err(e) = system_tray.update_menu(app).await {
                                eprintln!("重建托盘菜单失败: {}", e);
                            }
                        }
                        Err(e) => eprintln!("❌ 账户切换失败: {}", e),
                    }
                }
            }
            _ => {}
        }
    }
    
    /// 重建并更新菜单（用于账户列表更新）
    pub async fn update_menu(&self, app_handle: &AppHandle) -> Result<(), String> {
        // 1. 先构建菜单（异步操作，不持有锁）
        let menu = self.build_menu(app_handle).await.map_err(|e| e.to_string())?;
        
        // 2. 获取锁并更新
        let tray_lock = self.tray_icon.lock().unwrap();
        if let Some(tray) = tray_lock.as_ref() {
             tray.set_menu(Some(menu)).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}
