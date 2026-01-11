
// Antigravity 当前用户信息类型
export interface AntigravityAccount {
  auth: Auth | null
  context: Context
  field_5_base64: string | null
  field_7_base64: string | null
  field_9_base64: string | null
  field_10_base64: string | null
  field_11_base64: string | null
  field_15_base64: string | null
  field_16_base64: string | null
  field_17_base64: string | null
  f18_base64: string | null
  subscription: Subscription | null
}

interface Auth {
  has_access_token: boolean
  has_refresh_token: boolean
  token_type: string
  created_at: number | null  // 原 meta.expiry_timestamp
}

interface Context {
  email: string
  models: Models | null
  plan: Subscription | null  // 使用 Subscription 类型
  plan_name: string
  status: number
}

interface Models {
  items: Item[]
  recommended: Recommended | null
  default_model: number | null
}

interface Item {
  name: string
  id: number | null
  field_5: number
  field_11: number
  tag: string
  supported_types: string[]
}

interface Recommended {
  category: string
  model_names: string[] | null
}

// 订阅信息
interface Subscription {
  tier_id: string
  tier_name: string
  display_name: string
  upgrade_url: string
  upgrade_message: string
}

// 对应 Rust 的 AccountMetrics 结构
export interface QuotaItem {
  model_name: string;
  percentage: number;
  reset_text: string;
}

export interface AccountMetrics {
  email: string;
  user_id: string;
  avatar_url: string;
  quotas: QuotaItem[];
}
