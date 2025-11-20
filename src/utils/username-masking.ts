/**
 * 对用户名进行中间打码处理
 * @param username 原始用户名
 * @returns 打码后的用户名
 */
export const maskUsername = (username: string): string => {
  if (!username || username.length <= 1) {
    return username;
  }

  // 如果只有2个字符，显示第一个字符 + *
  if (username.length === 2) {
    return username.charAt(0) + '*';
  }

  // 如果是3个字符，显示首字符 + * + 尾字符
  if (username.length === 3) {
    return username.charAt(0) + '*' + username.charAt(2);
  }

  // 4个字符及以上，显示首尾字符，中间用*代替
  const firstChar = username.charAt(0);
  const lastChar = username.charAt(username.length - 1);
  const middleStars = '*'.repeat(username.length - 2);

  return firstChar + middleStars + lastChar;
};

/**
 * 生成用户名显示的title属性
 * @param username 原始用户名
 * @returns title文本
 */
export const getUsernameTitle = (username: string): string => {
  return `完整用户名: ${username}`;
};

/**
 * 从备份文件名中提取用户名部分进行打码，时间戳保持不变
 * 支持格式：{用户名}_{时间戳} 或 {时间戳}
 * @param backupFile 备份文件名
 * @returns 打码后的备份文件名
 */
export const maskBackupFilename = (backupFile: string): string => {
  if (!backupFile) {
    return backupFile;
  }

  // 检查是否是用户名+时间戳格式：{用户名}_{时间戳}
  const userTimestampPattern = /^(.+?)_(\d{4}-\d{2}-\d{2}T\d{2}-\d{2}-\d{2})$/;
  const match = backupFile.match(userTimestampPattern);

  if (match) {
    // 提取用户名和时间戳
    const originalUsername = match[1];
    const timestamp = match[2];

    // 只对用户名部分进行打码
    const maskedUsername = maskUsername(originalUsername);

    // 重新组合：{打码用户名}_{时间戳}
    return `${maskedUsername}_${timestamp}`;
  }

  // 检查是否是纯时间戳格式：{时间戳}
  const timestampPattern = /^(\d{4}-\d{2}-\d{2}T\d{2}-\d{2}-\d{2})$/;
  const timestampMatch = backupFile.match(timestampPattern);
  if (timestampMatch) {
    // 纯时间戳，直接返回
    return backupFile;
  }

  // 如果不是以上格式，直接对整个文件名进行打码
  return maskUsername(backupFile);
};

/**
 * 生成备份文件名的title属性
 * @param backupFile 备份文件名
 * @returns title文本
 */
export const getBackupFileTitle = (backupFile: string): string => {
  return `完整备份文件名: ${backupFile}`;
};