// Antigravity 用户数据恢复模块
// 负责将备份数据恢复到 Antigravity 应用数据库

use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::fs;

// 导入 platform_utils 模块 (需要在 main.rs 中声明为 pub mod)
use crate::platform_utils;

/// 通用数据库恢复方法
///
/// 执行精确的数据库恢复操作：
/// 1. 恢复认证信息 (antigravityAuthStatus)
/// 2. 恢复用户头像 (antigravity.profileUrl)
/// 3. 恢复用户设置 (antigravityUserSettings.allUserSettings)
/// 4. 恢复校验标记 (__$__targetStorageMarker)
/// 5. 重置分析时间戳 (antigravityAnalytics.lastUploadTime)
///
/// # 参数
/// - `db_path`: 数据库文件路径
/// - `db_name`: 数据库名称（用于日志显示）
/// - `backup_data`: 备份数据的 JSON 对象
///
/// # 返回
/// - `Ok(restored_count)`: 成功恢复的项目数量
/// - `Err(message)`: 错误信息
fn restore_database(
    db_path: &Path,
    db_name: &str,
    backup_data: &serde_json::Value
) -> Result<usize, String> {
    println!("🔄 恢复数据库: {}", db_name);

    let conn = Connection::open(db_path)
        .map_err(|e| format!("连接{}失败: {}", db_name, e))?;

    let mut restored_count = 0;

    // 1. 恢复认证信息
    if let Some(auth_status) = backup_data.get("auth_status") {
        if let Some(auth_str) = auth_status.as_str() {
            conn.execute(
                "INSERT OR REPLACE INTO ItemTable (key, value) VALUES ('antigravityAuthStatus', ?)",
                [auth_str],
            )
            .map_err(|e| format!("恢复认证信息失败: {}", e))?;

            println!("  ✅ 已恢复: antigravityAuthStatus");
            restored_count += 1;
        }
    }

    // 2. 恢复头像
    if let Some(profile_url) = backup_data.get("profile_url") {
        if let Some(url_str) = profile_url.as_str() {
            conn.execute(
                "INSERT OR REPLACE INTO ItemTable (key, value) VALUES ('antigravity.profileUrl', ?)",
                [url_str],
            )
            .map_err(|e| format!("恢复头像失败: {}", e))?;

            println!("  ✅ 已恢复: antigravity.profileUrl");
            restored_count += 1;
        }
    }

    // 3. 恢复用户设置
    if let Some(user_settings) = backup_data.get("user_settings") {
        if let Some(settings_str) = user_settings.as_str() {
            conn.execute(
                "INSERT OR REPLACE INTO ItemTable (key, value) VALUES ('antigravityUserSettings.allUserSettings', ?)",
                [settings_str],
            )
            .map_err(|e| format!("恢复用户设置失败: {}", e))?;

            println!("  ✅ 已恢复: antigravityUserSettings.allUserSettings");
            restored_count += 1;
        }
    }

    // 4. 恢复校验标记（从备份中动态获取）
    if let Some(target_marker) = backup_data.get("target_storage_marker") {
        if let Some(marker_str) = target_marker.as_str() {
            conn.execute(
                "INSERT OR REPLACE INTO ItemTable (key, value) VALUES ('__$__targetStorageMarker', ?)",
                [marker_str],
            )
            .map_err(|e| format!("恢复校验标记失败: {}", e))?;

            println!("  ✅ 已恢复: __$__targetStorageMarker");
            restored_count += 1;
        } else {
            println!("  ℹ️ 备份中无校验标记，跳过");
        }
    } else {
        println!("  ℹ️ 备份中无校验标记字段，跳过");
    }

    // 5. 重置分析时间戳（避免数据冲突）
    conn.execute(
        "INSERT OR REPLACE INTO ItemTable (key, value) VALUES ('antigravityAnalytics.lastUploadTime', '0')",
        [],
    )
    .map_err(|e| format!("重置分析时间戳失败: {}", e))?;

    println!("  ✅ 已重置分析时间戳");

    drop(conn);
    Ok(restored_count)
}

/// 恢复 Antigravity 的用户认证数据（完整恢复）
///
/// 从备份文件恢复用户数据到数据库：
/// - 恢复认证信息 (antigravityAuthStatus)
/// - 恢复用户头像 (antigravity.profileUrl)
/// - 恢复用户设置 (antigravityUserSettings.allUserSettings)
/// - 恢复校验标记 (__$__targetStorageMarker)
/// - 重置分析时间戳 (antigravityAnalytics.lastUploadTime)
///
/// 同时处理主数据库和备份数据库，保持数据一致性
///
/// # 参数
/// - `backup_file_path`: 备份 JSON 文件的完整路径
///
/// # 返回
/// - `Ok(message)`: 成功消息
/// - `Err(message)`: 错误信息
pub async fn restore_all_antigravity_data(
    backup_file_path: PathBuf
) -> Result<String, String> {
    println!("🔄 开始恢复 Antigravity 用户认证数据");
    println!("📂 备份文件: {}", backup_file_path.display());

    // 1. 读取备份文件
    if !backup_file_path.exists() {
        return Err(format!("备份文件不存在: {}", backup_file_path.display()));
    }

    let backup_content = fs::read_to_string(&backup_file_path)
        .map_err(|e| format!("读取备份文件失败: {}", e))?;

    let backup_data: serde_json::Value = serde_json::from_str(&backup_content)
        .map_err(|e| format!("解析备份数据失败: {}", e))?;

    println!("✅ 备份文件读取成功");

    // 2. 获取 Antigravity 数据库路径
    let app_data = match platform_utils::get_antigravity_db_path() {
        Some(path) => path,
        None => {
            let possible_paths = platform_utils::get_all_antigravity_db_paths();
            if possible_paths.is_empty() {
                return Err("未找到Antigravity安装位置".to_string());
            }
            possible_paths[0].clone()
        }
    };

    // 确保数据库目录存在
    if let Some(parent) = app_data.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("创建数据库目录失败: {}", e))?;
    }

    let mut restored_items = Vec::new();

    // 3. 恢复主数据库 (state.vscdb)
    println!("📊 步骤1: 恢复 state.vscdb 数据库");
    match restore_database(&app_data, "state.vscdb", &backup_data) {
        Ok(count) => {
            println!("  ✅ 主数据库已恢复 {} 项", count);
            restored_items.push(format!("state.vscdb({} 项)", count));
        }
        Err(e) => {
            return Err(format!("恢复主数据库失败: {}", e));
        }
    }

    // 4. 恢复备份数据库 (state.vscdb.backup) - 同步
    println!("💾 步骤2: 恢复 state.vscdb.backup");
    let backup_db_path = app_data.with_extension("vscdb.backup");
    if backup_db_path.exists() {
        match restore_database(&backup_db_path, "state.vscdb.backup", &backup_data) {
            Ok(count) => {
                println!("  ✅ 备份数据库已恢复 {} 项", count);
                restored_items.push(format!("state.vscdb.backup({} 项)", count));
            }
            Err(e) => {
                println!("  ⚠️ 恢复备份数据库失败: {}", e);
                // 备份数据库失败不中断流程
            }
        }
    } else {
        println!("  ℹ️ 备份数据库不存在，跳过");
    }

    Ok(format!(
        "✅ 已恢复 {} 个数据库\n恢复详情: {}",
        restored_items.len(),
        restored_items.join(", ")
    ))
}
