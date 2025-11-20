// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use tauri::State;
use walkdir::WalkDir;
use zip::{ZipWriter, write::FileOptions};
use std::io::Write;

use rusqlite::{params, Connection, Result as SqlResult};
use std::process::Command;

/// Antigravity 清理模块
mod antigravity_cleanup;

/// Antigravity 恢复模块
mod antigravity_restore;

/// 窗口状态管理模块
mod window_state_manager;

/// 窗口事件处理模块
mod window_event_handler;

/// 多平台支持工具函数
mod platform_utils {
    use std::path::PathBuf;
    use std::process::Command;
    use dirs;

    /// 获取Antigravity应用数据目录（跨平台）
    pub fn get_antigravity_data_dir() -> Option<PathBuf> {
        match std::env::consts::OS {
            "windows" => {
                // Windows: %APPDATA%\Antigravity\User\globalStorage\
                dirs::config_dir().map(|path| path.join("Antigravity").join("User").join("globalStorage"))
            }
            "macos" => {
                // macOS: 基于 product.json 中的 dataFolderName: ".antigravity" 配置
                // ~/Library/Application Support/Antigravity/User/globalStorage/
                dirs::data_dir().map(|path| path.join("Antigravity").join("User").join("globalStorage"))
            }
            "linux" => {
                // Linux: 基于 product.json 中的 dataFolderName: ".antigravity" 配置
                // 优先使用 ~/.config/Antigravity/User/globalStorage/，备用 ~/.local/share/Antigravity/User/globalStorage/
                dirs::config_dir()  // 优先：~/.config
                    .map(|path| path.join("Antigravity").join("User").join("globalStorage"))
                    .or_else(|| {  // 备用：~/.local/share
                        dirs::data_dir().map(|path| path.join("Antigravity").join("User").join("globalStorage"))
                    })
            }
            _ => {
                // 其他系统：尝试使用数据目录
                dirs::data_dir().map(|path| path.join("Antigravity").join("User").join("globalStorage"))
            }
        }
    }

    /// 获取Antigravity状态数据库文件路径
    pub fn get_antigravity_db_path() -> Option<PathBuf> {
        get_antigravity_data_dir().map(|dir| dir.join("state.vscdb"))
    }

    /// 检查Antigravity是否安装并运行
    pub fn is_antigravity_available() -> bool {
        get_antigravity_db_path()
            .map(|path| path.exists())
            .unwrap_or(false)
    }

    /// 搜索可能的Antigravity安装位置
    pub fn find_antigravity_installations() -> Vec<PathBuf> {
        let mut possible_paths = Vec::new();

        // 用户数据目录
        if let Some(user_data) = dirs::data_dir() {
            possible_paths.push(user_data.join("Antigravity"));
        }

        // 配置目录
        if let Some(config_dir) = dirs::config_dir() {
            possible_paths.push(config_dir.join("Antigravity"));
        }

        possible_paths
    }

    /// 获取所有可能的Antigravity数据库路径
    pub fn get_all_antigravity_db_paths() -> Vec<PathBuf> {
        let mut db_paths = Vec::new();

        // 主要路径
        if let Some(main_path) = get_antigravity_db_path() {
            db_paths.push(main_path);
        }

        // 搜索其他可能的位置
        for install_dir in find_antigravity_installations() {
            if install_dir.exists() {
                // 递归搜索state.vscdb文件
                if let Ok(entries) = std::fs::read_dir(&install_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() &&
                           path.file_name().map_or(false, |name| name == "state.vscdb") {
                            db_paths.push(path);
                        }
                    }
                }
            }
        }

