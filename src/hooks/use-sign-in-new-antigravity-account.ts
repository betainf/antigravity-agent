import {create} from 'zustand';
import {logger} from '../lib/logger.ts';
import toast from 'react-hot-toast';
import {AccountManageCommands} from "@/commands/AccountManageCommands.ts";

// Store 状态接口
export interface AntigravityProcessState {
  processing: boolean;
}

// Store 操作接口
export interface AntigravityProcessActions {
  run: () => Promise<void>;
}

// 创建 Zustand Store
export const useSignInNewAntigravityAccount = create<AntigravityProcessState & AntigravityProcessActions>()(
  (set, get) => ({
    // 初始状态
    processing: false,

    // 备份并重启 Antigravity（登录新账户）
    run: async () => {
      set({processing: true});

      try {
        await AccountManageCommands.signInNewAntigravityAccount();
      } catch (e) {
        logger.error('登录新账户操作失败', {
          module: 'AntigravityProcessStore',
          error: e instanceof Error ? e.message : String(e)
        });
        toast.error('登录新账户操作失败');
      } finally {
        set({processing: false});
      }
    },
  })
);
