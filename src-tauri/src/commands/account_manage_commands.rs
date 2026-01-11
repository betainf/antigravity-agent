//! 账户备份/导入导出与加解密命令

use crate::log_async_command;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::time::SystemTime;
use tauri::State;

fn is_safe_backup_name(s: &str) -> bool {
    if s.is_empty() || s.len() > 255 {
        return false;
    }
    if s.contains('/') || s.contains('\\') || s.contains(':') {
        return false;
    }
    s.chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '@' | '.' | '_' | '-' | '+'))
}

fn is_safe_backup_filename(filename: &str) -> bool {
    if !filename.ends_with(".json") {
        return false;
    }
    let name = filename.trim_end_matches(".json");
    is_safe_backup_name(name)
}

/// 备份数据收集结构
#[derive(Serialize, Deserialize, Debug)]
pub struct AccountExportedData {
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
    failed: Vec<FailedAccountExportedData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FailedAccountExportedData {
    filename: String,
    error: String,
}

/// 收集所有账户文件的完整内容, 用于导出
#[tauri::command]
pub async fn collect_account_contents(
    state: State<'_, crate::AppState>,
) -> Result<Vec<AccountExportedData>, String> {
    let mut backups_with_content = Vec::new();

    const MAX_ACCOUNT_JSON_BYTES: u64 = 5 * 1024 * 1024;

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

            if !is_safe_backup_filename(&filename) {
                continue;
            }

            if let Ok(meta) = fs::metadata(&path) {
                if meta.len() > MAX_ACCOUNT_JSON_BYTES {
                    tracing::warn!(target: "backup::scan", filename = %filename, "跳过过大的账户文件");
                    continue;
                }
            }

            match fs::read_to_string(&path).map_err(|e| format!("读取文件失败 {}: {}", filename, e))
            {
                Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(json_value) => {
                        backups_with_content.push(AccountExportedData {
                            filename,
                            content: json_value,
                            timestamp: SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs(),
                        });
                    }
                    Err(e) => {
                        tracing::warn!(target: "backup::scan", filename = %filename, error = %e, "跳过损坏的备份文件");
                    }
                },
                Err(_) => {
                    tracing::warn!(target: "backup::scan", filename = %filename, "跳过无法读取的文件");
                }
            }
        }
    }

    Ok(backups_with_content)
}

/// 恢复备份文件到本地
#[tauri::command]
pub async fn restore_backup_files(
    account_file_data: Vec<AccountExportedData>,
    state: State<'_, crate::AppState>,
) -> Result<RestoreResult, String> {
    let mut results = RestoreResult {
        restored_count: 0,
        failed: Vec::new(),
    };

    const MAX_RESTORE_FILES: usize = 200;
    const MAX_ACCOUNT_JSON_BYTES: usize = 5 * 1024 * 1024;

    if account_file_data.len() > MAX_RESTORE_FILES {
        return Err("导入文件过多".to_string());
    }

    // 获取目标目录
    let antigravity_dir = state.config_dir.join("antigravity-accounts");

    // 确保目录存在
    if let Err(e) = fs::create_dir_all(&antigravity_dir) {
        return Err(format!("创建目录失败: {}", e));
    }

    // 遍历每个备份
    for account_file in account_file_data {
        if !is_safe_backup_filename(&account_file.filename) {
            results.failed.push(FailedAccountExportedData {
                filename: account_file.filename,
                error: "非法文件名".to_string(),
            });
            continue;
        }
        let file_path = antigravity_dir.join(&account_file.filename);

        let serialized = match serde_json::to_string_pretty(&account_file.content)
            .map_err(|e| format!("序列化失败: {}", e))
        {
            Ok(s) => s,
            Err(e) => {
                results.failed.push(FailedAccountExportedData {
                    filename: account_file.filename,
                    error: e,
                });
                continue;
            }
        };

        if serialized.len() > MAX_ACCOUNT_JSON_BYTES {
            results.failed.push(FailedAccountExportedData {
                filename: account_file.filename,
                error: "账户文件过大".to_string(),
            });
            continue;
        }

        let write_result = (|| -> Result<(), String> {
            let mut tmp = tempfile::Builder::new()
                .prefix(".restore_")
                .suffix(".tmp")
                .tempfile_in(&antigravity_dir)
                .map_err(|e| format!("创建临时文件失败: {}", e))?;
            use std::io::Write;
            tmp.write_all(serialized.as_bytes())
                .map_err(|e| format!("写入临时文件失败: {}", e))?;
            if file_path.exists() {
                fs::remove_file(&file_path).map_err(|e| format!("覆盖旧文件失败: {}", e))?;
            }
            tmp.persist(&file_path)
                .map_err(|e| format!("落盘失败: {}", e.error))?;
            Ok(())
        })();

        match write_result {
            Ok(()) => results.restored_count += 1,
            Err(e) => results.failed.push(FailedAccountExportedData {
                filename: account_file.filename,
                error: e,
            }),
        }
    }

    Ok(results)
}