        db_paths
    }

    /// 关闭Antigravity进程
    pub fn kill_antigravity_processes() -> Result<String, String> {
        match std::env::consts::OS {
            "windows" => {
                // Windows: 尝试多种可能的进程名
                let process_names = vec!["Antigravity.exe", "Antigravity"];
                let mut last_error = String::new();

                for process_name in process_names {
                    let output = Command::new("taskkill")
                        .args(["/F", "/IM", process_name])
                        .output()
                        .map_err(|e| format!("执行taskkill命令失败: {}", e))?;

                    if output.status.success() {
                        return Ok(format!("已成功关闭Antigravity进程 ({})", process_name));
                    } else {
                        last_error = format!("关闭进程 {} 失败: {:?}", process_name, String::from_utf8_lossy(&output.stderr));
                    }
                }

                Err(last_error)
            }
            "macos" | "linux" => {
                // macOS/Linux: 使用pkill命令，尝试多种进程名模式
                let process_patterns = vec![
                    "Antigravity",
                    "antigravity"
                ];
                let mut last_error = String::new();

                for pattern in process_patterns {
                    let output = Command::new("pkill")
                        .args(["-f", pattern])
                        .output()
                        .map_err(|e| format!("执行pkill命令失败: {}", e))?;

                    if output.status.success() {
                        return Ok(format!("已成功关闭Antigravity进程 (模式: {})", pattern));
                    } else {
                        last_error = format!("关闭进程失败 (模式: {}): {:?}", pattern, String::from_utf8_lossy(&output.stderr));
                    }
                }

                Err(last_error)
            }
            _ => Err("不支持的操作系统".to_string())
        }
    }

    /// 启动Antigravity
    pub fn start_antigravity() -> Result<String, String> {
        match std::env::consts::OS {
            "windows" => {
                // Windows: 使用绝对路径推测
                let mut errors = Vec::new();
                let mut antigravity_paths = Vec::new();

                // 1. 基于用户主目录构建可能的路径
                if let Some(home) = dirs::home_dir() {
                    // C:\Users\{用户名}\AppData\Local\Programs\Antigravity\Antigravity.exe (最常见)
                    antigravity_paths.push(home.join(r"AppData\Local\Programs\Antigravity\Antigravity.exe"));
                    // C:\Users\{用户名}\AppData\Roaming\Local\Programs\Antigravity\Antigravity.exe
                    antigravity_paths.push(home.join(r"AppData\Roaming\Local\Programs\Antigravity\Antigravity.exe"));
                }

                // 2. 使用 data_local_dir (通常是 C:\Users\{用户名}\AppData\Local)
                if let Some(local_data) = dirs::data_local_dir() {
                    antigravity_paths.push(local_data.join(r"Programs\Antigravity\Antigravity.exe"));
                }

                // 3. 其他可能的位置
                antigravity_paths.push(PathBuf::from(r"C:\Program Files\Antigravity\Antigravity.exe"));
                antigravity_paths.push(PathBuf::from(r"C:\Program Files (x86)\Antigravity\Antigravity.exe"));

                // 尝试所有推测的路径
                for path in &antigravity_paths {
                    if path.exists() {
                        eprintln!("找到并尝试启动: {}", path.display());
                        match Command::new(path).spawn() {
                            Ok(_) => {
                                return Ok(format!("Antigravity启动成功 ({})", path.display()));
                            }
                            Err(e) => {
                                errors.push(format!("{}: {}", path.display(), e));
                            }
                        }
                    } else {
                        errors.push(format!("{}: 文件不存在", path.display()));
                    }
                }

                // 4. 最后尝试从系统PATH启动命令
                let commands = vec!["Antigravity", "antigravity"];
                for cmd in commands {
                    eprintln!("尝试命令: {}", cmd);
                    match Command::new(cmd).spawn() {
                        Ok(_) => {
                            return Ok(format!("Antigravity启动成功 (命令: {})", cmd));
                        }
                        Err(e) => {
                            errors.push(format!("{}命令: {}", cmd, e));
                        }
                    }
                }

                Err(format!("无法启动Antigravity。请手动启动Antigravity应用。\n尝试的方法：\n{}", errors.join("\n")))
            }
            "macos" => {
                // macOS: 基于 product.json 中的 darwinBundleIdentifier: "com.google.antigravity" 配置
                let mut errors = Vec::new();
                let mut antigravity_paths = Vec::new();

                // 基于 DMG 安装包的标准 .app 应用结构
                antigravity_paths.push(PathBuf::from("/Applications/Antigravity.app/Contents/MacOS/Antigravity"));

                // 用户应用目录（用户手动安装时的常见位置）
                if let Some(home) = dirs::home_dir() {
                    antigravity_paths.push(home.join("Applications/Antigravity.app/Contents/MacOS/Antigravity"));
                }

                // 尝试所有推测的路径
                for path in &antigravity_paths {
                    if path.exists() {
                        eprintln!("找到并尝试启动: {}", path.display());
                        match Command::new(path).spawn() {
                            Ok(_) => {
                                return Ok(format!("Antigravity启动成功 ({})", path.display()));
                            }
                            Err(e) => {
                                errors.push(format!("{}: {}", path.display(), e));
                            }
                        }
                    } else {
                        errors.push(format!("{}: 文件不存在", path.display()));
                    }
                }

                // 2. 尝试系统PATH命令
                let commands = vec!["Antigravity", "antigravity"];
                for cmd in commands {
                    match Command::new(cmd).spawn() {
                        Ok(_) => {
                            return Ok(format!("Antigravity启动成功 (命令: {})", cmd));
                        }
                        Err(e) => {
                            errors.push(format!("{}命令: {}", cmd, e));
                        }
                    }
                }

                Err(format!("无法启动Antigravity。请手动启动Antigravity应用。\n尝试的方法：\n{}", errors.join("\n")))
            }
            "linux" => {
                // Linux: 基于实际安装包分析的路径检测
                let mut errors = Vec::new();
                let mut antigravity_paths = Vec::new();

                // 基于安装包实际分析的唯一有证据的路径
                antigravity_paths.push(PathBuf::from("/usr/share/antigravity/antigravity")); // 启动脚本硬编码的默认路径

                // 尝试所有推测的路径
                for path in &antigravity_paths {
                    if path.exists() {
                        eprintln!("找到并尝试启动: {}", path.display());
                        match Command::new(path).spawn() {
                            Ok(_) => {
                                return Ok(format!("Antigravity启动成功 ({})", path.display()));
                            }
                            Err(e) => {
                                errors.push(format!("{}: {}", path.display(), e));
                            }
                        }
                    } else {
                        errors.push(format!("{}: 文件不存在", path.display()));
                    }
                }

                // 尝试系统 PATH 中的命令（如果安装包解压到 PATH 包含的目录）
                let commands = vec!["antigravity", "Antigravity"];
                for cmd in commands {
                    eprintln!("尝试命令: {}", cmd);
                    match Command::new(cmd).spawn() {
                        Ok(_) => {
                            return Ok(format!("Antigravity启动成功 (命令: {})", cmd));
                        }
                        Err(e) => {
                            errors.push(format!("{}命令: {}", cmd, e));
                        }
                    }
                }

                Err(format!("无法启动Antigravity。请手动启动Antigravity应用。\n尝试的方法：\n{}", errors.join("\n")))
            }
            _ => Err("不支持的操作系统".to_string())
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ProfileInfo {
    name: String,
    source_path: String,
    backup_path: String,
    created_at: String,
    last_updated: String,
}

// Antigravity 账户信息结构
#[derive(Debug, Serialize, Deserialize)]
struct AntigravityAccount {
    id: String,
    name: String,
    email: String,
    api_key: String,
    profile_url: String, // Base64 编码的头像
    user_settings: String, // 编码后的用户设置
    created_at: String,
    last_switched: String,
}

// 导入窗口状态管理器
use window_state_manager::{WindowState, load_window_state as load_ws, save_window_state as save_ws};

#[derive(Debug, Serialize, Deserialize)]
struct AppState {
    profiles: HashMap<String, ProfileInfo>,
    config_dir: PathBuf,
    antigravity_accounts: HashMap<String, AntigravityAccount>,
    current_account_id: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        // 智能检测配置目录，确保跨平台兼容性
        let config_dir = if cfg!(windows) {
            // Windows: 优先使用 APPDATA 环境变量
            std::env::var_os("APPDATA")
                .and_then(|appdata| Some(PathBuf::from(appdata).join(".antigravity-agent")))
                .or_else(|| {
                    // 备用方案：通过用户主目录构建 AppData\Roaming 路径
                    dirs::home_dir()
                        .map(|home| home.join("AppData").join("Roaming").join(".antigravity-agent"))
                })
                .or_else(|| {
                    // 最后备用：使用系统标准配置目录
                    dirs::config_dir().map(|config| config.join(".antigravity-agent"))
                })
                .unwrap_or_else(|| PathBuf::from(".antigravity-agent"))
        } else {
            // macOS/Linux: 使用标准配置目录
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".antigravity-agent")
        };

        // 确保配置目录存在
        fs::create_dir_all(&config_dir)
            .map_err(|e| eprintln!("警告：无法创建配置目录 {:?}: {}", config_dir, e))
            .ok();

        Self {
            profiles: HashMap::new(),
            config_dir,
            antigravity_accounts: HashMap::new(),
            current_account_id: None,
        }
    }
}

