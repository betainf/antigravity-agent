// Tauri API 类型定义
export interface BackupProfileParams {
  name: string;
  source_path: string;
}

export interface RestoreProfileParams {
  name: string;
  target_path: string;
}

export interface DeleteBackupParams {
  name: string;
}

export interface ProfileInfo {
  name: string;
  source_path: string;
  backup_path: string;
  created_at: string;
  last_updated: string;
}

// Tauri 命令返回类型
export type BackupProfileResult = string;
export type RestoreProfileResult = string;
export type ListBackupsResult = string[];
export type DeleteBackupResult = string;
export type SwitchToAntigravityAccountResult = string;

// 切换账户参数
export interface SwitchToAntigravityAccountParams {
  account_name: string;
}

// 错误类型
export type TauriError = string;