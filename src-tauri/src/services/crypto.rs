use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};

/// 加密配置数据（用于账户导出）
pub async fn encrypt_config_data(json_data: String, password: String) -> Result<String, String> {
    if password.is_empty() {
        return Err("密码不能为空".to_string());
    }

    let password_bytes = password.as_bytes();
    let mut result = Vec::new();

    // XOR 加密
    for (i, byte) in json_data.as_bytes().iter().enumerate() {
        let key_byte = password_bytes[i % password_bytes.len()];
        result.push(byte ^ key_byte);
    }

    // Base64 编码
    let encoded = BASE64.encode(&result);

    Ok(encoded)
}

/// 解密配置数据（用于账户导入）
pub async fn decrypt_config_data(
    encrypted_data: String,
    password: String,
) -> Result<String, String> {
    if password.is_empty() {
        return Err("密码不能为空".to_string());
    }

    let decoded = BASE64
        .decode(encrypted_data)
        .map_err(|_| "Base64 解码失败".to_string())?;

    let password_bytes = password.as_bytes();
    let mut result = Vec::new();

    for (i, byte) in decoded.iter().enumerate() {
        let key_byte = password_bytes[i % password_bytes.len()];
        result.push(byte ^ key_byte);
    }

    let decrypted =
        String::from_utf8(result).map_err(|_| "解密失败，数据可能已损坏".to_string())?;

    Ok(decrypted)
}