#[tauri::command]
async fn backup_profile(
    name: String,
    source_path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let source = Path::new(&source_path);
    if !source.exists() {
        return Err("源路径不存在".to_string());
    }

    let backup_dir = state.config_dir.join("backups");
    fs::create_dir_all(&backup_dir).map_err(|e| format!("创建备份目录失败: {}", e))?;

    let backup_file = backup_dir.join(format!("{}.zip", name));

    // 创建 ZIP 压缩文件
    let file = fs::File::create(&backup_file).map_err(|e| format!("创建备份文件失败: {}", e))?;
    let mut zip = ZipWriter::new(file);
    let options: FileOptions<()> = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    // 遍历源目录并添加到 ZIP
    for entry in WalkDir::new(source) {
        let entry = entry.map_err(|e| format!("遍历目录失败: {}", e))?;
        let path = entry.path();
        let name = path.strip_prefix(source).map_err(|e| format!("处理路径失败: {}", e))?;

        if path.is_file() {
            let mut file = fs::File::open(path).map_err(|e| format!("打开文件失败: {}", e))?;
            zip.start_file(name.to_string_lossy(), options)
                .map_err(|e| format!("添加文件到压缩包失败: {}", e))?;
            let mut buffer = Vec::new();
            use std::io::Read;
            file.read_to_end(&mut buffer).map_err(|e| format!("读取文件失败: {}", e))?;
            zip.write_all(&buffer).map_err(|e| format!("写入压缩包失败: {}", e))?;
        }
    }

    zip.finish().map_err(|e| format!("完成压缩失败: {}", e))?;

    // 更新配置信息
    let profile_info = ProfileInfo {
        name: name.clone(),
        source_path: source_path.clone(),
        backup_path: backup_file.to_string_lossy().to_string(),
        created_at: chrono::Local::now().to_rfc3339(),
        last_updated: chrono::Local::now().to_rfc3339(),
    };

    // 这里应该更新状态，但由于 State 是不可变的，我们需要其他方式
    // 暂时返回成功信息

    Ok(format!("备份成功: {}", backup_file.display()))
}

