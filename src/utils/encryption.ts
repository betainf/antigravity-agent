/**
 * 加密工具类 - 提供Unicode安全的加密/解密功能
 */
export class EncryptionService {
  /**
   * Unicode安全的base64编码函数
   */
  static unicodeBase64Encode(str: string): string {
    try {
      // 首先尝试使用TextEncoder来处理Unicode
      const encoder = new TextEncoder();
      const data = encoder.encode(str);
      let binary = '';
      for (let i = 0; i < data.byteLength; i++) {
        binary += String.fromCharCode(data[i]);
      }
      return btoa(binary);
    } catch (error) {
      // 如果TextEncoder失败，使用fallback方法
      return btoa(unescape(encodeURIComponent(str)));
    }
  }

  /**
   * Unicode安全的base64解码函数
   */
  static unicodeBase64Decode(str: string): string {
    try {
      // 首先尝试使用TextDecoder来处理Unicode
      const binary = atob(str);
      const bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) {
        bytes[i] = binary.charCodeAt(i);
      }
      const decoder = new TextDecoder();
      return decoder.decode(bytes);
    } catch (error) {
      // 如果TextDecoder失败，使用fallback方法
      try {
        return decodeURIComponent(escape(atob(str)));
      } catch (fallbackError) {
        throw new Error('Base64解码失败');
      }
    }
  }

  /**
   * 简单的XOR加密函数
   */
  static encrypt(data: string, password: string): string {
    let result = '';
    for (let i = 0; i < data.length; i++) {
      result += String.fromCharCode(
        data.charCodeAt(i) ^ password.charCodeAt(i % password.length)
      );
    }
    return this.unicodeBase64Encode(result);
  }

  /**
   * 简单的XOR解密函数
   */
  static decrypt(encryptedData: string, password: string): string {
    try {
      const data = this.unicodeBase64Decode(encryptedData);
      let result = '';
      for (let i = 0; i < data.length; i++) {
        result += String.fromCharCode(
          data.charCodeAt(i) ^ password.charCodeAt(i % password.length)
        );
      }
      return result;
    } catch (error) {
      throw new Error('解密失败：数据格式错误或密码不正确');
    }
  }

  /**
   * 密码验证函数
   */
  static validatePassword(password: string): { isValid: boolean; message?: string } {
    if (!password || password.length < 1) {
      return { isValid: false, message: '请输入密码' };
    }
    return { isValid: true };
  }
}