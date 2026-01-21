//! OAuth 凭据安全管理
//!
//! 凭据获取优先级：
//! 1. 环境变量 ANTIGRAVITY_OAUTH_CLIENT_ID / ANTIGRAVITY_OAUTH_CLIENT_SECRET
//! 2. 系统凭据存储 (Windows Credential Manager / macOS Keychain / Linux Secret Service)
//! 3. 旧格式 JSON 文件迁移（迁移后自动删除）

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const KEYRING_SERVICE: &str = "antigravity-agent";
const KEYRING_USERNAME: &str = "oauth_credentials";

#[derive(Serialize, Deserialize)]
struct StoredCredentials {
    client_id: String,
    client_secret: String,
}

#[derive(Deserialize)]
struct CredentialsFile {
    client_id: String,
    client_secret: String,
}

/// 检查系统凭据存储中是否有 OAuth 凭据
pub fn has_oauth_credentials_in_keyring() -> Result<bool, String> {
    let ent = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USERNAME)
        .map_err(|e| format!("初始化系统凭据存储失败: {}", e))?;
    match ent.get_password() {
        Ok(_) => Ok(true),
        Err(keyring::Error::NoEntry) => Ok(false),
        Err(e) => Err(format!("读取系统凭据存储失败: {}", e)),
    }
}

/// 保存 OAuth 凭据到系统凭据存储
pub fn save_oauth_credentials_to_keyring(
    client_id: &str,
    client_secret: &str,
) -> Result<(), String> {
    let payload = StoredCredentials {
        client_id: client_id.to_string(),
        client_secret: client_secret.to_string(),
    };
    let serialized =
        serde_json::to_string(&payload).map_err(|e| format!("序列化凭据失败: {}", e))?;
    let ent = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USERNAME)
        .map_err(|e| format!("初始化系统凭据存储失败: {}", e))?;
    ent.set_password(&serialized)
        .map_err(|e| format!("写入系统凭据存储失败: {}", e))
}

/// 清除系统凭据存储中的 OAuth 凭据
pub fn clear_oauth_credentials_from_keyring() -> Result<(), String> {
    let ent = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USERNAME)
        .map_err(|e| format!("初始化系统凭据存储失败: {}", e))?;
    match ent.delete_password() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(format!("清除系统凭据存储失败: {}", e)),
    }
}

fn load_oauth_credentials_from_keyring() -> Result<(String, String), String> {
    let ent = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USERNAME)
        .map_err(|e| format!("初始化系统凭据存储失败: {}", e))?;
    let raw = ent
        .get_password()
        .map_err(|e| format!("读取系统凭据存储失败: {}", e))?;
    let parsed: StoredCredentials =
        serde_json::from_str(&raw).map_err(|_| "系统凭据存储内容已损坏".to_string())?;
    if parsed.client_id.is_empty() || parsed.client_secret.is_empty() {
        return Err("系统凭据存储内容不完整".to_string());
    }
    Ok((parsed.client_id, parsed.client_secret))
}

fn try_migrate_from_plain_file(config_dir: &Path) -> Option<(String, String)> {
    let path = config_dir.join("oauth_credentials.json");
    let content = fs::read_to_string(&path).ok()?;
    let parsed: CredentialsFile = serde_json::from_str(&content).ok()?;
    if parsed.client_id.is_empty() || parsed.client_secret.is_empty() {
        return None;
    }
    if save_oauth_credentials_to_keyring(&parsed.client_id, &parsed.client_secret).is_ok() {
        let _ = fs::remove_file(&path);
        return Some((parsed.client_id, parsed.client_secret));
    }
    None
}

/// 解析 OAuth 凭据
///
/// 优先级：环境变量 > 系统凭据存储 > 旧文件迁移
pub fn resolve_oauth_credentials(config_dir: &Path) -> Result<(String, String), String> {
    // 1. 环境变量优先
    let env_client_id = std::env::var("ANTIGRAVITY_OAUTH_CLIENT_ID").ok();
    let env_client_secret = std::env::var("ANTIGRAVITY_OAUTH_CLIENT_SECRET").ok();

    if let (Some(id), Some(secret)) = (env_client_id, env_client_secret) {
        if id.is_empty() || secret.is_empty() {
            return Err(
                "环境变量 ANTIGRAVITY_OAUTH_CLIENT_ID / ANTIGRAVITY_OAUTH_CLIENT_SECRET 为空"
                    .to_string(),
            );
        }
        return Ok((id, secret));
    }

    // 2. 系统凭据存储
    if let Ok(pair) = load_oauth_credentials_from_keyring() {
        return Ok(pair);
    }

    // 3. 旧文件迁移
    if let Some(pair) = try_migrate_from_plain_file(config_dir) {
        return Ok(pair);
    }

    Err(format!(
        "缺少 OAuth 凭据：请设置环境变量 ANTIGRAVITY_OAUTH_CLIENT_ID / ANTIGRAVITY_OAUTH_CLIENT_SECRET，或在应用内保存到系统凭据存储（也可提供旧文件用于迁移：{}）",
        config_dir.join("oauth_credentials.json").display()
    ))
}
