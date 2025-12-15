import { invoke } from '@tauri-apps/api/core';

/**
 * 系统托盘命令
 */
export class TrayCommands {
  /**
   * 最小化窗口到托盘
   * @returns 最小化结果消息
   */
  static async minimize(): Promise<string> {
    return invoke('minimize_to_tray');
  }

  /**
   * 从托盘恢复窗口
   * @returns 恢复结果消息
   */
  static async restore(): Promise<string> {
    return invoke('restore_from_tray');
  }

  
  /**
   * 更新托盘菜单
   * @param accounts 账户邮箱列表
   * @returns 更新结果消息
   */
  static async updateMenu(accounts: string[]): Promise<string> {
    return invoke('update_tray_menu_command', { accounts });
  }
}
