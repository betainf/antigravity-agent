import { invoke } from '@tauri-apps/api/core';
import type { BackupData, RestoreResult } from './types/backup.types';

/**
 * 备份管理命令
 */
export class BackupCommands {
  /**
   * 获取最近使用的账户列表（按文件修改时间排序）
   * @param limit 返回的最大数量（可选）
   * @returns 账户名列表
   */
  static async getRecentAccounts(limit?: number): Promise<string[]> {
    return invoke('get_recent_accounts', { limit });
  }

  /**
   * 收集所有备份文件的完整内容（用于导出）
   * @returns 备份数据数组
   */
  static async collectContents(): Promise<BackupData[]> {
    return invoke('collect_backup_contents');
  }

  /**
   * 恢复备份文件到本地（用于导入）
   * @param backups 备份数据数组
   * @returns 恢复结果
   */
  static async restoreFiles(backups: BackupData[]): Promise<RestoreResult> {
    return invoke('restore_backup_files', { backups });
  }

  /**
   * 删除指定的备份
   * @param name 账户名
   * @returns 删除结果消息
   */
  static async delete(name: string): Promise<string> {
    return invoke('delete_backup', { name });
  }

  /**
   * 清空所有备份
   * @returns 清空结果消息
   */
  static async clearAll(): Promise<string> {
    return invoke('clear_all_backups');
  }
}
