import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type { ListBackupsResult } from './types/tauri';
import ManageSection from './components/ManageSection';
import StatusNotification from './components/StatusNotification';
import Toolbar from './components/Toolbar';

interface Status {
  message: string;
  isError: boolean;
}

function App() {
  const [backups, setBackups] = useState<string[]>([]);
  const [status, setStatus] = useState<Status>({ message: '', isError: false });
  const [isRefreshing, setIsRefreshing] = useState(false);

  const showStatus = (message: string, isError: boolean = false): void => {
    setStatus({ message, isError });
    setTimeout(() => {
      setStatus({ message: '', isError: false });
    }, 5000);
  };

  const refreshBackupList = async (skipAutoBackup: boolean = false): Promise<void> => {
    try {
      // 1. 首先获取当前的备份列表
      let existingBackups: string[] = [];
      try {
        existingBackups = await invoke<ListBackupsResult>('list_backups');
      } catch (error) {
        console.log('获取现有备份列表失败:', error);
      }

      // 2. 尝试获取Antigravity当前运行时的用户信息并进行智能备份
      // 如果 skipAutoBackup 为 true，则跳过此步骤
      let autoBackedUp = false;
      if (!skipAutoBackup) {
        try {
          // 注意：智能备份可以在进程运行时进行（只读数据库）
          const currentInfo = await invoke('get_current_antigravity_info');
          console.log('当前Antigravity用户信息:', currentInfo);

          // 检查是否有有效的用户信息（通过API Key或用户状态判断）
          if (currentInfo && ((currentInfo as any).apiKey || (currentInfo as any).userStatusProtoBinaryBase64)) {
            // 从认证信息中提取邮箱
            const userEmail = (currentInfo as any).email;

            console.log('提取的邮箱:', userEmail);

            // 生成基于邮箱和时间的备份文件名
            let backupName: string;
            let isOverwrite = false;

            // 格式: {邮箱}_{时间戳}
            const timestamp = new Date().toISOString().slice(0, 19).replace(/:/g, '-');
            const newBackupName = `${userEmail}_${timestamp}`;

            // 检查是否已存在该用户的任何备份（以邮箱开头）
            const existingUserBackup = existingBackups.find(backup =>
              backup.startsWith(`${userEmail}_`)
            );

            if (existingUserBackup) {
              // 如果存在备份，使用相同的文件名覆盖
              backupName = existingUserBackup;
              isOverwrite = true;
              console.log('发现已存在的备份，将覆盖:', backupName);
            } else {
              // 如果不存在，创建新的备份文件名
              backupName = newBackupName;
              console.log('创建新的备份，文件名:', backupName);
            }

            try {
              const result = await invoke('backup_antigravity_current_account', {
                accountName: backupName
              });
              console.log('智能备份成功:', result);

              if (isOverwrite) {
                showStatus(`已更新用户备份: ${userEmail}`, false);
              } else {
                showStatus(`已自动备份当前用户: ${userEmail}`, false);
              }
              autoBackedUp = true;
            } catch (backupError) {
              console.error('自动备份失败:', backupError);
              showStatus(`自动备份失败: ${backupError}`, true);
            }
          } else {
            console.log('未检测到有效的用户信息');
          }
        } catch (antigravityError) {
          console.log('无法获取Antigravity当前用户信息:', antigravityError);
          // 改为信息提示而非错误提示，因为"未登录"是正常状态
          showStatus('未检测到已登录的用户', false);
        }
      }

      // 3. 获取更新后的备份文件列表，添加延迟确保文件写入完成
      await new Promise(resolve => setTimeout(resolve, 500)); // 等待500ms确保文件写入完成
      const backupList = await invoke<ListBackupsResult>('list_backups');
      setBackups(backupList);

      if (autoBackedUp) {
        showStatus('刷新成功并已更新备份', false);
      } else if (!skipAutoBackup) {
        // 如果没有备份成功，说明当前没有登录用户，这是正常状态
        // showStatus('刷新成功', false);  // 不再显示，避免覆盖上面的"未检测到已登录的用户"提示
      } else {
        showStatus('刷新成功', false);
      }
    } catch (error) {
      showStatus(`获取备份列表失败: ${error}`, true);
    }
  };

  const handleRefresh = async (): Promise<void> => {
    setIsRefreshing(true);
    try {
      // 正确：刷新按钮应该触发智能备份，然后刷新列表
      await refreshBackupList(false);
    } catch (error) {
      showStatus(`刷新失败: ${error}`, true);
    } finally {
      setIsRefreshing(false);
    }
  };

  useEffect(() => {
    // 应用启动时自动加载备份文件列表
    const loadInitialData = async () => {
      try {
        const backupList = await invoke<ListBackupsResult>('list_backups');
        setBackups(backupList);
      } catch (error) {
        console.error('启动时加载备份列表失败:', error);
        // 静默失败，不影响用户体验
      }
    };

    loadInitialData();
  }, []);

  return (
    <>
      <style>{`
        .DialogOverlay {
          position: fixed;
          inset: 0;
          background-color: rgba(0, 0, 0, 0.5);
          z-index: 50;
        }

        .DialogContent {
          position: fixed;
          left: 50%;
          top: 50%;
          transform: translate(-50%, -50%);
          background-color: rgb(17, 24, 39);
          color: white;
          border-radius: 0.5rem;
          box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
          padding: 1.5rem;
          width: 100%;
          max-width: 28rem;
          z-index: 50;
        }

        .DialogTitle {
          font-size: 1.125rem;
          font-weight: 600;
          margin-bottom: 1rem;
        }

        .DialogDescription {
          color: rgb(209, 213, 219);
          margin-bottom: 1rem;
        }

        .Button {
          padding: 0.5rem 1rem;
          border-radius: 0.375rem;
          font-weight: 500;
          transition: all 0.2s;
          cursor: pointer;
          border: none;
        }

        .Button:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }

        .Button--secondary {
          background-color: transparent;
          color: rgb(209, 213, 219);
          border: 1px solid rgb(75, 85, 99);
        }

        .Button--secondary:hover:not(:disabled) {
          background-color: rgb(75, 85, 99);
          color: white;
        }

        .Button--danger {
          background-color: rgb(220, 38, 38);
          color: white;
        }

        .Button--danger:hover:not(:disabled) {
          background-color: rgb(185, 28, 28);
        }
      `}</style>

      <div>
        <Toolbar onRefresh={handleRefresh} isRefreshing={isRefreshing} showStatus={showStatus} />

        <div className="container">
          <ManageSection
            backups={backups}
            showStatus={showStatus}
            onRefresh={refreshBackupList}
          />
        </div>

        <StatusNotification status={status} />
      </div>
    </>
  );
}

export default App;