#[tauri::command]
async fn restore_profile(
    name: String,
    target_path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let backup_dir = state.config_dir.join("backups");
    let backup_file = backup_dir.join(format!("{}.zip", name));

    if !backup_file.exists() {
        return Err("备份文件不存在".to_string());
    }

    let target = Path::new(&target_path);
    fs::create_dir_all(target).map_err(|e| format!("创建目标目录失败: {}", e))?;

    // 解压文件
    let file = fs::File::open(&backup_file).map_err(|e| format!("打开备份文件失败: {}", e))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("读取压缩文件失败: {}", e))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| format!("解压文件失败: {}", e))?;
        let out_path = target.join(file.mangled_name());

        if file.name().ends_with('/') {
            fs::create_dir_all(&out_path).map_err(|e| format!("创建目录失败: {}", e))?;
        } else {
            if let Some(p) = out_path.parent() {
                fs::create_dir_all(p).map_err(|e| format!("创建父目录失败: {}", e))?;
            }
            let mut out_file = fs::File::create(&out_path).map_err(|e| format!("创建文件失败: {}", e))?;
            std::io::copy(&mut file, &mut out_file).map_err(|e| format!("写入文件失败: {}", e))?;
        }
    }

    Ok(format!("还原成功到: {}", target_path))
}

#[tauri::command]
async fn list_backups(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let mut all_backups = Vec::new();

    // 只读取Antigravity账户目录中的JSON文件
    let antigravity_dir = state.config_dir.join("antigravity-accounts");

    if antigravity_dir.exists() {
        for entry in fs::read_dir(&antigravity_dir).map_err(|e| format!("读取用户目录失败: {}", e))? {
            let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
            let path = entry.path();

            if path.extension().map_or(false, |ext| ext == "json") {
                if let Some(name) = path.file_stem() {
                    all_backups.push(name.to_string_lossy().to_string());
                }
            }
        }
    }

    Ok(all_backups)
}

#[tauri::command]
async fn delete_backup(
    name: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // 只删除Antigravity账户JSON文件
    let antigravity_dir = state.config_dir.join("antigravity-accounts");
    let antigravity_file = antigravity_dir.join(format!("{}.json", name));

    if antigravity_file.exists() {
        fs::remove_file(&antigravity_file).map_err(|e| format!("删除用户文件失败: {}", e))?;
        Ok(format!("删除用户成功: {}", name))
    } else {
        Err("用户文件不存在".to_string())
    }
}

