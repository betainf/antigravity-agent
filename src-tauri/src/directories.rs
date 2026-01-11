use serde_json::{self, Value};
/// 目录获取模块
/// 统一管理所有配置和数据目录路径
use std::fs;
use std::io;
use std::path::PathBuf;
use tracing::{info, warn};

/// 获取应用主配置目录
/// 所有配置、日志、数据都统一存放在用户主目录的 .antigravity-agent 下
#[cfg(windows)]
pub fn get_config_directory() -> PathBuf {
    let config_dir = dirs::home_dir()
        .expect("Home directory not found")
        .join(".antigravity-agent");

    // 确保目录存在
    if let Err(e) = fs::create_dir_all(&config_dir) {
        eprintln!("警告：无法创建配置目录 {}: {}", config_dir.display(), e);
    }

    config_dir
}

/// 获取应用主配置目录
#[cfg(not(windows))]
pub fn get_config_directory() -> PathBuf {
    let config_dir = dirs::home_dir()
        .expect("Home directory not found")
        .join(".antigravity-agent");

    // 确保目录存在
    if let Err(e) = fs::create_dir_all(&config_dir) {
        eprintln!("警告：无法创建配置目录 {}: {}", config_dir.display(), e);
    }

    config_dir
}

/// 获取日志目录路径
#[cfg(windows)]
pub fn get_log_directory() -> PathBuf {
    get_config_directory().join("logs")
}

/// 获取日志目录路径
#[cfg(not(windows))]
pub fn get_log_directory() -> PathBuf {
    get_config_directory().join("logs")
}

/// 获取账户备份目录
pub fn get_accounts_directory() -> PathBuf {
    let accounts_dir = get_config_directory().join("antigravity-accounts");

    // 确保目录存在
    if let Err(e) = fs::create_dir_all(&accounts_dir) {
        eprintln!("警告：无法创建账户目录 {}: {}", accounts_dir.display(), e);
    }

    accounts_dir
}

/// 获取应用设置文件路径
pub fn get_app_settings_file() -> PathBuf {
    get_config_directory().join("app_settings.json")
}

/// 获取窗口状态文件路径
pub fn get_window_state_file() -> PathBuf {
    get_config_directory().join("window_state.json")
}

/// 获取 Antigravity 路径配置文件路径
pub fn get_antigravity_path_file() -> PathBuf {
    get_config_directory().join("antigravity_path.json")
}

/// 在应用启动时检查并迁移旧账户目录到新路径。
/// 当前为空实现，后续补充实际迁移逻辑。
pub fn migrate_legacy_accounts_if_needed() -> io::Result<()> {
    let new_config_dir = get_config_directory();
    let new_accounts_dir = get_accounts_directory();
    info!(
        target: "app::startup",
        "当前配置目录: {}",
        new_config_dir.display()
    );

    // 旧账户目录（Roaming 配置目录下）
    let Some(config_dir) = dirs::config_dir() else {
        info!(target: "app::startup", "未找到系统配置目录 (dirs::config_dir)，跳过旧账户目录检测");
        return Ok(());
    };
    let legacy_accounts_dir = config_dir
        .join(".antigravity-agent")
        .join("antigravity-accounts");
    info!(
        target: "app::startup",
        "检测旧账户目录: {}",
        legacy_accounts_dir.display()
    );

    // 收集旧账户目录下的 JSON 文件列表（忽略子目录）
    let mut legacy_files = Vec::new();
    let read_dir = match fs::read_dir(&legacy_accounts_dir) {
        Ok(rd) => rd,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(err),
    };
    for entry in read_dir {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if path
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s.eq_ignore_ascii_case("json"))
                .unwrap_or(false)
            {
                legacy_files.push(path);
            }
        }
    }
    info!(
        target: "app::startup",
        "旧账户目录文件数: {}",
        legacy_files.len()
    );

    // 读取 JSON，并只保留 jetskiStateSync.agentManagerInitState 字段（作为单一键存储）
    let mut legacy_account_states: Vec<Value> = Vec::new();
    let mut migrated_count = 0usize;
    let mut skipped_existing = 0usize;
    let mut renamed_count = 0usize;
    for json_path in legacy_files {
        let file_name = json_path.file_name().map(|f| f.to_owned());

        if let Some(file_name) = file_name {
            match fs::read_to_string(&json_path) {
                Ok(content) => {
                    match serde_json::from_str::<Value>(&content) {
                        Ok(v) => {
                            // 仅使用顶层键 jetskiStateSync.agentManagerInitState
                            if let Some(state) = v.get("jetskiStateSync.agentManagerInitState") {
                                let mut filtered = serde_json::Map::new();
                                filtered.insert(
                                    "jetskiStateSync.agentManagerInitState".to_string(),
                                    state.clone(),
                                );
                                let filtered_value = Value::Object(filtered);
                                legacy_account_states.push(filtered_value.clone());

                                // 目标写入路径
                                let new_path = new_accounts_dir.join(&file_name);

                                if new_path.exists() {
                                    warn!(
                                        target: "app::startup",
                                        "新目录已存在同名文件，跳过写入: {}",
                                        new_path.display()
                                    );
                                    skipped_existing += 1;
                                } else {
                                    match serde_json::to_string_pretty(&filtered_value) {
                                        Ok(serialized) => {
                                            if let Err(e) = fs::write(&new_path, serialized) {
                                                warn!(
                                                    target: "app::startup",
                                                    "写入新文件失败: {}，错误: {}",
                                                    new_path.display(),
                                                    e
                                                );
                                            } else {
                                                migrated_count += 1;
                                            }
                                        }
                                        Err(e) => {
                                            warn!(
                                                target: "app::startup",
                                                "序列化 JSON 失败: {}，错误: {}",
                                                json_path.display(),
                                                e
                                            );
                                        }
                                    }
                                }
                            } else {
                                warn!(
                                    target: "app::startup",
                                    "未找到 jetskiStateSync.agentManagerInitState，文件已忽略: {}",
                                    json_path.display()
                                );
                            }
                        }
                        Err(e) => {
                            warn!(
                                target: "app::startup",
                                "解析 JSON 失败（忽略此文件）: {}，错误: {}",
                                json_path.display(),
                                e
                            );
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        target: "app::startup",
                        "读取文件失败（忽略此文件）: {}，错误: {}",
                        json_path.display(),
                        e
                    );
                }
            }
        } else {
            warn!(
                target: "app::startup",
                "文件名无效，跳过: {}",
                json_path.display()
            );
        }

        // 无论写入/解析是否成功，都尝试将旧文件改为 .bak
        let bak_path = json_path.with_extension("bak");
        match fs::rename(&json_path, &bak_path) {
            Ok(()) => {
                renamed_count += 1;
            }
            Err(e) => {
                warn!(
                    target: "app::startup",
                    "重命名旧文件为 .bak 失败: {} -> {}，错误: {}",
                    json_path.display(),
                    bak_path.display(),
                    e
                );
            }
        }
    }
    info!(
        target: "app::startup",
        "提取到旧账户状态数: {}，成功迁移: {}，同名跳过: {}，已重命名为 .bak: {}",
        legacy_account_states.len(),
        migrated_count,
        skipped_existing,
        renamed_count
    );

    // TODO: 检测 Roaming 下的旧账户目录并迁移到新目录，处理冲突和错误。
    Ok(())
}
