use crate::system_tray::{update_tray_menu, SystemTrayManager, TrayMenuLabels};
use tauri::Manager;

/// 更新托盘菜单（新增命令，供前端调用）
#[tauri::command]
pub async fn update_tray_menu_command(
    app: tauri::AppHandle,
    accounts: Vec<String>,
    labels: Option<TrayMenuLabels>,
) -> Result<String, String> {
    update_tray_menu(&app, accounts, labels)?;
    Ok("托盘菜单已更新".to_string())
}

/// 最小化到托盘
#[tauri::command]
pub async fn minimize_to_tray(app: tauri::AppHandle) -> Result<String, String> {
    let system_tray = app.state::<SystemTrayManager>();
    system_tray.minimize_to_tray(&app)?;
    Ok("已最小化到托盘".to_string())
}

/// 从托盘恢复
#[tauri::command]
pub async fn restore_from_tray(app: tauri::AppHandle) -> Result<String, String> {
    let system_tray = app.state::<SystemTrayManager>();
    system_tray.restore_from_tray(&app)?;
    Ok("已恢复窗口".to_string())
}
