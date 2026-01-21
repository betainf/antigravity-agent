//! 安全模块 - 独立于上游代码，避免合并冲突
//!
//! 包含：
//! - `credentials`: OAuth 凭据安全管理（系统凭据存储）
//! - `crypto`: 账户导入导出加密（ChaCha20-Poly1305）

pub mod credentials;
pub mod crypto;
