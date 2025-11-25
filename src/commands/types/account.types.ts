/**
 * Antigravity 账户相关类型定义
 */

/**
 * Antigravity 账户认证信息
 * 注意：后端返回的是动态 JSON (serde_json::Value)，可能包含额外字段
 */
export interface AntigravityAuthInfo {
  /** 用户邮箱地址 */
  email?: string;

  /** 数据库文件路径 */
  db_path?: string;

  /** 其他可能的认证相关字段（支持动态扩展） */
  [key: string]: unknown;
}

/**
 * Antigravity 账户信息
 */
export interface AntigravityAccount {
  /** 账户ID */
  id: string;

  /** 账户名称 */
  name: string;

  /** 邮箱地址 */
  email: string;

  /** API密钥或访问令牌 */
  api_key: string;

  /** Base64 编码的头像URL */
  profile_url: string;

  /** 编码后的用户设置 */
  user_settings: string;

  /** 账户创建时间 */
  created_at: string;

  /** 最后切换时间 */
  last_switched: string;
}
