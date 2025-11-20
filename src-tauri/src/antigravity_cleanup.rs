// Antigravity 用户数据清除模块
// 负责清除 Antigravity 应用的所有用户认证和设置信息

use rusqlite::Connection;
use std::path::{Path, PathBuf};

// 导入 platform_utils 模块 (需要在 main.rs 中声明为 pub mod)
use crate::platform_utils;

/// 未登录状态的校验标记值（从对比 ItemTable.json 和 ItemTable_logined.json 得到）
const RESET_MARKER_VALUE: &str = r#"{"jetskiStateSync.agentManagerInitState":1,"history.recentlyOpenedPathsList":1,"antigravityUserSettings.allUserSettings":1,"workbench.view.debug.state.hidden":0,"workbench.activity.pinnedViewlets2":0,"workbench.activity.placeholderViewlets":1,"workbench.view.remote.state.hidden":0,"editorGroupAntigravityWelcomeKeybindings":0,"memento/notebookEditors":1,"memento/customEditors":1,"productIconThemeData":1,"colorThemeData":0,"iconThemeData":1,"workbench.panel.pinnedPanels":0,"workbench.panel.placeholderPanels":1,"~remote.forwardedPortsContainer.hidden":0,"workbench.telemetryOptOutShown":0,"releaseNotes/lastVersion":1,"perf/lastRunningCommit":1,"workbench.sideBar.size":1,"workbench.auxiliaryBar.size":1,"workbench.panel.size":1,"workbench.panel.lastNonMaximizedHeight":1,"workbench.panel.lastNonMaximizedWidth":1,"workbench.auxiliaryBar.lastNonMaximizedSize":1,"workbench.auxiliaryBar.empty":1,"workbench.panel.alignment":0,"chat.ChatSessionStore.index":1,"workbench.panel.repl.hidden":0,"google.antigravity":1,"extensions.trustedPublishers":0,"trusted-publishers-init-migration":1,"remote.wslFeatureInstalled":1,"chat.participantNameRegistry":1,"extensionTips/lastPromptedMediumImpExeTime":1,"vscode.typescript-language-features":1,"editorFontInfo":1,"workbench.activityBar.location":0,"antigravityChangelog/lastVersion":1,"sync.productQuality":1,"terminal.history.entries.dirs":1,"terminal.history.timestamp.dirs":1,"antigravityAuthStatus":0,"antigravity_allowed_command_model_configs":0,"antigravityOnboarding":0,"workbench.quickInput.viewState":1,"workbench.explorer.views.state.hidden":0,"chat.workspaceTransfer":1,"vscode.git":1,"vscode.github":1,"content.trust.model.key":1,"extensionsAssistant/recommendations":1,"editorOverrideService.cache":1,"workbench.editor.languageDetectionOpenedLanguages.global":1}"#;

/// 需要删除的认证相关字段
const DELETE_KEYS: &[&str] = &[
    "antigravityAuthStatus",    // 认证状态
    "antigravity.profileUrl",   // 用户头像
    "antigravityOnboarding"     // 新手引导标记
];

/// 通用数据库清理方法
///
/// 执行精确的数据库操作：
/// 1. 删除认证信息、头像和新手引导标记
/// 2. 重置校验标记为未登录状态
///
/// # 参数
/// - `db_path`: 数据库文件路径
/// - `db_name`: 数据库名称（用于日志显示）
///
/// # 返回
/// - `Ok(cleared_count)`: 成功清除的项目数量
/// - `Err(message)`: 错误信息
fn clear_database(db_path: &Path, db_name: &str) -> Result<usize, String> {
    let conn = Connection::open(db_path)
        .map_err(|e| format!("连接{}失败: {}", db_name, e))?;

    let mut cleared_count = 0;

    // 1. 删除认证信息、头像和新手引导标记
    for key in DELETE_KEYS {
        match conn.execute(
            "DELETE FROM ItemTable WHERE key = ?",
            [key],
        ) {
            Ok(rows) if rows > 0 => {
                println!("  ✅ 已删除: {}", key);
                cleared_count += 1;
            }
            Ok(_) => {
                println!("  ℹ️ 未找到: {}", key);
            }
            Err(e) => {
                println!("  ⚠️ 删除失败 {}: {}", key, e);
            }
        }
    }

    // 2. 重置校验标记为未登录状态
    match conn.execute(
        "UPDATE ItemTable SET value = ? WHERE key = '__$__targetStorageMarker'",
        [RESET_MARKER_VALUE],
    ) {
        Ok(rows) if rows > 0 => {
            println!("  ✅ 已重置校验标记");
            cleared_count += 1;
        }
        Ok(_) => {
            println!("  ℹ️ 未找到校验标记，跳过");
        }
        Err(e) => {
            println!("  ⚠️ 重置校验标记失败: {}", e);
        }
    }

    drop(conn);
    Ok(cleared_count)
}

/// 清除 Antigravity 的用户认证数据（精确登出）
///
/// 通过精确的数据库操作实现登出效果：
/// - 删除认证信息 (antigravityAuthStatus)
/// - 删除用户头像 (antigravity.profileUrl)
/// - 删除新手引导标记 (antigravityOnboarding)
/// - 重置校验标记 (__$__targetStorageMarker) 为未登录状态
///
/// 这种方式保留了所有配置文件，只清除用户身份信息
/// 建议在"登录新账户"时使用
pub async fn clear_all_antigravity_data() -> Result<String, String> {
    println!("🗑️ 开始清除 Antigravity 用户认证数据");
    let mut cleared_items = Vec::new();

    // 1. 获取主数据库路径
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

    if !app_data.exists() {
        return Err(format!("Antigravity 状态数据库不存在: {}", app_data.display()));
    }

    // 2. 清除主数据库 (state.vscdb)
    println!("📊 步骤1: 清除 state.vscdb 数据库");
    match clear_database(&app_data, "state.vscdb") {
        Ok(count) => {
            println!("  ✅ 主数据库已清除 {} 项", count);
            cleared_items.push(format!("state.vscdb({} 项)", count));
        }
        Err(e) => {
            return Err(format!("清除主数据库失败: {}", e));
        }
    }

    // 3. 清除备份数据库 (state.vscdb.backup)
    println!("💾 步骤2: 清除 state.vscdb.backup");
    let backup_db_path = app_data.with_extension("vscdb.backup");
    if backup_db_path.exists() {
        match clear_database(&backup_db_path, "state.vscdb.backup") {
            Ok(count) => {
                println!("  ✅ 备份数据库已清除 {} 项", count);
                cleared_items.push(format!("state.vscdb.backup({} 项)", count));
            }
            Err(e) => {
                println!("  ⚠️ 清除备份数据库失败: {}", e);
            }
        }
    } else {
        println!("  ℹ️ 备份数据库不存在，跳过");
    }

    Ok(format!(
        "✅ 已清除 {} 个数据库，保留了所有配置文件\n清除详情: {}",
        cleared_items.len(),
        cleared_items.join(", ")
    ))
}