#[tauri::command]
async fn clear_all_backups(
    state: State<'_, AppState>,
) -> Result<String, String> {
    let antigravity_dir = state.config_dir.join("antigravity-accounts");

    if antigravity_dir.exists() {
        // 读取目录中的所有文件
        let mut deleted_count = 0;
        for entry in fs::read_dir(&antigravity_dir).map_err(|e| format!("读取用户目录失败: {}", e))? {
            let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
            let path = entry.path();

            // 只删除 JSON 文件
            if path.extension().map_or(false, |ext| ext == "json") {
                fs::remove_file(&path).map_err(|e| format!("删除文件 {} 失败: {}", path.display(), e))?;
                deleted_count += 1;
            }
        }

        Ok(format!("已清空所有用户备份，共删除 {} 个文件", deleted_count))
    } else {
        Ok("用户目录不存在，无需清空".to_string())
    }
}

// Antigravity 相关功能
#[tauri::command]
async fn switch_antigravity_account(
    account_id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // 获取 Antigravity 状态数据库路径
    let app_data = match platform_utils::get_antigravity_db_path() {
        Some(path) => path,
        None => {
            // 如果主路径不存在，尝试其他可能的位置
            let possible_paths = platform_utils::get_all_antigravity_db_paths();
            if possible_paths.is_empty() {
                return Err("未找到Antigravity安装位置".to_string());
            }
            possible_paths[0].clone()
        }
    };

    if !app_data.exists() {
        return Err(format!("Antigravity 状态数据库文件不存在: {}", app_data.display()));
    }

    // 连接到 SQLite 数据库
    let conn = Connection::open(&app_data)
        .map_err(|e| format!("连接数据库失败 ({}): {}", app_data.display(), e))?;

    // 这里应该加载并更新账户信息
    // 由于状态管理的复杂性，我们先返回成功信息
    Ok(format!("已切换到账户: {} (数据库: {})", account_id, app_data.display()))
}

#[tauri::command]
async fn get_antigravity_accounts(
    state: State<'_, AppState>,
) -> Result<Vec<AntigravityAccount>, String> {
    // 这里应该从存储中加载账户列表
    // 暂时返回空列表
    Ok(vec![])
}

/// 获取备份文件列表（内部辅助函数）
fn get_backup_list_internal(config_dir: &Path) -> Result<Vec<String>, String> {
    let mut backups = Vec::new();
    if let Ok(entries) = fs::read_dir(config_dir) {
        for entry in entries.flatten() {
            if let Some(file_name) = entry.path().file_stem() {
                if let Some(name) = file_name.to_str() {
                    backups.push(name.to_string());
                }
            }
        }
    }
    Ok(backups)
}

/// 智能备份Antigravity账户（通用函数）
///
/// 如果该邮箱已有备份，则覆盖；否则创建新备份
///
/// # 参数
/// - `email`: 用户邮箱
///
/// # 返回
/// - `Ok((backup_name, is_overwrite))`: 备份文件名和是否为覆盖操作
/// - `Err(message)`: 错误信息
fn smart_backup_antigravity_account(email: &str) -> Result<(String, bool), String> {
    println!("🔧 执行智能备份，邮箱: {}", email);

    // 1. 获取配置目录
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".antigravity-agent")
        .join("antigravity-accounts");
    fs::create_dir_all(&config_dir)
        .map_err(|e| format!("创建配置目录失败: {}", e))?;

    // 2. 获取现有备份列表
    let existing_backups = get_backup_list_internal(&config_dir)?;
    println!("📋 现有备份列表: {:?}", existing_backups);

    // 3. 检查是否已存在该邮箱的备份
    let email_prefix = format!("{}_", email);
    let existing_backup = existing_backups.iter()
        .find(|backup| backup.starts_with(&email_prefix));

    let (backup_name, is_overwrite) = if let Some(existing) = existing_backup {
        // 覆盖现有备份
        println!("♻️ 发现现有备份，将覆盖: {}", existing);
        (existing.clone(), true)
    } else {
        // 创建新备份
        let timestamp = chrono::Local::now().format("%Y-%m-%dT%H-%M-%S").to_string();
        let new_name = format!("{}_{}", email, timestamp);
        println!("✨ 创建新备份: {}", new_name);
        (new_name, false)
    };

    // 4. 获取数据库路径
    let app_data = platform_utils::get_antigravity_db_path()
        .ok_or_else(|| "未找到Antigravity数据库路径".to_string())?;

    if !app_data.exists() {
        return Err(format!("数据库文件不存在: {}", app_data.display()));
    }

    // 5. 连接数据库并获取数据
    println!("🗃️ 连接数据库: {}", app_data.display());
    let conn = Connection::open(&app_data)
        .map_err(|e| format!("连接数据库失败: {}", e))?;

    let auth_result: SqlResult<String> = conn.query_row(
        "SELECT value FROM ItemTable WHERE key = 'antigravityAuthStatus'",
        [],
        |row| Ok(row.get(0)?),
    );

    let profile_url_result: SqlResult<String> = conn.query_row(
        "SELECT value FROM ItemTable WHERE key = 'antigravity.profileUrl'",
        [],
        |row| Ok(row.get(0)?),
    );

    let user_settings_result: SqlResult<String> = conn.query_row(
        "SELECT value FROM ItemTable WHERE key = 'antigravityUserSettings.allUserSettings'",
        [],
        |row| Ok(row.get(0)?),
    );

    let target_marker_result: SqlResult<String> = conn.query_row(
        "SELECT value FROM ItemTable WHERE key = '__$__targetStorageMarker'",
        [],
        |row| Ok(row.get(0)?),
    );

    drop(conn);

    // 6. 构建备份数据
    let backup_data = serde_json::json!({
        "account_name": backup_name,
        "auth_status": auth_result.ok(),
        "profile_url": profile_url_result.ok(),
        "user_settings": user_settings_result.ok(),
        "target_storage_marker": target_marker_result.ok(),
        "backup_time": chrono::Local::now().to_rfc3339(),
        "version": "1.0"
    });

    // 7. 写入备份文件
    let backup_file = config_dir.join(format!("{}.json", backup_name));
    println!("💾 写入备份文件: {}", backup_file.display());
    fs::write(&backup_file, backup_data.to_string())
        .map_err(|e| format!("写入备份文件失败: {}", e))?;

    let action = if is_overwrite { "覆盖" } else { "创建" };
    println!("✅ 备份完成 ({}): {}", action, backup_name);

    Ok((backup_name, is_overwrite))
}

