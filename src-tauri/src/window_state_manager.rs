// 窗口状态管理模块
// 负责保存和恢复应用程序窗口状态

use std::path::PathBuf;
use std::fs;
use serde::{Deserialize, Serialize};

// 窗口状态结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub maximized: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            x: 100.0,
            y: 100.0,
            width: 800.0,
            height: 600.0,
            maximized: false,
        }
    }
}

/// 保存窗口状态
pub async fn save_window_state(state: WindowState) -> Result<(), String> {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".antigravity-agent");

    fs::create_dir_all(&config_dir)
        .map_err(|e| format!("创建配置目录失败: {}", e))?;

    let state_file = config_dir.join("window_state.json");
    let json_content = serde_json::to_string(&state)
        .map_err(|e| format!("序列化窗口状态失败: {}", e))?;

    fs::write(state_file, json_content)
        .map_err(|e| format!("保存窗口状态失败: {}", e))?;

    println!("💾 窗口状态已保存: 位置({:.1}, {:.1}), 大小({:.1}x{:.1}), 最大化:{}",
             state.x, state.y, state.width, state.height, state.maximized);

    Ok(())
}

/// 加载窗口状态
pub async fn load_window_state() -> Result<WindowState, String> {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".antigravity-agent");

    let state_file = config_dir.join("window_state.json");

    if state_file.exists() {
        let content = fs::read_to_string(&state_file)
            .map_err(|e| format!("读取窗口状态文件失败: {}", e))?;

        let state: WindowState = serde_json::from_str(&content)
            .map_err(|e| format!("解析窗口状态失败: {}", e))?;

        println!("📄 成功加载窗口状态: 位置({:.1}, {:.1}), 大小({:.1}x{:.1}), 最大化:{}",
                 state.x, state.y, state.width, state.height, state.maximized);

        Ok(state)
    } else {
        println!("📄 窗口状态文件不存在，使用默认状态");
        Ok(WindowState::default())
    }
}