/**
 * 静默启动服务
 *
 * 提供静默启动功能的前端接口
 */

import { invoke } from '@tauri-apps/api/core';
import { logger } from '../utils/logger';

export interface SilentStartStatus {
  enabled: boolean;
  message: string;
}

/**
 * 静默启动服务类
 */
export class SilentStartService {
  /**
   * 获取静默启动状态
   */
  static async getSilentStartState(): Promise<boolean> {
    try {
      return await invoke<boolean>('is_silent_start_enabled');
    } catch (error) {
      logger.warn('获取静默启动状态失败，使用默认值', {
        module: 'SilentStartService',
        action: 'get_state_failed',
        error: error instanceof Error ? error.message : String(error)
      });
      return false; // 默认不启用
    }
  }

  /**
   * 切换静默启动状态
   */
  static async setSilentStartEnabled(enabled: boolean): Promise<SilentStartStatus> {
    try {
      const result = await invoke<string>('save_silent_start_state', { enabled });

      return {
        enabled,
        message: result
      };
    } catch (error) {
      // 出错时返回错误信息
      return {
        enabled: false,
        message: `操作失败: ${error}`
      };
    }
  }

}
