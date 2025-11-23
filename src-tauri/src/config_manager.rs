use crate::constants::paths;
/// 配置管理器
/// 统一管理所有配置目录和文件路径
use std::fs;
use std::path::PathBuf;

/// 配置管理器结构
pub struct ConfigManager {
    config_dir: PathBuf,
}

impl ConfigManager {
    /// 创建新的配置管理器
    pub fn new() -> Result<Self, String> {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(paths::CONFIG_DIR_NAME);

        fs::create_dir_all(&config_dir).map_err(|e| format!("创建配置目录失败: {}", e))?;

        Ok(Self { config_dir })
    }

    /// 获取窗口状态文件路径
    pub fn window_state_file(&self) -> PathBuf {
        self.config_dir.join(paths::WINDOW_STATE_FILE)
    }

    /// 获取应用设置文件路径
    pub fn app_settings_file(&self) -> PathBuf {
        self.config_dir.join("app_settings.json")
    }
}
