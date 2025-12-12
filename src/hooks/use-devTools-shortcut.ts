import {useEffect} from 'react';
import {logger} from '../lib/logger.ts';

/**
 * 开发者工具快捷键 Hook
 * 监听 Shift+Ctrl+I 快捷键来切换开发者工具
 */
export const useDevToolsShortcut = () => {
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      // 检测 Shift+Ctrl+I 组合键
      if (event.shiftKey && event.ctrlKey && event.key === 'I') {
        event.preventDefault();

        try {
          // 使用 Tauri 内部 API 切换开发者工具
          if ((window as any).__TAURI__) {
            (window as any).__TAURI__.invoke('tauri', {
              __tauriModule: 'Webview',
              message: {
                cmd: 'internalToggleDevtools',
                data: {}
              }
            });
          }
        } catch (error) {
          logger.error('Failed to toggle devtools', {
            module: 'DevToolsShortcut',
            action: 'toggle_failed',
            error: error instanceof Error ? error.message : String(error)
          });
        }
      }
    };

    // 添加全局键盘事件监听器
    window.addEventListener('keydown', handleKeyDown);

    // 清理函数
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, []);
};
