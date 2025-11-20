import React, { useCallback, useState, useEffect } from 'react';
import {
  Plus,
  Download,
  Upload,
  RefreshCw
} from 'lucide-react';
import ToolbarButton from './ui/toolbar-button';
import ConfirmDialog from './ConfirmDialog';
import { ConfigManager } from '../services/config-manager';
import { AntigravityService } from '../services/antigravity-service';
import { EncryptionService } from '../utils/encryption';

interface LoadingState {
  isProcessLoading: boolean;
  isImporting: boolean;
  isExporting: boolean;
}

interface ToolbarActionsProps {
  loadingState: LoadingState;
  isRefreshing?: boolean;
  isAnyLoading: boolean;
  onRefresh: () => void;
  showStatus: (message: string, isError?: boolean) => void;
  setLoadingState: React.Dispatch<React.SetStateAction<LoadingState>>;
  showPasswordDialog: (config: {
    title: string;
    description?: string;
    requireConfirmation?: boolean;
    onSubmit: (password: string) => void;
    validatePassword?: (password: string) => { isValid: boolean; message?: string };
  }) => void;
  closePasswordDialog: () => void;
}

const ToolbarActions: React.FC<ToolbarActionsProps> = ({
  loadingState,
  isRefreshing = false,
  isAnyLoading,
  onRefresh,
  showStatus,
  setLoadingState,
  showPasswordDialog,
  closePasswordDialog
}) => {
  // 状态：是否有用户数据可以导出
  const [hasUserData, setHasUserData] = useState<boolean>(false);
  const [isCheckingData, setIsCheckingData] = useState<boolean>(false);

  // 确认对话框状态
  const [confirmDialog, setConfirmDialog] = useState<{
    isOpen: boolean;
    title: string;
    description: string;
    onConfirm: () => void;
  }>({
    isOpen: false,
    title: '',
    description: '',
    onConfirm: () => { }
  });

  // 检查是否有用户数据可以导出
  const checkUserData = useCallback(async () => {
    try {
      setIsCheckingData(true);
      const backupList = await AntigravityService.getBackupList();
      setHasUserData(backupList.length > 0);
    } catch (error) {
      console.error('检查用户数据失败:', error);
      setHasUserData(false);
    } finally {
      setIsCheckingData(false);
    }
  }, []);

  // 组件挂载时检查用户数据，当刷新时重新检查
  useEffect(() => {
    checkUserData();
  }, [checkUserData]);

  // 当刷新操作完成后，重新检查用户数据
  useEffect(() => {
    if (!isRefreshing) {
      const timer = setTimeout(() => {
        checkUserData();
      }, 500); // 延迟500ms确保刷新完成
      return () => clearTimeout(timer);
    }
  }, [isRefreshing, checkUserData]);
  // 备份并重启Antigravity (登录新账户)
  const backupAndRestartAntigravity = useCallback(async () => {
    console.log('🔘 用户点击登录新账户按钮，显示确认对话框');

    setConfirmDialog({
      isOpen: true,
      title: '登录新账户',
      description: `确定要关闭 Antigravity 并登录新账户吗？

此操作将会：
1. 关闭所有 Antigravity 进程
2. 自动备份当前账户信息
3. 清除 Antigravity 用户信息
4. 重新启动 Antigravity

登录新账户后点击 “刷新” 即可保存新账户 
注意：请确保已保存所有重要工作`,
      onConfirm: async () => {
        console.log('✅ 用户确认登录新账户操作');
        try {
          setLoadingState(prev => ({ ...prev, isProcessLoading: true }));

          console.log('📤 发送状态更新: 正在备份当前用户并注销...');
          showStatus('正在备份当前用户并注销...');

          console.log('🔄 调用 AntigravityService.backupAndRestartAntigravity');
          await AntigravityService.backupAndRestartAntigravity(showStatus);

          console.log('✅ 备份并重启操作完成，准备刷新界面');
          // 延迟刷新以确保操作完成
          setTimeout(() => {
            console.log('🔄 执行界面刷新');
            onRefresh();
          }, 1000);

        } catch (error) {
          console.error('❌ 登录新账户操作失败:', error);
          const errorMessage = error instanceof Error ? error.message : String(error);
          showStatus(errorMessage, true);
        } finally {
          setLoadingState(prev => ({ ...prev, isProcessLoading: false }));
          console.log('🔧 操作流程结束，重置加载状态');
        }
      }
    });
  }, [showStatus, onRefresh, setLoadingState]);

  // 导入配置文件
  const importConfig = useCallback(async () => {
    try {
      const result = await ConfigManager.importConfig(showStatus);

      if (!result.success) {
        showStatus(result.message, true);
        return;
      }

      // 使用密码对话框获取密码
      showPasswordDialog({
        title: '导入配置文件',
        description: '请输入配置文件的解密密码',
        requireConfirmation: false,
        onSubmit: async (password) => {
          try {
            closePasswordDialog();
            setLoadingState(prev => ({ ...prev, isImporting: true }));
            showStatus('正在解密配置文件...');

            const configData = await ConfigManager.decryptConfig(result.encryptedData!, password);

            showStatus(`配置文件导入成功 (版本: ${configData.version})`);
            console.log('导入的配置:', configData);

            // 延迟刷新以确保数据完整性
            setTimeout(() => {
              onRefresh();
            }, 500);

          } catch (error) {
            const errorMessage = error instanceof Error ? error.message : String(error);
            showStatus(`导入配置文件失败: ${errorMessage}`, true);
          } finally {
            setLoadingState(prev => ({ ...prev, isImporting: false }));
          }
        }
      });

    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      showStatus(`选择文件失败: ${errorMessage}`, true);
    }
  }, [showStatus, onRefresh, showPasswordDialog, closePasswordDialog, setLoadingState]);

  // 导出配置文件
  const exportConfig = useCallback(async () => {
    // 使用密码对话框获取密码
    showPasswordDialog({
      title: '导出配置文件',
      description: '请设置导出密码，用于保护您的配置文件',
      requireConfirmation: true,
      validatePassword: EncryptionService.validatePassword,
      onSubmit: async (password) => {
        try {
          closePasswordDialog();
          setLoadingState(prev => ({ ...prev, isExporting: true }));

          await ConfigManager.exportConfig(password, showStatus);

        } catch (error) {
          // ConfigManager 已经处理了错误显示
        } finally {
          setLoadingState(prev => ({ ...prev, isExporting: false }));
        }
      }
    });
  }, [showStatus, showPasswordDialog, closePasswordDialog, setLoadingState]);

  return (
    <>
      <div className="flex items-center gap-2">
        <ToolbarButton
          onClick={backupAndRestartAntigravity}
          isLoading={loadingState.isProcessLoading}
          loadingText="处理中..."
          tooltip="关闭 Antigravity，备份当前用户，并清除 Antigravity 用户信息"
          variant="primary"
          className="shadow-md hover:shadow-xl"
          isAnyLoading={isAnyLoading}
        >
          <span className="flex items-center gap-2">
            <Plus className="h-4 w-4" />
            登录新账户
          </span>
        </ToolbarButton>

        <ToolbarButton
          onClick={importConfig}
          isLoading={loadingState.isImporting}
          loadingText="导入中..."
          tooltip="导入加密的配置文件"
          isAnyLoading={isAnyLoading}
        >
          <span className="flex items-center gap-2">
            <Upload className="h-4 w-4" />
            导入
          </span>
        </ToolbarButton>

        <ToolbarButton
          onClick={exportConfig}
          isLoading={loadingState.isExporting || isCheckingData}
          loadingText={isCheckingData ? "检查中..." : "导出中..."}
          tooltip={hasUserData ? "导出为加密配置文件" : "没有用户信息可以导出"}
          disabled={!hasUserData || isAnyLoading}
          variant={hasUserData ? "secondary" : "danger"}
          isAnyLoading={isAnyLoading}
        >
          <span className="flex items-center gap-2">
            <Download className="h-4 w-4" />
            导出
          </span>
        </ToolbarButton>

        <ToolbarButton
          onClick={onRefresh}
          isLoading={isRefreshing}
          loadingText="刷新中..."
          tooltip="刷新所有数据"
          isAnyLoading={isAnyLoading}
        >
          <span className="flex items-center gap-2">
            <RefreshCw className="h-4 w-4" />
            刷新
          </span>
        </ToolbarButton>
      </div>

      <ConfirmDialog
        isOpen={confirmDialog.isOpen}
        onOpenChange={(open) => {
          // 只在对话框被关闭时更新状态，不打印日志
          // 因为关闭可能是由确认或取消触发的
          if (!open) {
            setConfirmDialog(prev => ({ ...prev, isOpen: false }));
          }
        }}
        title={confirmDialog.title}
        description={confirmDialog.description}
        onConfirm={confirmDialog.onConfirm}
        onCancel={() => {
          console.log('❌ 用户取消了登录新账户操作');
          setConfirmDialog(prev => ({ ...prev, isOpen: false }));
        }}
      />
    </>
  );
};

export default ToolbarActions;