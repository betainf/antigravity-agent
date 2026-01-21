//! 账户导入导出加密模块
//!
//! 使用 ChaCha20-Poly1305 认证加密 + Argon2id 密钥派生
//!
//! 输出格式（Base64 编码）：
//! [version: 1 byte][salt: 16 bytes][nonce: 12 bytes][ciphertext + tag]

use argon2::{Algorithm, Argon2, ParamsBuilder, Version};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use rand::RngCore;
use zeroize::Zeroize;

/// 当前加密格式版本
const CRYPTO_VERSION: u8 = 1;

/// Salt 长度（字节）
const SALT_LEN: usize = 16;

/// Nonce 长度（字节）
const NONCE_LEN: usize = 12;

/// 使用密码加密配置数据（用于账户导出）
///
/// 算法：Argon2id 密钥派生 + ChaCha20-Poly1305 认证加密
pub async fn encrypt_config_data(json_data: String, password: String) -> Result<String, String> {
    if password.is_empty() {
        return Err("密码不能为空".to_string());
    }

    // 生成随机 salt
    let mut salt_bytes = [0u8; SALT_LEN];
    rand::thread_rng().fill_bytes(&mut salt_bytes);

    // 派生密钥
    let mut key = derive_key(&password, &salt_bytes)?;

    // 生成随机 nonce
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // 加密
    let cipher = ChaCha20Poly1305::new_from_slice(&key)
        .map_err(|e| format!("初始化加密器失败: {}", e))?;

    let ciphertext = cipher
        .encrypt(nonce, json_data.as_bytes())
        .map_err(|e| format!("加密失败: {}", e))?;

    // 清除密钥
    key.zeroize();

    // 组装输出: version + salt + nonce + ciphertext
    let mut output = Vec::with_capacity(1 + SALT_LEN + NONCE_LEN + ciphertext.len());
    output.push(CRYPTO_VERSION);
    output.extend_from_slice(&salt_bytes);
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);

    Ok(BASE64.encode(&output))
}

/// 使用密码解密配置数据（用于账户导入）
pub async fn decrypt_config_data(
    encrypted_data: String,
    password: String,
) -> Result<String, String> {
    if password.is_empty() {
        return Err("密码不能为空".to_string());
    }

    let data = BASE64
        .decode(&encrypted_data)
        .map_err(|_| "Base64 解码失败，文件可能已损坏".to_string())?;

    // 检查最小长度
    let min_len = 1 + SALT_LEN + NONCE_LEN + 16; // 16 是 Poly1305 tag 长度
    if data.len() < min_len {
        return Err("加密数据格式无效".to_string());
    }

    // 解析格式
    let version = data[0];
    if version != CRYPTO_VERSION {
        return Err(format!("不支持的加密格式版本: {}", version));
    }

    let salt = &data[1..1 + SALT_LEN];
    let nonce_bytes = &data[1 + SALT_LEN..1 + SALT_LEN + NONCE_LEN];
    let ciphertext = &data[1 + SALT_LEN + NONCE_LEN..];

    // 派生密钥
    let mut key = derive_key(&password, salt)?;

    // 解密
    let cipher = ChaCha20Poly1305::new_from_slice(&key)
        .map_err(|e| format!("初始化解密器失败: {}", e))?;

    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "解密失败：密码错误或数据已损坏".to_string())?;

    // 清除密钥
    key.zeroize();

    String::from_utf8(plaintext).map_err(|_| "解密后的数据不是有效的 UTF-8 文本".to_string())
}

/// 使用 Argon2id 从密码派生 32 字节密钥
fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32], String> {
    // 使用适中的参数（内存 64MB，3 次迭代，4 并行度）
    let params = ParamsBuilder::new()
        .m_cost(65536) // 64 MB
        .t_cost(3)
        .p_cost(4)
        .output_len(32)
        .build()
        .map_err(|e| format!("构建 Argon2 参数失败: {}", e))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    // 使用 hash_password_into 获取原始字节
    let mut key = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| format!("密钥派生失败: {}", e))?;

    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_encrypt_decrypt_roundtrip() {
        let original = r#"{"email": "test@example.com", "token": "secret123"}"#.to_string();
        let password = "test_password".to_string();

        let encrypted = encrypt_config_data(original.clone(), password.clone())
            .await
            .expect("加密失败");

        assert_ne!(encrypted, original);
        assert!(encrypted.len() > original.len());

        let decrypted = decrypt_config_data(encrypted, password)
            .await
            .expect("解密失败");

        assert_eq!(decrypted, original);
    }

    #[tokio::test]
    async fn test_wrong_password_fails() {
        let original = "test data".to_string();
        let password = "correct_password".to_string();

        let encrypted = encrypt_config_data(original, password)
            .await
            .expect("加密失败");

        let result = decrypt_config_data(encrypted, "wrong_password".to_string()).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("密码错误"));
    }

    #[tokio::test]
    async fn test_empty_password_rejected() {
        let result = encrypt_config_data("data".to_string(), "".to_string()).await;
        assert!(result.is_err());

        let result = decrypt_config_data("data".to_string(), "".to_string()).await;
        assert!(result.is_err());
    }
}
