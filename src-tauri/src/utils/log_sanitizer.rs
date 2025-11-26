//! 日志脱敏模块
//! 对敏感信息进行智能遮盖，保护用户隐私的同时保留调试价值

use regex::Regex;

/// 日志脱敏器
pub struct LogSanitizer {
    /// 邮箱正则表达式
    email_regex: Regex,
    /// API密钥正则表达式
    api_key_regex: Regex,
    /// 用户主目录正则表达式
    user_home_regex: Regex,
    /// Windows用户目录正则表达式
    windows_user_regex: Regex,
}

impl Default for LogSanitizer {
    fn default() -> Self {
        Self {
            email_regex: Regex::new(r"(?i)[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap(),
            api_key_regex: Regex::new(r"(?i)(?P<prefix>key|token|secret|api[-_]?key|access[-_]?token)[\s=:]+(?P<key>[a-zA-Z0-9+/=_-]{20,})").unwrap(),
            user_home_regex: Regex::new(r"(?P<prefix>/home/[^/]+)").unwrap(),
            windows_user_regex: Regex::new(r"C:\\\\Users\\\\[^\\\\]+").unwrap(),
        }
    }
}

impl LogSanitizer {
    /// 创建新的脱敏器实例
    pub fn new() -> Self {
        Self::default()
    }

    /// 对字符串进行脱敏处理
    pub fn sanitize(&self, input: &str) -> String {
        let mut result = input.to_string();

        // 1. 脱敏邮箱地址
        result = self.sanitize_email(&result);

        // 2. 使用基础的路径脱敏
        result = self.sanitize_paths(&result);

        // 3. 脱敏API密钥
        result = self.sanitize_api_keys(&result);

        result
    }

    /// 智能邮箱脱敏 - 保留首尾字符，中间用*替代
    ///
    /// 策略：
    /// - 1个字符：保留原样
    /// - 2个字符：显示首字符 + *
    /// - 3个及以上：显示首字符 + 中间* + 尾字符
    ///
    /// # 示例
    /// ```
    /// "a@domain.com" → "a@domain.com"
    /// "ab@domain.com" → "a*@domain.com"
    /// "user@domain.com" → "u***r@domain.com"
    /// "very.long.email@domain.com" → "v***l@domain.com"
    /// ```
    pub fn sanitize_email(&self, input: &str) -> String {
        self.email_regex.replace_all(input, |caps: &regex::Captures| {
            let email = &caps[0];

            let at_pos = email.find('@').unwrap_or(0);
            let (local_part, domain) = email.split_at(at_pos);

            match local_part.len() {
                0 | 1 => email.to_string(),
                2 => {
                    let first_char = local_part.chars().next().unwrap_or('_');
                    format!("{}*{}", first_char, domain)
                }
                _ => {
                    let first_char = local_part.chars().next().unwrap_or('_');
                    let last_char = local_part.chars().last().unwrap_or('_');
                    let middle_stars = "*".repeat(local_part.len().saturating_sub(2).saturating_sub(2));
                    format!("{}{}{}@{}", first_char, middle_stars, last_char, domain)
                }
            }
        }).to_string()
    }

    /// 路径脱敏 - 隐藏用户主目录部分
    ///
    /// # 示例
    /// ```
    /// "/home/user/.antigravity-agent" → "~/.antigravity-agent"
    /// "/home/user/Documents/file.txt" → "~/Documents/file.txt"
    /// "C:\\Users\\Kiki\\AppData" → "~\\AppData"
    /// "C:\\Users\\Kiki\\AppData\\Roaming\\Antigravity" → "~\\AppData\\Roaming\\Antigravity"
    /// ```
    pub fn sanitize_paths(&self, input: &str) -> String {
        let mut result = input.to_string();

        // 处理 Linux/Unix 路径
        result = self.user_home_regex.replace_all(&result, |_caps: &regex::Captures| {
            "~"
        }).to_string();

        // 处理 Windows 路径 - 修正正则表达式匹配用户名
        result = self.windows_user_regex.replace_all(&result, |_caps: &regex::Captures| {
            "~"
        }).to_string();

        // 额外处理一些可能遗漏的路径格式
        if result.contains("C:\\Users\\") {
            // 使用更简单的替换方式
            result = regex::Regex::new(r"C:\\\\Users\\\\[^\\\\]+")
                .unwrap()
                .replace_all(&result, "~")
                .to_string();
        }

        result
    }

    /// API密钥脱敏 - 只显示前几个字符，后面用*替代
    ///
    /// # 示例
    /// ```
    /// "api_key: sk-1234567890abcdef" → "api_key: sk-12****************"
    /// "token: abcdef1234567890" → "token: ab****************"
    /// ```
    pub fn sanitize_api_keys(&self, input: &str) -> String {
        self.api_key_regex.replace_all(input, |caps: &regex::Captures| {
            let prefix = &caps["prefix"];
            let key = &caps["key"];
            let visible_len = std::cmp::min(4, key.len());
            let masked_len = key.len().saturating_sub(visible_len);

            if key.len() <= 4 {
                format!("{}{}", prefix, key)
            } else {
                let visible_part = &key[..visible_len];
                let masked_part = "*".repeat(masked_len);
                format!("{}{}{}", prefix, visible_part, masked_part)
            }
        }).to_string()
    }


  }

