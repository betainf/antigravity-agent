use crate::antigravity::account::decode_jetski_state_proto;
use base64::Engine;
use prost::Message;
use rusqlite::{Connection, OptionalExtension};
use serde_json::{from_str, Value};
use std::fs;

/// 获取所有 Antigravity 账户
pub async fn get_all(config_dir: &std::path::Path) -> Result<Vec<Value>, String> {
    tracing::debug!("📋 开始获取所有 Antigravity 账户 (Service)");
    let start_time = std::time::Instant::now();

    let result = async {
        let mut accounts: Vec<(std::time::SystemTime, Value)> = Vec::new();
        let antigravity_dir = config_dir.join("antigravity-accounts");

        if !antigravity_dir.exists() {
            tracing::info!("📂 备份目录不存在，返回空列表");
            return Ok(Vec::new());
        }

        let entries =
            fs::read_dir(&antigravity_dir).map_err(|e| format!("读取备份目录失败: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "json") {
                let file_name = match path.file_stem() {
                    Some(name) => name.to_string_lossy().to_string(),
                    None => continue,
                };

                let content = fs::read_to_string(&path)
                    .map_err(|e| format!("读取文件失败 {}: {}", file_name, e))?;

                let backup_data: Value = from_str(&content)
                    .map_err(|e| format!("解析 JSON 失败 {}: {}", file_name, e))?;

                let jetski_state = backup_data
                    .get("jetskiStateSync.agentManagerInitState")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        format!(
                            "备份文件 {} 缺少 jetskiStateSync.agentManagerInitState",
                            file_name
                        )
                    })?;

                let decoded = crate::antigravity::account::decode_jetski_state_proto(jetski_state)?;

                let modified_time = fs::metadata(&path)
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH);

                accounts.push((modified_time, decoded));
            }
        }

        accounts.sort_by(|a, b| b.0.cmp(&a.0));
        let decoded_only: Vec<Value> = accounts.into_iter().map(|(_, decoded)| decoded).collect();
        Ok(decoded_only)
    }
    .await;

    let duration = start_time.elapsed();
    match result {
        Ok(accounts) => {
            tracing::debug!(
                duration_ms = duration.as_millis(),
                account_count = accounts.len(),
                "获取账户列表完成"
            );
            Ok(accounts)
        }
        Err(e) => {
            tracing::error!(error = %e, duration_ms = duration.as_millis(), "获取账户列表失败");
            Err(e)
        }
    }
}

/// 获取当前 Antigravity 账户信息
pub async fn get_current() -> Result<Value, String> {
    tracing::info!("开始获取当前 Antigravity 信息");

    let start_time = std::time::Instant::now();

    let result = async {
        // 尝试获取 Antigravity 状态数据库路径
        let app_data = match crate::platform::get_antigravity_db_path() {
            Some(path) => path,
            None => {
                // 如果主路径不存在，尝试其他可能的位置
                let possible_paths = crate::platform::get_all_antigravity_db_paths();
                if possible_paths.is_empty() {
                    return Err("未找到Antigravity安装位置".to_string());
                }
                possible_paths[0].clone()
            }
        };

        if !app_data.exists() {
            return Err(format!(
                "Antigravity 状态数据库文件不存在: {}",
                app_data.display()
            ));
        }

        // 连接到 SQLite 数据库并获取认证信息
        let conn = Connection::open(&app_data)
            .map_err(|e| format!("连接数据库失败 ({}): {}", app_data.display(), e))?;

        // jetski 状态（可选）
        let jetski_state: Option<String> = conn
            .query_row(
                "SELECT value FROM ItemTable WHERE key = 'jetskiStateSync.agentManagerInitState'",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| format!("查询 jetskiStateSync.agentManagerInitState 失败: {}", e))?;

        let state_str = jetski_state
            .ok_or_else(|| "未找到 jetskiStateSync.agentManagerInitState".to_string())?;

        // 解码 jetski 状态（base64 + proto）；失败直接报错
        let decoded = decode_jetski_state_proto(&state_str)?;

        Ok(serde_json::json!(decoded))
    }
    .await;

    let duration = start_time.elapsed();

    match result {
        Ok(data) => {
            tracing::info!(
                duration_ms = duration.as_millis(),
                "获取 Antigravity 信息完成"
            );
            Ok(data)
        }
        Err(e) => {
            tracing::error!(
                error = %e,
                duration_ms = duration.as_millis(),
                "获取 Antigravity 信息失败"
            );
            Err(e)
        }
    }
}