/// 删除指定备份
#[tauri::command]
pub async fn delete_backup(
    name: String,
    state: State<'_, crate::AppState>,
) -> Result<String, String> {
    if !is_safe_backup_name(&name) {
        return Err("非法账户名".to_string());
    }
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
pub async fn clear_all_backups(state: State<'_, crate::AppState>) -> Result<String, String> {
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

/// 加密配置数据（用于账户导出）
#[tauri::command]
pub async fn encrypt_config_data(json_data: String, password: String) -> Result<String, String> {
    log_async_command!("encrypt_config_data", async {
        use argon2::Argon2;
        use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
        use chacha20poly1305::aead::{Aead, KeyInit};
        use chacha20poly1305::XChaCha20Poly1305;
        use rand::RngCore;
        use zeroize::Zeroize;

        const ENCRYPTED_PREFIX: &str = "AGENC1:";
        const MAX_PLAINTEXT_BYTES: usize = 5 * 1024 * 1024;

        if json_data.len() > MAX_PLAINTEXT_BYTES {
            return Err("待加密数据过大".to_string());
        }

        let mut password_bytes = password.into_bytes();
        if password_bytes.is_empty() {
            return Err("密码不能为空".to_string());
        }
        if password_bytes.len() < 8 {
            return Err("密码长度至少 8 位".to_string());
        }
        if password_bytes.len() > 1024 {
            return Err("密码长度过长".to_string());
        }

        let mut salt = [0u8; 16];
        let mut nonce = [0u8; 24];
        rand::thread_rng().fill_bytes(&mut salt);
        rand::thread_rng().fill_bytes(&mut nonce);

        let params = argon2::Params::new(32768, 3, 1, Some(32))
            .map_err(|_| "加密参数初始化失败".to_string())?;
        let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

        let mut key = [0u8; 32];
        argon2
            .hash_password_into(&password_bytes, &salt, &mut key)
            .map_err(|_| "派生密钥失败".to_string())?;
        password_bytes.zeroize();

        let cipher = XChaCha20Poly1305::new((&key).into());
        let ciphertext = cipher
            .encrypt((&nonce).into(), json_data.as_bytes())
            .map_err(|_| "加密失败".to_string())?;
        key.zeroize();

        #[derive(Serialize)]
        struct Payload<'a> {
            v: u8,
            kdf: &'a str,
            m_cost_kib: u32,
            t_cost: u32,
            p_cost: u32,
            salt_b64: String,
            nonce_b64: String,
            ct_b64: String,
        }

        let payload = Payload {
            v: 1,
            kdf: "argon2id",
            m_cost_kib: 32768,
            t_cost: 3,
            p_cost: 1,
            salt_b64: BASE64.encode(salt),
            nonce_b64: BASE64.encode(nonce),
            ct_b64: BASE64.encode(ciphertext),
        };

        let json = serde_json::to_string(&payload).map_err(|_| "序列化密文失败".to_string())?;
        Ok(format!(
            "{}{}",
            ENCRYPTED_PREFIX,
            BASE64.encode(json.as_bytes())
        ))
    })
}

/// 解密配置数据（用于账户导入）
#[tauri::command]
pub async fn decrypt_config_data(
    encrypted_data: String,
    password: String,
) -> Result<String, String> {
    log_async_command!("decrypt_config_data", async {
        use argon2::Argon2;
        use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
        use chacha20poly1305::aead::{Aead, KeyInit};
        use chacha20poly1305::XChaCha20Poly1305;
        use zeroize::Zeroize;

        const ENCRYPTED_PREFIX: &str = "AGENC1:";

        let mut password_bytes = password.into_bytes();
        if password_bytes.is_empty() {
            return Err("密码不能为空".to_string());
        }
        if password_bytes.len() > 1024 {
            return Err("密码长度过长".to_string());
        }

        if let Some(rest) = encrypted_data.strip_prefix(ENCRYPTED_PREFIX) {
            #[derive(Deserialize)]
            struct Payload {
                v: u8,
                kdf: String,
                m_cost_kib: u32,
                t_cost: u32,
                p_cost: u32,
                salt_b64: String,
                nonce_b64: String,
                ct_b64: String,
            }

            let json_bytes = BASE64
                .decode(rest)
                .map_err(|_| "密文格式无效".to_string())?;
            let json_str =
                std::str::from_utf8(&json_bytes).map_err(|_| "密文格式无效".to_string())?;
            let payload: Payload =
                serde_json::from_str(json_str).map_err(|_| "密文格式无效".to_string())?;

            if payload.v != 1 || payload.kdf != "argon2id" {
                return Err("不支持的密文版本".to_string());
            }

            let salt = BASE64
                .decode(payload.salt_b64)
                .map_err(|_| "密文格式无效".to_string())?;
            let nonce = BASE64
                .decode(payload.nonce_b64)
                .map_err(|_| "密文格式无效".to_string())?;
            let ciphertext = BASE64
                .decode(payload.ct_b64)
                .map_err(|_| "密文格式无效".to_string())?;

            if salt.len() != 16 || nonce.len() != 24 {
                return Err("密文格式无效".to_string());
            }

            let params =
                argon2::Params::new(payload.m_cost_kib, payload.t_cost, payload.p_cost, Some(32))
                    .map_err(|_| "密文参数无效".to_string())?;
            let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

            let mut key = [0u8; 32];
            argon2
                .hash_password_into(&password_bytes, &salt, &mut key)
                .map_err(|_| "解密失败".to_string())?;
            password_bytes.zeroize();

            let cipher = XChaCha20Poly1305::new((&key).into());
            let plaintext = cipher
                .decrypt((&nonce[..]).into(), ciphertext.as_ref())
                .map_err(|_| "解密失败，密码错误或数据已损坏".to_string())?;
            key.zeroize();

            let decrypted =
                String::from_utf8(plaintext).map_err(|_| "解密失败，数据可能已损坏".to_string())?;
            return Ok(decrypted);
        }

        use base64::engine::general_purpose::STANDARD as LEGACY_BASE64;
        let decoded = LEGACY_BASE64
            .decode(encrypted_data)
            .map_err(|_| "Base64 解码失败".to_string())?;
        let mut result = Vec::with_capacity(decoded.len());
        for (i, byte) in decoded.iter().enumerate() {
            let key_byte = password_bytes[i % password_bytes.len()];
            result.push(byte ^ key_byte);
        }
        password_bytes.zeroize();
        let decrypted =
            String::from_utf8(result).map_err(|_| "解密失败，数据可能已损坏".to_string())?;
        Ok(decrypted)
    })
}

/// 备份并重启 Antigravity（迁移自 process_commands）
#[tauri::command]
pub async fn sign_in_new_antigravity_account() -> Result<String, String> {
    println!("🔄 开始执行 sign_in_new_antigravity_account 命令");

    // 1. 关闭进程 (如果存在)
    println!("🛑 步骤1: 检查并关闭 Antigravity 进程");
    let kill_result = match crate::platform::kill_antigravity_processes() {
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

    // 等待500ms确保进程完全关闭（缩短等待时间避免前端超时）
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // 2. 备份当前账户信息（直接调用 save_antigravity_current_account）
    println!("💾 步骤2: 调用 save_antigravity_current_account 备份当前账户信息");
    let backup_info = match crate::commands::save_antigravity_current_account().await {
        Ok(msg) => {
            println!("✅ 备份完成: {}", msg);
            Some(msg)
        }
        Err(e) => {
            println!("⚠️ 备份失败: {}", e);
            None
        }
    };

    // 3. 清除 Antigravity 所有数据 (彻底注销)
    println!("🗑️ 步骤3: 清除所有 Antigravity 数据 (彻底注销)");
    match crate::antigravity::cleanup::clear_all_antigravity_data().await {
        Ok(result) => {
            tracing::info!("✅ 清除完成: {}", result);
        }
        Err(e) => {
            // 清除失败可能是因为数据库本来就是空的，这是正常情况
            println!("ℹ️ 清除数据时出现: {}（可能数据库本来就是空的）", e);
        }
    }

    // 等待300ms确保操作完成（缩短等待时间避免前端超时）
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

    // 4. 重新启动进程
    println!("🚀 步骤4: 重新启动 Antigravity");
    let start_result = crate::antigravity::starter::start_antigravity();
    let start_message = match start_result {
        Ok(result) => {
            println!("✅ 启动结果: {}", result);
            result
        }
        Err(e) => {
            println!("⚠️ 启动失败: {}", e);
            format!("启动失败: {}", e)
        }
    };

    let final_message = if let Some(backup_message) = backup_info {
        format!(
            "{} -> 已备份: {} -> 已清除账户数据 -> {}",
            kill_result, backup_message, start_message
        )
    } else {
        format!(
            "{} -> 未检测到登录用户（跳过备份） -> 已清除账户数据 -> {}",
            kill_result, start_message
        )
    };
    println!("🎉 所有操作完成: {}", final_message);

    Ok(final_message)
}
#[cfg(test)]
mod tests {
    use super::{decrypt_config_data, encrypt_config_data};
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};

    fn legacy_encrypt_xor_base64(plaintext: &str, password: &str) -> String {
        let password_bytes = password.as_bytes();
        let mut result = Vec::with_capacity(plaintext.len());
        for (i, byte) in plaintext.as_bytes().iter().enumerate() {
            let key_byte = password_bytes[i % password_bytes.len()];
            result.push(byte ^ key_byte);
        }
        BASE64.encode(result)
    }

    #[tokio::test]
    async fn encrypt_decrypt_roundtrip_v1() {
        let json = r#"{"a":1,"b":"x","c":[true,false]}"#.to_string();
        let password = "password123".to_string();
        let encrypted = encrypt_config_data(json.clone(), password.clone())
            .await
            .unwrap();
        assert!(encrypted.starts_with("AGENC1:"));
        let decrypted = decrypt_config_data(encrypted, password).await.unwrap();
        assert_eq!(decrypted, json);
    }

    #[tokio::test]
    async fn decrypt_fails_with_wrong_password_v1() {
        let json = r#"{"k":"v"}"#.to_string();
        let encrypted = encrypt_config_data(json, "password123".to_string())
            .await
            .unwrap();
        let err = decrypt_config_data(encrypted, "password124".to_string()).await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn decrypt_legacy_xor_base64() {
        let json = r#"{"legacy":true,"n":42}"#;
        let password = "password123";
        let encrypted = legacy_encrypt_xor_base64(json, password);
        let decrypted = decrypt_config_data(encrypted, password.to_string())
            .await
            .unwrap();
        assert_eq!(decrypted, json);
    }

    #[tokio::test]
    async fn encrypt_rejects_short_password() {
        let err = encrypt_config_data("{}".to_string(), "short".to_string()).await;
        assert!(err.is_err());
    }
}
