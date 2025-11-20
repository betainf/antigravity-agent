import { invoke } from '@tauri-apps/api/core';

/**
 * Antigravity 服务 - 处理 Antigravity 相关操作
 */
export class AntigravityService {
  /**
   * 备份并重启Antigravity
   */
  static async backupAndRestartAntigravity(
    onStatusUpdate: (message: string, isError?: boolean) => void
  ): Promise<void> {
    try {
      console.log('🚀 开始执行备份并重启 Antigravity 流程');
      onStatusUpdate('正在关闭 Antigravity 进程...');

      console.log('📞 调用后端 backup_and_restart_antigravity 命令');
      const result = await invoke('backup_and_restart_antigravity') as string;
      console.log('✅ 后端命令执行成功，结果:', result);

      onStatusUpdate(result);

    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      console.error('❌ 备份并重启失败:', errorMessage);
      console.error('❌ 完整错误对象:', error);
      throw new Error(`备份并重启失败: ${errorMessage}`);
    }
  }

  /**
   * 获取备份列表
   */
  static async getBackupList(): Promise<string[]> {
    try {
      const backupList = await invoke('list_backups') as string[];
      if (!backupList) {
        throw new Error('无法获取备份列表');
      }
      return backupList;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      throw new Error(`获取备份列表失败: ${errorMessage}`);
    }
  }
}