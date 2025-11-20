import { open, save } from '@tauri-apps/plugin-dialog';
import { readTextFile, writeTextFile } from '@tauri-apps/plugin-fs';
import { EncryptionService } from '../utils/encryption';
import { AntigravityService } from './antigravity-service';

export interface ConfigData {
  version: string;
  exportTime: string;
  exportUser: string;
  backupCount: number;
  backups: string[];
  settings: {
    theme: string;
    autoBackup: boolean;
  };
  metadata: {
    platform: string;
    userAgent: string;
    antigravityAgent: string;
    encryptionType: string;
  };
}

export interface ImportResult {
  success: boolean;
  message: string;
  encryptedData?: string;
}

/**
 * 配置文件管理器 - 处理导入导出功能
 */
export class ConfigManager {
  /**
   * 导入配置文件
   */
  static async importConfig(
    onStatusUpdate: (message: string, isError?: boolean) => void
  ): Promise<ImportResult> {
    try {
      // 选择文件
      const selected = await open({
        title: '选择要导入的配置文件',
        filters: [
          {
            name: 'Antigravity 加密配置文件',
            extensions: ['enc']
          },
          {
            name: '所有文件',
            extensions: ['*']
          }
        ],
        multiple: false,
      });

      if (!selected) {
        return { success: false, message: '导入取消：未选择文件' };
      }

      onStatusUpdate('正在读取文件...');

      // 使用 Tauri 文件系统 API 读取文件内容
      let encryptedData: string;
      try {
        encryptedData = await readTextFile(selected);
      } catch (readError) {
        throw new Error(`无法读取文件: ${readError}`);
      }

      if (!encryptedData.trim()) {
        throw new Error('文件内容为空');
      }

      return {
        success: true,
        message: '文件读取成功，需要解密',
        encryptedData
      };

    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      return { success: false, message: `选择文件失败: ${errorMessage}` };
    }
  }

  /**
   * 解密配置文件
   */
  static async decryptConfig(
    encryptedData: string,
    password: string
  ): Promise<ConfigData> {
    try {
      const decryptedData = EncryptionService.decrypt(encryptedData, password);
      const configData = JSON.parse(decryptedData) as ConfigData;

      // 验证配置文件格式
      if (!configData.version || !configData.exportTime || !configData.metadata?.antigravityAgent) {
        throw new Error('配置文件格式不正确');
      }

      return configData;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : '解密失败';
      throw new Error(`解密或解析失败：${errorMessage}`);
    }
  }

  /**
   * 导出配置文件
   */
  static async exportConfig(
    password: string,
    onStatusUpdate: (message: string, isError?: boolean) => void
  ): Promise<void> {
    try {
      onStatusUpdate('正在收集配置数据...');

      // 获取当前备份列表
      const backupList = await AntigravityService.getBackupList();

      // 检查是否有用户数据可以导出
      if (backupList.length === 0) {
        onStatusUpdate('没有找到任何用户信息，无法导出配置文件', true);
        throw new Error('没有找到任何用户信息，无法导出配置文件');
      }

      onStatusUpdate(`找到 ${backupList.length} 个用户备份，正在生成加密配置文件...`);

      // 构建增强的配置数据
      const configData: ConfigData = {
        version: '1.1.0',
        exportTime: new Date().toISOString(),
        exportUser: 'Antigravity Agent User',
        backupCount: backupList.length,
        backups: backupList,
        settings: {
          theme: 'dark',
          autoBackup: true,
        },
        metadata: {
          platform: navigator.platform,
          userAgent: navigator.userAgent.substring(0, 100),
          antigravityAgent: 'encrypted_config',
          encryptionType: 'XOR-Base64'
        }
      };

      // 加密数据
      const jsonData = JSON.stringify(configData, null, 2);
      const encryptedData = EncryptionService.encrypt(jsonData, password);

      // 生成带时间戳的文件名
      const timestamp = new Date().toISOString().slice(0, 19).replace(/:/g, '-');
      const defaultFileName = `antigravity_encrypted_config_${timestamp}.enc`;

      onStatusUpdate('正在选择保存位置...');

      // 使用 Tauri 的保存对话框
      const selectedPath = await save({
        title: '保存加密配置文件',
        defaultPath: defaultFileName,
        filters: [
          {
            name: 'Antigravity 加密配置文件',
            extensions: ['enc']
          },
          {
            name: '所有文件',
            extensions: ['*']
          }
        ],
      });

      if (!selectedPath) {
        onStatusUpdate('导出取消：未选择保存位置', true);
        return;
      }

      onStatusUpdate('正在保存配置文件...');

      // 使用 Tauri 文件系统 API 写入文件
      try {
        await writeTextFile(selectedPath, encryptedData);
        onStatusUpdate(`配置文件已保存: ${selectedPath}`);
      } catch (writeError) {
        throw new Error(`保存文件失败: ${writeError}`);
      }

    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      onStatusUpdate(`导出配置文件失败: ${errorMessage}`, true);
      throw error;
    }
  }
}