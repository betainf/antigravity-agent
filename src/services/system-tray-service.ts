/**
 * 系统托盘服务
 *
 * 提供系统托盘功能的简化前端接口 - 所有逻辑由后端处理
 */

import { invoke } from '@tauri-apps/api/core';

export interface SystemTrayStatus {
  enabled: boolean;
  message: string;
}

export interface SystemTrayFullStatus {
  runtime_enabled: boolean;
  tray_exists: boolean;
  saved_state: boolean;
  is_consistent: boolean;
}

/**
 * 系统托盘服务类 - 纯调用接口，所有逻辑由后端处理
 */
export class SystemTrayService {
  /**
   * 获取系统托盘状态（持久化状态）
   */
  static async getSystemTrayState(): Promise<boolean> {
    try {
      return await invoke<boolean>('get_system_tray_state');
    } catch (error) {
      console.warn('获取系统托盘状态失败，使用默认值:', error);
      return true; // 默认启用
    }
  }

  /**
   * 切换系统托盘状态（纯后端处理所有逻辑）
   * 后端会自动处理状态检查、防重复创建、状态同步等
   */
  static async toggleSystemTray(): Promise<SystemTrayStatus> {
    try {
      const result = await invoke<any>('toggle_system_tray');

      return {
        enabled: result.enabled,
        message: result.message
      };
    } catch (error) {
      // 出错时返回当前状态
      const currentState = await this.getSystemTrayState();
      return {
        enabled: currentState,
        message: `操作失败: ${error}`
      };
    }
  }

  /**
   * 同步系统托盘状态（确保状态一致性）
   * 后端会检查所有状态并自动修复不一致问题
   */
  static async syncSystemTrayState(): Promise<string> {
    try {
      return await invoke<string>('sync_system_tray_state');
    } catch (error) {
      throw new Error(`同步状态失败: ${error}`);
    }
  }

  /**
   * 获取系统托盘完整状态信息（用于调试）
   */
  static async getSystemTrayFullStatus(): Promise<SystemTrayFullStatus> {
    try {
      return await invoke<SystemTrayFullStatus>('get_system_tray_status');
    } catch (error) {
      console.warn('获取系统托盘完整状态失败:', error);
      return {
        runtime_enabled: false,
        tray_exists: false,
        saved_state: true,
        is_consistent: false
      };
    }
  }

  /**
   * 检查系统托盘是否启用（运行时状态）
   */
  static async isSystemTrayEnabled(): Promise<boolean> {
    return await invoke<boolean>('is_system_tray_enabled');
  }
}