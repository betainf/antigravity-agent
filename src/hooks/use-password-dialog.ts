import { useState, useCallback } from 'react';

interface PasswordDialogConfig {
  title: string;
  description?: string;
  requireConfirmation?: boolean;
  onSubmit: (password: string) => void;
  validatePassword?: (password: string) => { isValid: boolean; message?: string };
}

interface PasswordDialogState {
  isOpen: boolean;
  title: string;
  description: string;
  requireConfirmation: boolean;
  onSubmit: (password: string) => void;
  validatePassword?: (password: string) => { isValid: boolean; message?: string };
}

export const usePasswordDialog = (
  onStatusUpdate: (message: string, isError?: boolean) => void
) => {
  const [passwordDialog, setPasswordDialog] = useState<PasswordDialogState>({
    isOpen: false,
    title: '',
    description: '',
    requireConfirmation: false,
    onSubmit: () => {}
  });

  // 打开密码对话框
  const showPasswordDialog = useCallback((config: PasswordDialogConfig) => {
    setPasswordDialog({
      isOpen: true,
      title: config.title,
      description: config.description || '',
      requireConfirmation: config.requireConfirmation || false,
      onSubmit: config.onSubmit,
      validatePassword: config.validatePassword
    });
  }, []);

  // 关闭密码对话框
  const closePasswordDialog = useCallback(() => {
    setPasswordDialog(prev => ({ ...prev, isOpen: false }));
  }, []);

  // 处理密码对话框取消
  const handlePasswordDialogCancel = useCallback(() => {
    closePasswordDialog();
    onStatusUpdate('操作已取消', true);
  }, [closePasswordDialog, onStatusUpdate]);

  return {
    passwordDialog,
    showPasswordDialog,
    closePasswordDialog,
    handlePasswordDialogCancel
  };
};