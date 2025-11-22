use crate::AppState;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;
/// 备份相关命令
/// 负责配置文件和账户的备份、恢复、删除等操作
use tauri::State;

/// 备份数据收集结构
#[derive(Serialize, Deserialize, Debug)]
pub struct BackupData {
    filename: String,
    #[serde(rename = "content")]
    content: Value,
    #[serde(rename = "timestamp")]
    timestamp: u64,
}

/// 恢复结果
#[derive(Serialize, Deserialize, Debug)]
pub struct RestoreResult {
    #[serde(rename = "restoredCount")]
    restored_count: u32,
    failed: Vec<FailedBackup>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FailedBackup {
    filename: String,
    error: String,
}

use std::fs;
use std::io::Write;
use walkdir::WalkDir;
use zip::{write::FileOptions, ZipWriter};

/// 创建配置文件备份
#[tauri::command]
pub async fn backup_profile(
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
        let name = path
            .strip_prefix(source)
            .map_err(|e| format!("处理路径失败: {}", e))?;

        if path.is_file() {
            let mut file = fs::File::open(path).map_err(|e| format!("打开文件失败: {}", e))?;
            zip.start_file(name.to_string_lossy(), options)
                .map_err(|e| format!("添加文件到压缩包失败: {}", e))?;
            let mut buffer = Vec::new();
            use std::io::Read;
            file.read_to_end(&mut buffer)
                .map_err(|e| format!("读取文件失败: {}", e))?;
            zip.write_all(&buffer)
                .map_err(|e| format!("写入压缩包失败: {}", e))?;
        }
    }

    zip.finish().map_err(|e| format!("完成压缩失败: {}", e))?;

    // 更新配置信息
    let _profile_info = crate::ProfileInfo {
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

/// 恢复配置文件备份
#[tauri::command]
pub async fn restore_profile(
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
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("解压文件失败: {}", e))?;
        let out_path = target.join(file.mangled_name());

        if file.name().ends_with('/') {
            fs::create_dir_all(&out_path).map_err(|e| format!("创建目录失败: {}", e))?;
        } else {
            if let Some(p) = out_path.parent() {
                fs::create_dir_all(p).map_err(|e| format!("创建父目录失败: {}", e))?;
            }
            let mut out_file =
                fs::File::create(&out_path).map_err(|e| format!("创建文件失败: {}", e))?;
            std::io::copy(&mut file, &mut out_file).map_err(|e| format!("写入文件失败: {}", e))?;
        }
    }

    Ok(format!("还原成功到: {}", target_path))
}

/// 列出所有可用备份
#[tauri::command]
pub async fn list_backups(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let mut all_backups = Vec::new();

    // 只读取Antigravity账户目录中的JSON文件
    let antigravity_dir = state.config_dir.join("antigravity-accounts");

    if antigravity_dir.exists() {
        for entry in
            fs::read_dir(&antigravity_dir).map_err(|e| format!("读取用户目录失败: {}", e))?
        {
            let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "json") {
                if let Some(name) = path.file_stem() {
                    all_backups.push(name.to_string_lossy().to_string());
                }
            }
        }
    }

    Ok(all_backups)
}

