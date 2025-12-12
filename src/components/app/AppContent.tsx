import React, {useEffect, useState} from "react";
import BusinessUserDetail from "@/components/business/AccountDetailModal.tsx";
import {useAntigravityAccount, useCurrentAntigravityAccount} from "@/modules/use-antigravity-account.ts";
import {useAccountAdditionData, UserTier} from "@/modules/use-account-addition-data.ts";
import {useTrayMenu} from "@/hooks/use-tray-menu.ts";

import BusinessConfirmDialog from "@/components/business/ConfirmDialog.tsx";
import toast from 'react-hot-toast';
import {maskEmail} from "@/lib/string-masking.ts";
import {useAppGlobalLoader} from "@/modules/use-app-global-loader.ts";
import {AccountSessionList, AccountSessionListAccountItem} from "@/components/business/AccountSessionList.tsx";

export function AppContent() {
  const [isUserDetailOpen, setIsUserDetailOpen] = useState(false);
  const [selectedUser, setSelectedUser] = useState<AccountSessionListAccountItem | null>(null);
  const antigravityAccount = useAntigravityAccount();
  const availableModels = useAccountAdditionData();
  const currentAntigravityAccount = useCurrentAntigravityAccount();
  const appGlobalLoader = useAppGlobalLoader();

  // 初始化托盘菜单更新
  useTrayMenu();

  // 组件挂载时获取用户列表
  useEffect(() => {
    const loadUsers = async () => {
      try {
        await antigravityAccount.getAccounts();
      } catch (error) {
        toast.error(`获取用户列表失败: ${error}`);
      } finally {
      }
    };

    loadUsers();
  }, []);

  useEffect(() => {
    antigravityAccount.accounts.forEach(user => {
      availableModels.fetchData(user)
    })
  }, [antigravityAccount.accounts]);

  const [isClearDialogOpen, setIsClearDialogOpen] = useState(false);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [accountToDelete, setAccountToDelete] = useState<string | null>(null);

  // 用户详情处理
  const handleUserClick = (account: AccountSessionListAccountItem) => {
    setSelectedUser(account);
    setIsUserDetailOpen(true);
  };

  const handleUserDetailClose = () => {
    setIsUserDetailOpen(false);
    setSelectedUser(null);
  };

  const handleDeleteBackup = (user: AccountSessionListAccountItem) => {
    setAccountToDelete(user.email);
    setDeleteDialogOpen(true);
  };

  const confirmDeleteAccount = async () => {
    if (!accountToDelete) return;

    await antigravityAccount.delete(accountToDelete);
    toast.success(`账户 "${accountToDelete}" 删除成功`);
    setDeleteDialogOpen(false);
    setAccountToDelete(null);
  };

  const handleSwitchAccount = async (user: AccountSessionListAccountItem) => {
    try {
      appGlobalLoader.open({label: `正在切换到用户: ${maskEmail(user.email)}...`});
      await antigravityAccount.switchToAccount(user.email);
    } finally {
      appGlobalLoader.close();
    }
  };

  const handleClearAllBackups = () => {
    if (antigravityAccount.accounts.length === 0) {
      toast.error('当前没有用户备份可清空');
      return;
    }
    setIsClearDialogOpen(true);
  };

  const confirmClearAllBackups = async () => {
    try {
      await antigravityAccount.clearAllAccounts();
      toast.success('清空所有备份成功');
      setIsClearDialogOpen(false);
    } catch (error) {
      toast.error(`清空备份失败: ${error}`);
    }
  };

  const accounts: AccountSessionListAccountItem[] = antigravityAccount.accounts.map((account) => {
    const accountAdditionDatum = availableModels.data[account.context.email]

    return {
      geminiQuota: accountAdditionDatum?.geminiQuote ?? -1,
      claudeQuota: accountAdditionDatum?.geminiQuote ?? -1,
      email: account.context.email,
      nickName: account.context.plan_name,
      userAvatar: accountAdditionDatum?.userAvatar ?? "",
      apiKey: account.auth.access_token,
      tier: account.context.plan.slug as UserTier,
    }
  })

  return (
    <>
      <section className="flex flex-col relative flex-1">
        <AccountSessionList
          accounts={accounts}
          onSwitch={handleSwitchAccount}
          onDelete={handleDeleteBackup}
          onSelect={handleUserClick}
          currentUserEmail={currentAntigravityAccount?.context.email}
        />
      </section>

      {/* 清空所有备份确认对话框 */}
      <BusinessConfirmDialog
        isOpen={isClearDialogOpen}
        onOpenChange={setIsClearDialogOpen}
        title="确认清空所有备份"
        description={`此操作将永久删除所有 ${antigravityAccount.accounts.length} 个账户，且无法恢复。请确认您要继续此操作吗？`}
        onConfirm={confirmClearAllBackups}
        onCancel={() => setIsClearDialogOpen(false)}
        variant="destructive"
        isLoading={false}
        confirmText="确认删除"
      />

      {/* 单个删除确认对话框 */}
      <BusinessConfirmDialog
        isOpen={deleteDialogOpen}
        onOpenChange={setDeleteDialogOpen}
        title="确认删除账户"
        description={`确定要删除账户 "${accountToDelete}" 吗？此操作无法撤销。`}
        onConfirm={confirmDeleteAccount}
        onCancel={() => setDeleteDialogOpen(false)}
        variant="destructive"
        isLoading={false}
        confirmText="确认删除"
      />

      <BusinessUserDetail
        isOpen={isUserDetailOpen}
        onOpenChange={handleUserDetailClose}
        account={selectedUser}
      />
    </>
  );
}