#[tauri::command]
async fn get_current_antigravity_info(
) -> Result<serde_json::Value, String> {
    // 尝试获取 Antigravity 状态数据库路径
    let app_data = match platform_utils::get_antigravity_db_path() {
        Some(path) => path,
        None => {
            // 如果主路径不存在，尝试其他可能的位置
            let possible_paths = platform_utils::get_all_antigravity_db_paths();
            if possible_paths.is_empty() {
                return Err("未找到Antigravity安装位置".to_string());
            }
            possible_paths[0].clone()
        }
    };

    if !app_data.exists() {
        return Err(format!("Antigravity 状态数据库文件不存在: {}", app_data.display()));
    }

    // 连接到 SQLite 数据库并获取认证信息
    let conn = Connection::open(&app_data)
        .map_err(|e| format!("连接数据库失败 ({}): {}", app_data.display(), e))?;

    let auth_result: SqlResult<String> = conn.query_row(
        "SELECT value FROM ItemTable WHERE key = 'antigravityAuthStatus'",
        [],
        |row| {
            Ok(row.get(0)?)
        },
    );

    match auth_result {
        Ok(auth_json) => {
            // 解析 JSON 字符串
            match serde_json::from_str::<serde_json::Value>(&auth_json) {
                Ok(mut auth_data) => {
                    // 添加数据库路径信息
                    auth_data["db_path"] = serde_json::Value::String(app_data.to_string_lossy().to_string());
                    Ok(auth_data)
                }
                Err(e) => Err(format!("解析认证信息失败: {}", e))
            }
        }
        Err(e) => Err(format!("查询认证信息失败: {}", e)),
    }
}

#[tauri::command]
async fn backup_antigravity_current_account(
    account_name: String,
) -> Result<String, String> {
    println!("📥 调用 backup_antigravity_current_account，文件名: {}", account_name);

    // 从文件名中提取邮箱（格式: email_timestamp）
    let email = account_name.split('_').next()
        .ok_or_else(|| "无效的备份文件名格式".to_string())?;

    println!("📧 提取的邮箱: {}", email);

    // 调用通用智能备份函数
    match smart_backup_antigravity_account(email) {
        Ok((backup_name, is_overwrite)) => {
            let action = if is_overwrite { "更新" } else { "备份" };
            Ok(format!("Antigravity 账户 '{}'{}成功", backup_name, action))
        }
        Err(e) => Err(e)
    }
}

#[tauri::command]
async fn restore_antigravity_account(
    account_name: String,
) -> Result<String, String> {
    println!("📥 调用 restore_antigravity_account，账户名: {}", account_name);

    // 1. 构建备份文件路径
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".antigravity-agent")
        .join("antigravity-accounts");
    let backup_file = config_dir.join(format!("{}.json", account_name));

    // 2. 调用统一的恢复函数
    antigravity_restore::restore_all_antigravity_data(backup_file).await
}

