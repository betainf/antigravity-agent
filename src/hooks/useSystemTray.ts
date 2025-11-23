/**
 * 系统托盘 Hook - 简化版
 *
 * 所有复杂逻辑都由后端处理，前端只负责：
 * - 基本状态管理
 * - 调用后端接口
 * - UI 反馈
 */

import { useState, useEffect, useCallback, useRef } from 'react';
import { SystemTrayService, SystemTrayStatus } from '../services/system-tray-service';

interface UseSystemTrayOptions {
  /** 状态变化时的回调函数 */
  onStatusChange?: (enabled: boolean, message?: string) => void;
  /** 是否在组件挂载时自动初始化 */
  autoInit?: boolean;
}

interface UseSystemTrayReturn {
  /** 当前系统托盘是否启用 */
  enabled: boolean;
  /** 是否正在执行操作 */
  isLoading: boolean;
  /** 是否已初始化 */
  isInitialized: boolean;
  /** 错误信息 */
  error: string | null;
  /** 切换系统托盘状态（后端自动处理所有逻辑） */
  toggle: () => Promise<void>;
  /** 重新初始化 */
  reinitialize: () => Promise<void>;
}

/**
 * 系统托盘管理 Hook - 简化版
 */
export function useSystemTray(options: UseSystemTrayOptions = {}): UseSystemTrayReturn {
  const {
    onStatusChange,
    autoInit = true
  } = options;

  // 状态管理
  const [enabled, setEnabled] = useState<boolean>(false);
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [isInitialized, setIsInitialized] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  // 防重复操作标志
  const isOperatingRef = useRef<boolean>(false);

  /**
   * 清除错误信息
   */
  const clearError = useCallback(() => {
    setError(null);
  }, []);

  /**
   * 初始化系统托盘状态
   */
  const initialize = useCallback(async (): Promise<void> => {
    try {
      // 获取持久化状态作为UI显示状态
      const savedEnabled = await SystemTrayService.getSystemTrayState();

      setEnabled(savedEnabled);
      setIsInitialized(true);
      clearError();

      // 新的 toggle_system_tray 命令会自动处理状态同步，不需要手动同步

      onStatusChange?.(savedEnabled);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : '未知错误';
      setError(`初始化系统托盘失败: ${errorMessage}`);
      setIsInitialized(true);

      // 出错时使用默认启用状态
      setEnabled(true);
      onStatusChange?.(true, errorMessage);
    }
  }, [clearError, onStatusChange]);

  /**
   * 切换系统托盘状态
   * 后端会自动处理状态检查、防重复创建、状态同步等所有逻辑
   */
  const toggle = useCallback(async (): Promise<void> => {
    if (isOperatingRef.current || isLoading) {
      return;
    }

    setIsLoading(true);
    clearError();
    isOperatingRef.current = true;

    try {
      // 调用后端统一切换接口（所有逻辑由后端处理）
      const result: SystemTrayStatus = await SystemTrayService.toggleSystemTray();

      // 更新UI状态
      setEnabled(result.enabled);
      onStatusChange?.(result.enabled, result.message);

      console.log('系统托盘切换结果:', result);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : '未知错误';
      setError(`切换系统托盘失败: ${errorMessage}`);
      console.error('系统托盘切换失败:', error);
    } finally {
      setIsLoading(false);
      isOperatingRef.current = false;
    }
  }, [isLoading, clearError, onStatusChange]);

  /**
   * 重新初始化
   */
  const reinitialize = useCallback(async (): Promise<void> => {
    setIsInitialized(false);
    await initialize();
  }, [initialize]);

  // 自动初始化
  useEffect(() => {
    if (autoInit && !isInitialized) {
      initialize();
    }
  }, [autoInit, isInitialized, initialize]);

  return {
    enabled,
    isLoading,
    isInitialized,
    error,
    toggle,
    reinitialize
  };
}