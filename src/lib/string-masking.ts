/**
 * 对邮箱字符串进行打码脱敏。
 *
 * 规则：
 * 1. @ 前（本地部分 local）：
 *    - 长度 = 1：全部打码 => `*`
 *    - 长度 = 2：全部打码 => `**`
 *    - 长度 ≥ 3：保留首尾，中间用 `*` 打码，
 *      且中间连续 `*` 的数量最多为 3 个
 *      例如：
 *        "abc"      -> "a*c"
 *        "abcd"     -> "a**d"
 *        "abcde"    -> "a***e"
 *        "abcdefg"  -> "a***g"  （多出来的也仍然只显示 3 个 *）
 *
 * 2. @ 后（域名部分 domain）：
 *    - 使用 '.' 分割
 *    - “最后一级域名” = 最后两段（如 gmail.com / company.cn），原样保留
 *    - 前面的所有段统一替换为 `**`
 *      例如：
 *        "gmail.com"                -> "gmail.com"
 *        "bin.gmail.com"            -> "**.gmail.com"
 *        "sub.corp.company.com"     -> "**.**.company.com"
 *
 * 3. 若字符串不符合邮箱基本格式（@ 不在中间），则原样返回。
 */
export function maskEmail(email: string): string {
  const atIndex = email.indexOf("@");

  // 没有 @，或者 @ 在首/尾，认为不是正常邮箱，直接原样返回
  if (atIndex <= 0 || atIndex === email.length - 1) {
    return email;
  }

  const local = email.slice(0, atIndex);
  const domain = email.slice(atIndex + 1);

  const maskedLocal = maskLocal(local);
  const maskedDomain = maskDomain(domain);

  return `${maskedLocal}@${maskedDomain}`;
}

/**
 * 按规则打码本地部分（@ 前）
 * - 中间连续 * 最多 3 个
 */
function maskLocal(local: string): string {
  const chars = [...local]; // 支持多字节字符（如中文）
  const n = chars.length;

  if (n === 0) return "";

  if (n === 1) {
    // 1 个字符：全部打码
    return "*";
  }

  if (n === 2) {
    // 2 个字符：全部打码
    return "**";
  }

  // 3 个及以上：保留首尾，中间最多 3 个 *
  const head = chars[0];
  const tail = chars[n - 1];
  const middleCount = Math.min(3, n - 2);
  const middle = "*".repeat(middleCount);

  return head + middle + tail;
}

/**
 * 按规则打码域名部分（@ 后）：
 * - 最后一级域名（最后两段）保留
 * - 前面的所有段统一替换为 `**`
 */
function maskDomain(domain: string): string {
  const parts = domain.split(".");

  // 只有 1 或 2 段：整个域名就是“最后一级”，直接保留
  if (parts.length <= 2) {
    return domain;
  }

  // 最后两段是“最后一级域名”，保留；前面的全部打成 `**`
  const lastTwo = parts.slice(-2);           // [second-level, tld]
  const front = parts.slice(0, -2).map(() => "**");

  return [...front, ...lastTwo].join(".");
}

export const maskName = (username: string): string => {
  if (!username) return "";
  const trimmed = username.trim(); // 👈 去掉前后空格
  const chars = [...trimmed];
  const n = chars.length;

  if (n === 0) return "";
  if (n === 1) return "*";
  if (n === 2) return chars[0] + "*";

  const head = chars[0];
  const tail = chars[n - 1];
  const middle = "*".repeat(n - 2);

  return head + middle + tail;
}