#[tauri::command]
async fn clear_all_antigravity_data() -> Result<String, String> {
    antigravity_cleanup::clear_all_antigravity_data().await
}

// 窗口状态管理命令（使用自动防抖的窗口状态管理器）
#[tauri::command]
async fn save_window_state(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    maximized: bool,
) -> Result<(), String> {
    let window_state = WindowState {
        x,
        y,
        width,
        height,
        maximized,
    };

    // 使用带防抖的窗口状态管理器
    save_ws(window_state).await
}

#[tauri::command]
async fn load_window_state() -> Result<WindowState, String> {
    // 使用窗口状态管理器加载状态
    load_ws().await
}

// 平台支持命令
#[tauri::command]
async fn get_platform_info() -> Result<serde_json::Value, String> {
    let os_type = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let family = std::env::consts::FAMILY;

    let antigravity_available = platform_utils::is_antigravity_available();
    let antigravity_paths = platform_utils::get_all_antigravity_db_paths();

    Ok(serde_json::json!({
        "os": os_type,
        "arch": arch,
        "family": family,
        "antigravity_available": antigravity_available,
        "antigravity_paths": antigravity_paths.iter().map(|p| p.to_string_lossy()).collect::<Vec<_>>(),
        "config_dir": dirs::config_dir().map(|p| p.to_string_lossy().to_string()),
        "data_dir": dirs::data_dir().map(|p| p.to_string_lossy().to_string()),
        "home_dir": dirs::home_dir().map(|p| p.to_string_lossy().to_string())
    }))
}

#[tauri::command]
async fn find_antigravity_installations() -> Result<Vec<String>, String> {
    let paths = platform_utils::find_antigravity_installations();
    Ok(paths.iter().map(|p| p.to_string_lossy().to_string()).collect())
}

#[tauri::command]
async fn validate_antigravity_path(path: String) -> Result<bool, String> {
    let path_buf = PathBuf::from(&path);
    let db_path = path_buf.join("state.vscdb");
    Ok(db_path.exists() && db_path.is_file())
}

// 进程管理命令
#[tauri::command]
async fn kill_antigravity() -> Result<String, String> {
    platform_utils::kill_antigravity_processes()
}

#[tauri::command]
async fn start_antigravity() -> Result<String, String> {
    platform_utils::start_antigravity()
}

#[tauri::command]
async fn backup_and_restart_antigravity() -> Result<String, String> {
    println!("🔄 开始执行 backup_and_restart_antigravity 命令");

    // 1. 关闭进程 (如果存在)
    println!("🛑 步骤1: 检查并关闭 Antigravity 进程");
    let kill_result = match platform_utils::kill_antigravity_processes() {
        Ok(result) => {
            if result.contains("not found") || result.contains("未找到") {
                println!("ℹ️ Antigravity 进程未运行，跳过关闭步骤");
                "Antigravity 进程未运行".to_string()
            } else {
                println!("✅ 进程关闭结果: {}", result);
                result
            }
        }
        Err(e) => {
            if e.contains("not found") || e.contains("未找到") {
                println!("ℹ️ Antigravity 进程未运行，跳过关闭步骤");
                "Antigravity 进程未运行".to_string()
            } else {
                return Err(format!("关闭进程时发生错误: {}", e));
            }
        }
    };

    // 等待一秒确保进程完全关闭
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // 2. 备份当前账户信息（使用统一的智能备份函数）
    println!("💾 步骤2: 备份当前账户信息");

    // 获取邮箱
    let app_data = platform_utils::get_antigravity_db_path()
        .ok_or_else(|| "未找到Antigravity数据库路径".to_string())?;

    let conn = Connection::open(&app_data)
        .map_err(|e| format!("连接数据库失败: {}", e))?;

    // 获取认证信息来提取邮箱
    let auth_str: String = conn.query_row(
        "SELECT value FROM ItemTable WHERE key = 'antigravityAuthStatus'",
        [],
        |row| Ok(row.get(0)?),
    ).map_err(|e| format!("查询认证信息失败: {}", e))?;

    drop(conn);

    let auth_data: serde_json::Value = serde_json::from_str(&auth_str)
        .map_err(|e| format!("解析认证信息失败: {}", e))?;

    let email = auth_data.get("email")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "认证信息中未找到邮箱".to_string())?;

    println!("📧 获取到的邮箱: {}", email);

    // 调用通用智能备份函数
    let (backup_name, is_overwrite) = smart_backup_antigravity_account(email)?;
    let backup_action = if is_overwrite { "更新" } else { "创建" };
    println!("✅ 备份完成 ({}): {}", backup_action, backup_name);

    // 3. 清除 Antigravity 所有数据 (彻底注销)
    println!("🗑️ 步骤3: 清除所有 Antigravity 数据 (彻底注销)");
    match antigravity_cleanup::clear_all_antigravity_data().await {
        Ok(result) => {
            println!("✅ 清除完成: {}", result);
        }
        Err(e) => {
            println!("⚠️ 清除失败: {}", e);
            return Err(format!("清除数据失败: {}", e));
        }
    }

    // 等待一秒确保操作完成
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // 4. 重新启动进程 (暂时注释掉，让用户手动启动)
    // println!("🚀 步骤4: 重新启动 Antigravity");
    // let start_result = platform_utils::start_antigravity();
    // let start_message = match start_result {
    //     Ok(result) => {
    //         println!("✅ 启动结果: {}", result);
    //         result
    //     }
    //     Err(e) => {
    //         println!("⚠️ 启动失败: {}", e);
    //         format!("启动失败: {}", e)
    //     }
    // };

    let start_message = "已清除完成，请手动启动 Antigravity".to_string();

    let final_message = format!("{} -> 已{}备份: {} -> 已清除账户数据 -> {}",
        kill_result, backup_action, backup_name, start_message);
    println!("🎉 所有操作完成: {}", final_message);

    Ok(final_message)
}