/// 获取最近使用的账户列表（基于文件修改时间排序）
#[tauri::command]
pub async fn get_recent_accounts(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<String>, String> {
    let antigravity_dir = state.config_dir.join("antigravity-accounts");

    if !antigravity_dir.exists() {
        return Ok(Vec::new());
    }

    let mut accounts_with_time: Vec<(String, std::time::SystemTime)> = Vec::new();

    // 读取所有账户文件并获取修改时间
    for entry in fs::read_dir(&antigravity_dir).map_err(|e| format!("读取用户目录失败: {}", e))? {
        let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
        let path = entry.path();

        if path.extension().is_some_and(|ext| ext == "json") {
            if let Some(name) = path.file_stem() {
                let account_name = name.to_string_lossy().to_string();
                
                // 获取文件修改时间
                match fs::metadata(&path) {
                    Ok(metadata) => {
                        if let Ok(modified) = metadata.modified() {
                            accounts_with_time.push((account_name, modified));
                        }
                    }
                    Err(_) => continue,
                }
            }
        }
    }

    // 按修改时间降序排序（最近修改的在前）
    accounts_with_time.sort_by(|a, b| b.1.cmp(&a.1));

    // 提取账户名并应用限制
    let mut result: Vec<String> = accounts_with_time
        .into_iter()
        .map(|(name, _)| name)
        .collect();

    if let Some(limit) = limit {
        result.truncate(limit);
    }

    Ok(result)
}


/// 收集所有备份文件的完整内容
#[tauri::command]
pub async fn collect_backup_contents(
    state: State<'_, AppState>,
) -> Result<Vec<BackupData>, String> {
    let mut backups_with_content = Vec::new();

    // 读取Antigravity账户目录中的JSON文件
    let antigravity_dir = state.config_dir.join("antigravity-accounts");

    if !antigravity_dir.exists() {
        return Ok(backups_with_content);
    }

    for entry in fs::read_dir(&antigravity_dir).map_err(|e| format!("读取用户目录失败: {}", e))?
    {
        let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
        let path = entry.path();

        if path.extension().is_some_and(|ext| ext == "json") {
            let filename = path
                .file_name()
                .and_then(|name| name.to_str())
                .map(|s| s.to_string())
                .unwrap_or_default();

            if filename.is_empty() {
                continue;
            }

            match fs::read_to_string(&path).map_err(|e| format!("读取文件失败 {}: {}", filename, e))
            {
                Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(json_value) => {
                        backups_with_content.push(BackupData {
                            filename,
                            content: json_value,
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs(),
                        });
                    }
                    Err(e) => {
                        println!("⚠️ 跳过损坏的备份文件 {}: {}", filename, e);
                    }
                },
                Err(_) => {
                    println!("⚠️ 跳过无法读取的文件: {}", filename);
                }
            }
        }
    }

    Ok(backups_with_content)
}

/// 恢复备份文件到本地
#[tauri::command]
pub async fn restore_backup_files(
    backups: Vec<BackupData>,
    state: State<'_, AppState>,
) -> Result<RestoreResult, String> {
    let mut results = RestoreResult {
        restored_count: 0,
        failed: Vec::new(),
    };

    // 获取目标目录
    let antigravity_dir = state.config_dir.join("antigravity-accounts");

    // 确保目录存在
    if let Err(e) = fs::create_dir_all(&antigravity_dir) {
        return Err(format!("创建目录失败: {}", e));
    }

    // 遍历每个备份
    for backup in backups {
        let file_path = antigravity_dir.join(&backup.filename);

        match fs::write(
            &file_path,
            serde_json::to_string_pretty(&backup.content).unwrap_or_default(),
        )
        .map_err(|e| format!("写入文件失败: {}", e))
        {
            Ok(_) => {
                results.restored_count += 1;
            }
            Err(e) => {
                results.failed.push(FailedBackup {
                    filename: backup.filename,
                    error: e,
                });
            }
        }
    }

    Ok(results)
}

/// 删除指定备份
#[tauri::command]
pub async fn delete_backup(name: String, state: State<'_, AppState>) -> Result<String, String> {
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

/// 清空所有备份
#[tauri::command]
pub async fn clear_all_backups(state: State<'_, AppState>) -> Result<String, String> {
    let antigravity_dir = state.config_dir.join("antigravity-accounts");

    if antigravity_dir.exists() {
        // 读取目录中的所有文件
        let mut deleted_count = 0;
        for entry in
            fs::read_dir(&antigravity_dir).map_err(|e| format!("读取用户目录失败: {}", e))?
        {
            let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
            let path = entry.path();

            // 只删除 JSON 文件
            if path.extension().is_some_and(|ext| ext == "json") {
                fs::remove_file(&path)
                    .map_err(|e| format!("删除文件 {} 失败: {}", path.display(), e))?;
                deleted_count += 1;
            }
        }

        Ok(format!(
            "已清空所有用户备份，共删除 {} 个文件",
            deleted_count
        ))
    } else {
        Ok("用户目录不存在，无需清空".to_string())
    }
}

// 备份相关函数将在后续步骤中移动到这里