/// 备份当前 Antigravity 账户
pub async fn backup_current() -> Result<String, String> {
    tracing::info!("📥 开始保存 jetskiStateSync.agentManagerInitState");

    let start_time = std::time::Instant::now();

    let result = async {
        // 尝试获取 Antigravity 状态数据库路径
        let app_data = match crate::platform::get_antigravity_db_path() {
            Some(path) => path,
            None => {
                // 如果主路径不存在，尝试其他可能的位置
                let possible_paths = crate::platform::get_all_antigravity_db_paths();
                if possible_paths.is_empty() {
                    return Err("未找到Antigravity安装位置".to_string());
                }
                possible_paths[0].clone()
            }
        };

        if !app_data.exists() {
            return Err(format!(
                "Antigravity 状态数据库文件不存在: {}",
                app_data.display()
            ));
        }

        // 连接到 SQLite 数据库并获取认证信息
        let conn = Connection::open(&app_data)
            .map_err(|e| format!("连接数据库失败 ({}): {}", app_data.display(), e))?;

        // jetski 状态（必需）
        let jetski_state: String = conn
            .query_row(
                "SELECT value FROM ItemTable WHERE key = 'jetskiStateSync.agentManagerInitState'",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| format!("查询 jetskiStateSync.agentManagerInitState 失败: {}", e))?
            .ok_or_else(|| "未找到 jetskiStateSync.agentManagerInitState".to_string())?;

        // 认证状态 (可选)
        let auth_status: Option<String> = conn
            .query_row(
                "SELECT value FROM ItemTable WHERE key = 'antigravityAuthStatus'",
                [],
                |row| row.get(0),
            )
            .optional()
            .unwrap_or(None);

        // 从 jetski proto 解码邮箱（仅用于文件名）
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(jetski_state.trim())
            .map_err(|e| format!("jetskiStateSync Base64 解码失败: {}", e))?;
        let msg = crate::proto::SessionResponse::decode(bytes.as_slice())
            .map_err(|e| format!("jetskiStateSync Protobuf 解码失败: {}", e))?;

        let email = msg
            .context
            .as_ref()
            .and_then(|c| {
                if c.email.is_empty() {
                    None
                } else {
                    Some(c.email.as_str())
                }
            })
            .ok_or_else(|| "jetskiStateSync 中未找到邮箱字段，无法确定备份文件名".to_string())?;

        // 直接保存原始字符串，不解码，文件名与原逻辑保持：{email}.json
        let accounts_dir = crate::directories::get_accounts_directory();
        if let Err(e) = std::fs::create_dir_all(&accounts_dir) {
            return Err(format!("创建账户目录失败: {}", e));
        }

        let account_file = accounts_dir.join(format!("{email}.json"));
        let mut content_map = serde_json::Map::new();
        content_map.insert(
            "jetskiStateSync.agentManagerInitState".to_string(),
            serde_json::Value::String(jetski_state),
        );

        if let Some(status) = auth_status {
             content_map.insert(
                "antigravityAuthStatus".to_string(),
                serde_json::Value::String(status),
            );
        }

        let content = serde_json::Value::Object(content_map);
        std::fs::write(
            &account_file,
            serde_json::to_string_pretty(&content).unwrap(),
        )
        .map_err(|e| format!("写入 jetski 状态失败: {}", e))?;

        let message = format!(
            "已保存 jetskiStateSync.agentManagerInitState 到 {}",
            account_file.display()
        );
        tracing::info!(file = %account_file.display(), "✅ 保存 jetski 状态完成");
        Ok(message)
    }
    .await;

    let duration = start_time.elapsed();

    match result {
        Ok(message) => {
            tracing::info!(
                duration_ms = duration.as_millis(),
                result_message = %message,
                "账户保存操作完成"
            );
            Ok(message)
        }
        Err(e) => {
            tracing::error!(
                error = %e,
                duration_ms = duration.as_millis(),
                "账户保存操作失败"
            );
            Err(e)
        }
    }
}

