import React, { useState, useMemo } from 'react';
import PasswordDialog from './PasswordDialog';
import { TooltipProvider } from './ui/tooltip';
import ToolbarTitle from './ui/toolbar-title';
import ToolbarActions from './toolbar-actions';
import { usePasswordDialog } from '../hooks/use-password-dialog';

interface ToolbarProps {
  onRefresh: () => void;
  isRefreshing?: boolean;
  showStatus: (message: string, isError?: boolean) => void;
}

interface LoadingState {
  isProcessLoading: boolean;
  isImporting: boolean;
  isExporting: boolean;
}

const Toolbar: React.FC<ToolbarProps> = ({ onRefresh, isRefreshing = false, showStatus }) => {
  const [loadingState, setLoadingState] = useState<LoadingState>({
    isProcessLoading: false,
    isImporting: false,
    isExporting: false
  });

  // 使用密码对话框 Hook
  const {
    passwordDialog,
    showPasswordDialog,
    closePasswordDialog,
    handlePasswordDialogCancel
  } = usePasswordDialog(showStatus);

  // 计算全局加载状态
  const isAnyLoading = useMemo(() => {
    return loadingState.isProcessLoading ||
           loadingState.isImporting ||
           loadingState.isExporting ||
           isRefreshing;
  }, [loadingState, isRefreshing]);

  return (
    <TooltipProvider delayDuration={300}>
      <div className="toolbar bg-gradient-to-r from-slate-50 to-slate-100 dark:from-slate-800 dark:to-slate-900 border-b border-gray-200 dark:border-gray-700 sticky top-0 z-50 backdrop-blur-sm shadow-sm">
        <div className="toolbar-content max-w-7xl mx-auto px-4 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4">
              <ToolbarTitle />
            </div>

            <ToolbarActions
              loadingState={loadingState}
              isRefreshing={isRefreshing}
              isAnyLoading={isAnyLoading}
              onRefresh={onRefresh}
              showStatus={showStatus}
              setLoadingState={setLoadingState}
              showPasswordDialog={showPasswordDialog}
              closePasswordDialog={closePasswordDialog}
            />
          </div>
        </div>
      </div>

      <PasswordDialog
        isOpen={passwordDialog.isOpen}
        onOpenChange={(open) => !open && handlePasswordDialogCancel()}
        title={passwordDialog.title}
        description={passwordDialog.description}
        requireConfirmation={passwordDialog.requireConfirmation}
        validatePassword={passwordDialog.validatePassword}
        onSubmit={passwordDialog.onSubmit}
        onCancel={handlePasswordDialogCancel}
      />
    </TooltipProvider>
  );
};

export default Toolbar;