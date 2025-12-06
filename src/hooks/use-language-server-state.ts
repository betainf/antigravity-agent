import {create} from 'zustand';
import {CacheCommands} from '@/commands/CacheCommands';
import {platform} from "@tauri-apps/plugin-os";

// 状态接口
interface LanguageServerState {
  // LanguageServerState 是否已经获取
  initialized: boolean;
}

// 操作接口
interface LanguageServerActions {
  // 初始化 LanguageServer 状态
  initialize: () => Promise<void>;
  // 清除 LanguageServer 状态
  clear: () => Promise<void>;
}

export const useLanguageServerState = create<LanguageServerState & LanguageServerActions>((set) => ({
  initialized: false,
  initialize: async () => {
    if (platform()  !== 'windows') {
      return;
    }

    const result = await CacheCommands.initializeLanguageServerCache();
    if (result.success) {
      set({ initialized: true });
    } else {
      set({ initialized: false });
    }
  },
  clear: async () => {
    await CacheCommands.clearAllCache();
    set({ initialized: false });
  },
}));
