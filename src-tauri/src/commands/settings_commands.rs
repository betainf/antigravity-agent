//! 应用设置命令
//! 负责应用程序配置的管理和存储，使用 State 模式

use tauri::{AppHandle, Manager};

fn validate_oauth_input(client_id: &str, client_secret: &str) -> Result<(), String> {
    if client_id.trim().is_empty() {
        return Err("client_id 不能为空".to_string());
    }
    if client_secret.trim().is_empty() {
        return Err("client_secret 不能为空".to_string());
    }
    if client_id.len() > 4096 || client_secret.len() > 8192 {
        return Err("OAuth 凭据长度异常".to_string());
    }
    Ok(())
}

#[tauri::command]
pub async fn save_oauth_credentials(
    client_id: String,
    client_secret: String,
) -> Result<bool, String> {
    crate::log_async_command!("save_oauth_credentials", async {
        validate_oauth_input(&client_id, &client_secret)?;
        crate::oauth_credentials::save_oauth_credentials_to_keyring(&client_id, &client_secret)?;
        Ok(true)
    })
}

#[tauri::command]
pub async fn clear_oauth_credentials() -> Result<bool, String> {
    crate::log_async_command!("clear_oauth_credentials", async {
        crate::oauth_credentials::clear_oauth_credentials_from_keyring()?;
        Ok(true)
    })
}

#[tauri::command]
pub async fn has_oauth_credentials() -> Result<bool, String> {
    crate::log_async_command!("has_oauth_credentials", async {
        crate::oauth_credentials::has_oauth_credentials_in_keyring()
    })
}

/// 保存系统托盘状态
#[tauri::command]
pub async fn save_system_tray_state(app: AppHandle, enabled: bool) -> Result<bool, String> {
    crate::log_async_command!("save_system_tray_state", async {
        let system_tray = app.state::<crate::system_tray::SystemTrayManager>();

        if enabled {
            system_tray.enable(&app)?;
        } else {
            system_tray.disable(&app)?;
        }

        let settings_manager = app.state::<crate::app_settings::AppSettingsManager>();
        let settings = settings_manager.get_settings();
        Ok(settings.system_tray_enabled)
    })
}

/// 保存静默启动状态
#[tauri::command]
pub async fn save_silent_start_state(app: AppHandle, enabled: bool) -> Result<bool, String> {
    crate::log_async_command!("save_silent_start_state", async {
        let settings_manager = app.state::<crate::app_settings::AppSettingsManager>();

        settings_manager.update_settings(|settings| {
            settings.silent_start_enabled = enabled;
        })?;

        let settings = settings_manager.get_settings();
        Ok(settings.silent_start_enabled)
    })
}

/// 保存隐私模式状态
#[tauri::command]
pub async fn save_private_mode_state(app: AppHandle, enabled: bool) -> Result<bool, String> {
    crate::log_async_command!("save_private_mode_state", async {
        let settings_manager = app.state::<crate::app_settings::AppSettingsManager>();

        settings_manager.update_settings(|settings| {
            settings.private_mode = enabled;
        })?;

        let settings = settings_manager.get_settings();
        Ok(settings.private_mode)
    })
}

/// 保存 Debug Mode 状态
#[tauri::command]
pub async fn save_debug_mode_state(app: AppHandle, enabled: bool) -> Result<bool, String> {
    crate::log_async_command!("save_debug_mode_state", async {
        let settings_manager = app.state::<crate::app_settings::AppSettingsManager>();

        settings_manager.update_settings(|settings| {
            settings.debug_mode = enabled;
        })?;

        let settings = settings_manager.get_settings();
        Ok(settings.debug_mode)
    })
}

/// 获取所有应用设置
#[tauri::command]
pub async fn get_all_settings(app: AppHandle) -> Result<serde_json::Value, String> {
    crate::log_async_command!("get_all_settings", async {
        let settings_manager = app.state::<crate::app_settings::AppSettingsManager>();
        let settings = settings_manager.get_settings();

        Ok(serde_json::json!({
            "system_tray_enabled": settings.system_tray_enabled,
            "silent_start_enabled": settings.silent_start_enabled,
            "debugMode": settings.debug_mode,
            "privateMode": settings.private_mode,
            "language": settings.language
        }))
    })
}

/// 获取语言偏好设置
#[tauri::command]
pub async fn get_language(app: AppHandle) -> Result<String, String> {
    crate::log_async_command!("get_language", async {
        let settings_manager = app.state::<crate::app_settings::AppSettingsManager>();
        let settings = settings_manager.get_settings();
        Ok(settings.language.clone())
    })
}

/// 保存语言偏好设置
#[tauri::command]
pub async fn set_language(app: AppHandle, language: String) -> Result<(), String> {
    crate::log_async_command!("set_language", async {
        // Validate language code
        let valid_languages = vec!["en", "zh-CN", "zh-TW"];
        if !valid_languages.contains(&language.as_str()) {
            return Err(format!("Unsupported language: {}", language));
        }

        let settings_manager = app.state::<crate::app_settings::AppSettingsManager>();
        settings_manager.update_settings(|settings| {
            settings.language = language.clone();
        })?;

        tracing::info!("Language preference saved: {}", language);
        Ok(())
    })
}