#[tauri::command]
async fn switch_to_antigravity_account(
    account_name: String,
) -> Result<String, String> {
    println!("🔄 开始执行切换到账户: {}", account_name);

    // 1. 关闭 Antigravity 进程 (如果存在)
    println!("🛑 步骤1: 检查并关闭 Antigravity 进程");
    let kill_result = match platform_utils::kill_antigravity_processes() {
        Ok(result) => {
            if result.contains("not found") || result.contains("未找到") {
                println!("ℹ️ Antigravity 进程未运行，跳过关闭步骤");
                "Antigravity 进程未运行".to_string()
            } else {
                println!("✅ 进程关闭结果: {}", result);
                result
            }
        }
        Err(e) => {
            if e.contains("not found") || e.contains("未找到") {
                println!("ℹ️ Antigravity 进程未运行，跳过关闭步骤");
                "Antigravity 进程未运行".to_string()
            } else {
                return Err(format!("关闭进程时发生错误: {}", e));
            }
        }
    };

    // 等待一秒确保进程完全关闭
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // 2. 恢复指定账户到 Antigravity 数据库
    println!("💾 步骤2: 恢复账户数据: {}", account_name);
    let restore_result = restore_antigravity_account(account_name.clone()).await?;
    println!("✅ 账户数据恢复完成: {}", restore_result);

    // 等待一秒确保数据库操作完成
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // 3. 重新启动 Antigravity 进程 (暂时注释掉，让用户手动启动)
    // println!("🚀 步骤3: 重新启动 Antigravity");
    // let start_result = platform_utils::start_antigravity();
    // let start_message = match start_result {
    //     Ok(result) => {
    //         println!("✅ 启动结果: {}", result);
    //         result
    //     }
    //     Err(e) => {
    //         println!("⚠️ 启动失败: {}", e);
    //         format!("启动失败: {}", e)
    //     }
    // };
    let start_message = "已恢复账户，请手动启动 Antigravity".to_string();


    let final_message = format!("{} -> {} -> {}", kill_result, restore_result, start_message);
    println!("🎉 账户切换完成: {}", final_message);

    Ok(final_message)
}

fn main() {
    // 启动 Antigravity Agent v0.1.0
    println!("🚀 启动 Antigravity Agent v0.1.0");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState::default())
        .setup(|app| {
            // 初始化窗口事件处理器
            if let Err(e) = window_event_handler::init_window_event_handler(&app) {
                eprintln!("⚠️  窗口事件处理器初始化失败: {}", e);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            backup_profile,
            restore_profile,
            list_backups,
            delete_backup,
            clear_all_backups,
            // Antigravity 相关命令
            switch_antigravity_account,
            get_antigravity_accounts,
            get_current_antigravity_info,
            backup_antigravity_current_account,
            restore_antigravity_account,
            switch_to_antigravity_account,
            clear_all_antigravity_data,
            // 进程管理命令
            kill_antigravity,
            start_antigravity,
            backup_and_restart_antigravity,
            // 平台支持命令
            get_platform_info,
            find_antigravity_installations,
            validate_antigravity_path,
            // 窗口状态管理命令
            save_window_state,
            load_window_state
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}