/// 清除所有 Antigravity 数据
pub async fn clear_all_data() -> Result<String, String> {
    crate::antigravity::cleanup::clear_all_antigravity_data().await
}

/// 恢复 Antigravity 账户
pub async fn restore(account_name: String) -> Result<String, String> {
    tracing::debug!(target: "account::restore", account_name = %account_name, "调用 restore_antigravity_account");

    // 1. 构建备份文件路径
    let accounts_dir = crate::directories::get_accounts_directory();
    let account_file = accounts_dir.join(format!("{account_name}.json"));

    // 2. 调用统一的恢复函数
    crate::antigravity::restore::save_antigravity_account_to_file(account_file).await
}

/// 切换到 Antigravity 账户
///
/// 三分支逻辑：
/// 1. 有扩展连接 → 恢复数据 + 调用扩展 reloadWindow
/// 2. 无扩展 + Antigravity 运行中 → 提示安装扩展
/// 3. 无扩展 + Antigravity 未运行 → 恢复数据 + 启动进程
pub async fn switch(account_name: String) -> Result<String, String> {
        // 检查条件
        let has_extension = crate::server::websocket::has_extension_connections();
        let is_running = crate::platform::is_antigravity_running();

        tracing::info!(
            target: "account::switch",
            has_extension = has_extension,
            is_running = is_running,
            "账户切换条件检查"
        );

        match (has_extension, is_running) {
            // 场景 1: 有扩展连接 → 恢复数据 + reloadWindow
            (true, _) => {
                let client_count = crate::server::websocket::extension_client_count();
                tracing::info!(target: "account::switch::scenario1", client_count = client_count, "使用扩展模式切换");

                // 1. 清除原来的数据库
                clear_all_data().await?;
                tracing::debug!(target: "account::switch::step1", "Antigravity 数据库清除完成");

                // 2. 恢复指定账户到 Antigravity 数据库
                restore(account_name.clone()).await?;
                tracing::debug!(target: "account::switch::step2", "账户数据恢复完成");

                // 3. 等待数据库操作完成
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

                // 4. 调用所有扩展的 reloadWindow 方法（广播）
                crate::server::websocket::call_all_extensions(
                    "reloadWindow",
                    serde_json::json!({}),
                );
                tracing::info!(target: "account::switch::step3", client_count = client_count, "已广播 reloadWindow 到所有扩展");

                Ok(format!(
                    "账户已切换到 {}，正在重载 {} 个 VSCode 窗口",
                    account_name, client_count
                ))
            }

            // 场景 2: 无扩展 + Antigravity 运行中 → 提示安装扩展
            (false, true) => {
                tracing::warn!(target: "account::switch::scenario2", "Antigravity 正在运行但无扩展连接");
                Err("Antigravity 正在运行中，需要安装 VSCode 扩展才能切换账户。\n\n请安装 Antigravity Agent 扩展，扩展会自动重载 Antigravity 窗口。".to_string())
            }

            // 场景 3: 无扩展 + Antigravity 未运行 → 恢复数据 + 启动进程
            (false, false) => {
                // 0. 关闭 Antigravity 进程 (如果存在)
                match crate::platform::kill_antigravity_processes() {
                    Ok(result) => {
                        if result.contains("not found") || result.contains("未找到") {
                            tracing::debug!(target: "account::switch::step1", "Antigravity 进程未运行，跳过关闭步骤");
                            "Antigravity 进程未运行".to_string()
                        } else {
                            tracing::debug!(target: "account::switch::step1", result = %result, "进程关闭完成");
                            result
                        }
                    }
                    Err(e) => {
                        if e.contains("not found") || e.contains("未找到") {
                            tracing::debug!(target: "account::switch::step1", "Antigravity 进程未运行，跳过关闭步骤");
                            "Antigravity 进程未运行".to_string()
                        } else {
                            tracing::error!(target: "account::switch::step1", error = %e, "关闭进程时发生错误");
                            return Err(format!("关闭进程时发生错误: {}", e));
                        }
                    }
                };

                // 等待一秒确保进程完全关闭
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

                tracing::info!(target: "account::switch::scenario3", "Antigravity 未运行，使用进程启动模式");

                // 1. 清除原来的数据库
                clear_all_data().await?;
                tracing::debug!(target: "account::switch::step1", "Antigravity 数据库清除完成");

                // 2. 恢复指定账户到 Antigravity 数据库
                restore(account_name.clone()).await?;
                tracing::debug!(target: "account::switch::step2", "账户数据恢复完成");

                // 3. 等待数据库操作完成
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

                // 4. 启动 Antigravity 进程
                match crate::antigravity::starter::start_antigravity() {
                    Ok(result) => {
                        tracing::info!(target: "account::switch::step3", result = %result, "Antigravity 启动成功");
                        Ok(format!("账户已切换到 {}，已启动 Antigravity", account_name))
                    }
                    Err(e) => {
                        tracing::error!(target: "account::switch::step3", error = %e, "Antigravity 启动失败");
                        Err(format!("账户数据已恢复，但启动 Antigravity 失败: {}", e))
                    }
                }
        }
    }
}

