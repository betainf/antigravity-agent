use tauri::Manager;
use crate::system_tray::SystemTrayManager;

/// 启用系统托盘
#[tauri::command]
pub async fn enable_system_tray(app: tauri::AppHandle) -> Result<String, String> {
    let system_tray = app.state::<SystemTrayManager>();
    system_tray.enable(&app).await?;
    Ok("系统托盘已启用".to_string())
}

/// 禁用系统托盘
#[tauri::command]
pub async fn disable_system_tray(app: tauri::AppHandle) -> Result<String, String> {
    let system_tray = app.state::<SystemTrayManager>();
    system_tray.disable(&app)?;
    Ok("系统托盘已禁用".to_string())
}

/// 切换系统托盘状态
#[tauri::command]
pub async fn toggle_system_tray(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let system_tray = app.state::<SystemTrayManager>();
    let enabled = system_tray.toggle(&app).await?;
    
    Ok(serde_json::json!({
        "enabled": enabled,
        "message": if enabled { "系统托盘已启用" } else { "系统托盘已禁用" }
    }))
}

/// 获取系统托盘状态
#[tauri::command]
pub async fn get_system_tray_state(app: tauri::AppHandle) -> Result<bool, String> {
    let system_tray = app.state::<SystemTrayManager>();
    Ok(system_tray.is_enabled_setting(&app))
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

/// 检查系统托盘是否启用（兼容旧接口）
#[tauri::command]
pub async fn is_system_tray_enabled(app: tauri::AppHandle) -> Result<bool, String> {
    get_system_tray_state(app).await
}

#[tauri::command]
pub async fn save_system_tray_state(app: tauri::AppHandle, enabled: bool) -> Result<String, String> {
    let system_tray = app.state::<SystemTrayManager>();
    if enabled {
        system_tray.enable(&app).await?;
    } else {
        system_tray.disable(&app)?;
    }
    Ok("状态已保存".to_string())
}
