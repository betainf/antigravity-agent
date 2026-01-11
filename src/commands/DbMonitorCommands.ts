import { universalInvoke } from '@/lib/invoke-adapter';

/**
 * 数据库监控命令
 */
export class DbMonitorCommands {
  /**
   * 检查数据库监控是否正在运行
   * @returns 是否正在运行
   */
  static async isRunning(): Promise<boolean> {
    return universalInvoke('is_database_monitoring_running');
  }

  /**
   * 启动数据库监控
   * @returns 启动结果消息
   */
  static async start(): Promise<string> {
    return universalInvoke('start_database_monitoring');
  }

  /**
   * 停止数据库监控
   * @returns 停止结果消息
   */
  static async stop(): Promise<string> {
    return universalInvoke('stop_database_monitoring');
  }
}
