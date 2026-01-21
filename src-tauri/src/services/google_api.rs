use base64::Engine;
use prost::Message;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use tracing::{error, info};

pub const CLOUD_CODE_BASE_URL: &str = "https://daily-cloudcode-pa.sandbox.googleapis.com";

#[derive(Deserialize)]
pub struct UserInfoResponse {
    pub id: String,
    pub picture: String,
}

#[derive(Deserialize)]
pub struct RefreshTokenResponse {
    pub access_token: String,
}

pub struct ValidToken {
    pub access_token: String,
    pub user_id: String,
    pub avatar_url: String,
}

pub async fn load_account(
    config_dir: &std::path::Path,
    target_email: &str,
) -> Result<(String, Vec<u8>), String> {
    let antigravity_dir = config_dir.join("antigravity-accounts");
    let path = antigravity_dir.join(format!("{}.json", target_email));

    if !path.exists() {
        return Err(format!("账户文件不存在: {}", path.display()));
    }

    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let json: Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    if let Some(state_str) = json
        .get("jetskiStateSync.agentManagerInitState")
        .and_then(|v| v.as_str())
    {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(state_str.trim())
            .map_err(|e| e.to_string())?;

        return Ok((target_email.to_string(), bytes));
    }

    Err("无效的账户文件格式".to_string())
}

pub async fn get_valid_token(email: &str, proto_bytes: &[u8]) -> Result<ValidToken, String> {
    let mut msg = crate::proto::SessionResponse::decode(proto_bytes)
        .map_err(|e| format!("Proto decode failed: {}", e))?;

    let auth = msg.auth.as_mut().ok_or("No auth info")?;
    let access_token = auth.access_token.clone();
    let refresh_token = auth.refresh_token.clone();
    let _email_ctx = msg.context.as_ref().map(|c| c.email.clone()).unwrap_or_default();

    // Verify token and get user info
    match fetch_user_info(&access_token).await {
        Ok(info) => Ok(ValidToken {
            access_token,
            user_id: info.id,
            avatar_url: info.picture,
        }),
        Err(_) => {
            info!("Token expired for {}, refreshing...", email);
            let new_token = refresh_access_token(&refresh_token).await?;
            // Verify new token
            let info = fetch_user_info(&new_token).await.map_err(|e| format!("Failed to verify new token: {}", e))?;
            Ok(ValidToken {
                access_token: new_token,
                user_id: info.id,
                avatar_url: info.picture,
            })
        }
    }
}

pub async fn fetch_user_info(access_token: &str) -> Result<UserInfoResponse, String> {
    let client = reqwest::Client::new();
    let res = client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("Status: {}", res.status()));
    }

    res.json::<UserInfoResponse>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn refresh_access_token(refresh_token: &str) -> Result<String, String> {
    // 使用安全的凭据管理模块获取 OAuth 凭据
    let config_dir = crate::directories::get_config_directory();
    let (client_id, client_secret) = crate::security::credentials::resolve_oauth_credentials(&config_dir)?;
    
    let client = reqwest::Client::new();
    let params = [
        ("client_id", client_id.as_str()),
        ("client_secret", client_secret.as_str()),
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
    ];

    let res = client
        .post("https://oauth2.googleapis.com/token")
        .form(&params)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("Refresh failed: {}", res.status()));
    }

    let json: RefreshTokenResponse = res.json().await.map_err(|e| e.to_string())?;
    Ok(json.access_token)
}

pub async fn fetch_code_assist_project(access_token: &str) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;

    let res = client
        .post(format!("{}/v1internal:loadCodeAssist", CLOUD_CODE_BASE_URL))
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(CONTENT_TYPE, "application/json")
        .header(USER_AGENT, "antigravity/windows/amd64")
        .body(r#"{"metadata": {"ideType": "ANTIGRAVITY"}}"#)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = res.status();
    let text = res.text().await.map_err(|e| e.to_string())?;

    if !status.is_success() {
        return Err(format!("loadCodeAssist failed status {}: {}", status, text));
    }

    let json: Value = serde_json::from_str(&text).map_err(|e| {
        format!(
            "Failed to parse project response: {} | Raw Body: {:.100}",
            e, text
        )
    })?;

    let project_id = json
        .get("cloudaicompanionProject")
        .or_else(|| json.get("project"))
        .or_else(|| json.get("projectId"))
        .and_then(|v| v.as_str());

    match project_id {
        Some(id) => Ok(id.to_string()),
        None => Err("Project ID missing in loadCodeAssist response".to_string()),
    }
}

pub async fn fetch_available_models(access_token: &str, project: &str) -> Result<Value, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;

    let body = serde_json::json!({ "project": project });

    let res = client
        .post(format!(
            "{}/v1internal:fetchAvailableModels",
            CLOUD_CODE_BASE_URL
        ))
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(CONTENT_TYPE, "application/json")
        .header(USER_AGENT, "antigravity/windows/amd64")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = res.status();
    let text = res.text().await.map_err(|e| e.to_string())?;

    if !status.is_success() {
        return Err(format!(
            "fetchAvailableModels failed status {}: {}",
            status, text
        ));
    }

    serde_json::from_str(&text).map_err(|e| {
        error!("JSON parse failed for fetchAvailableModels. Raw body: {}", text);
        format!("Failed to parse models JSON: {} | Raw Body: {:.500}", e, text)
    })
}