/// 注册新账户 (Process-based restart flow)
pub async fn sign_in_new() -> Result<String, String> {
    println!("🔄 开始执行 sign_in_new 命令");

    // 1. 关闭进程
    let kill_result = match crate::platform::kill_antigravity_processes() {
        Ok(result) => result,
        Err(e) => {
             // 忽略未找到进程的错误
             if e.contains("not found") || e.contains("未找到") {
                 "Antigravity 进程未运行".to_string()
             } else {
                 return Err(format!("关闭进程时发生错误: {}", e));
             }
        }
    };
    
    // 短暂等待
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // 2. 备份当前
    let backup_msg = match backup_current().await {
        Ok(msg) => Some(msg),
        Err(e) => {
            tracing::warn!("备份失败: {}", e);
            None
        }
    };

    // 3. 清除数据
    let _ = clear_all_data().await;

    // 4. 重启
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
    let start_result = crate::antigravity::starter::start_antigravity();
    let start_msg = match start_result {
        Ok(res) => res,
        Err(e) => format!("启动失败: {}", e),
    };

    Ok(format!("{} -> 备份: {:?} -> 重启: {}", kill_result, backup_msg, start_msg))
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct QuotaItem {
    pub model_name: String,
    pub percentage: f64,
    pub reset_text: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct AccountMetrics {
    pub email: String,
    pub user_id: String,
    pub avatar_url: String,
    pub quotas: Vec<QuotaItem>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct TriggerResult {
    pub email: String,
    pub triggered_models: Vec<String>,
    pub failed_models: Vec<String>,
    pub skipped_models: Vec<String>,
    pub skipped_details: Vec<String>,
    pub success: bool,
    pub message: String,
}

pub async fn get_metrics(
    config_dir: &std::path::Path,
    email: String,
) -> Result<AccountMetrics, String> {
    use crate::services::google_api;
    
    // 1. Load Account & Token
    let (email, proto_bytes) = google_api::load_account(config_dir, &email).await?;
    let token_info = google_api::get_valid_token(&email, &proto_bytes).await?;

    // 2. Fetch Models
    let project = google_api::fetch_code_assist_project(&token_info.access_token).await
        .map_err(|e| format!("获取项目 ID 失败: {}", e))?;

    let models_json = google_api::fetch_available_models(&token_info.access_token, &project).await
        .map_err(|e| format!("获取模型列表失败: {}", e))?;

    // 3. Parse Quotas
    let quotas = parse_quotas(&models_json);

    Ok(AccountMetrics {
        email,
        user_id: token_info.user_id,
        avatar_url: token_info.avatar_url,
        quotas,
    })
}

pub async fn trigger_quota_refresh(
    config_dir: &std::path::Path,
    email: String,
) -> Result<TriggerResult, String> {
    use crate::services::google_api;
    use tracing::{info, error};

    info!("🚀 Check Quota & Trigger Refresh for: {}", email);

    // 1. Load Account & Token
    let (email_str, proto_bytes) = google_api::load_account(config_dir, &email).await?;
    let token_info = match google_api::get_valid_token(&email, &proto_bytes).await {
        Ok(t) => t,
        Err(e) => return Err(format!("Auth failed: {}", e)),
    };

    // 2. Get Project ID
    let project = match google_api::fetch_code_assist_project(&token_info.access_token).await {
        Ok(p) => p,
        Err(e) => {
            return Ok(TriggerResult {
                email: email_str,
                triggered_models: Vec::new(),
                failed_models: Vec::new(),
                skipped_models: Vec::new(),
                skipped_details: vec![format!("Account {} has no project ID: {}", email, e)],
                success: false,
                message: format!("Skipped: No project ID found: {}", e),
            });
        }
    };

    // 3. Get Available Models & Quotas
    let models_json = google_api::fetch_available_models(&token_info.access_token, &project).await?;
    let quotas = parse_quotas(&models_json);

    let mut triggered = Vec::new();
    let mut failed = Vec::new();
    let mut skipped = Vec::new();
    let mut skipped_details = Vec::new();

    for item in quotas {
        if item.percentage > 0.9999 {
            // Find key? We need key for trigger.
            // Simplified: we used display name for key mapping in parse_quotas.
            // We need to reverse map or pass key.
            // Let's assume we can map back for now or improve parse_quotas later.
            // For now, let's look up key from name.
            let key = match item.model_name.as_str() {
                "Gemini Pro" => "gemini-3-pro-high",
                "Gemini Flash" => "gemini-3-flash",
                "Gemini Image" => "gemini-3-pro-image",
                "Claude" => "claude-opus-4-5-thinking",
                _ => continue,
            };

            match trigger_minimal_query(&token_info.access_token, &project, key).await {
                Ok(_) => triggered.push(item.model_name.clone()),
                Err(e) => {
                    error!("Trigger failed for {}: {}", item.model_name, e);
                    failed.push(format!("{} ({})", item.model_name, e));
                }
            }
        } else {
             skipped.push(item.model_name.clone());
             skipped_details.push(format!("{} ({:.4}%)", item.model_name, item.percentage * 100.0));
        }
    }

    Ok(TriggerResult {
        email: email_str,
        triggered_models: triggered,
        failed_models: failed,
        skipped_models: skipped,
        skipped_details,
        success: true,
        message: "Refresh cycle check completed".to_string(),
    })
}

fn parse_quotas(models_json: &serde_json::Value) -> Vec<QuotaItem> {
    let mut items = Vec::new();
    let models_map = models_json.get("models").and_then(|v| v.as_object());

    if let Some(map) = models_map {
        let targets = vec![
            ("gemini-3-pro-high", "Gemini Pro"),
            ("gemini-3-flash", "Gemini Flash"),
            ("gemini-3-pro-image", "Gemini Image"),
            ("claude-opus-4-5-thinking", "Claude"),
        ];

        for (key, name) in targets {
            if let Some(model_data) = map.get(key) {
                if let Some(quota_info) = model_data.get("quotaInfo") {
                    let percentage = quota_info
                        .get("remainingFraction")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let reset_text = quota_info
                        .get("resetTime")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    items.push(QuotaItem {
                        model_name: name.to_string(),
                        percentage,
                        reset_text,
                    });
                }
            }
        }
    }
    items
}

async fn trigger_minimal_query(
    access_token: &str,
    project: &str,
    model_key: &str,
) -> Result<(), String> {
    use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!("{}/v1internal:generateContent", crate::services::google_api::CLOUD_CODE_BASE_URL);

    let body = serde_json::json!({
        "project": project,
        "model": model_key,
        "request": {
            "contents": [
                {
                    "role": "user",
                    "parts": [{ "text": format!("Hi [Ref: {}]", chrono::Utc::now().to_rfc3339()) }]
                }
            ],
            "generationConfig": {
                "maxOutputTokens": 10
            }
        }
    });

    let res = client
        .post(&url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(CONTENT_TYPE, "application/json")
        .header(USER_AGENT, "antigravity/windows/amd64")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("API Error {}", res.status()));
    }

    Ok(())
}

/// 检查是否运行中
pub fn is_running() -> bool {
    crate::platform::is_antigravity_running()
}
