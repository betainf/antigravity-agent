import {useCallback, useEffect, useState} from 'react';
import {DownloadProgress, UpdateInfo, updateService, UpdateState} from '../services/updateService';
import {logger} from '../lib/logger.ts';

export interface UseUpdateCheckerResult {
    updateState: UpdateState;
    updateInfo: UpdateInfo | null;
    downloadProgress: DownloadProgress | null;
    error: string | null;
    checkForUpdates: () => Promise<void>;
    startDownload: () => Promise<void>;
    installAndRelaunch: () => Promise<void>;
    dismissUpdate: () => void;
}

export function useUpdateChecker(autoCheck: boolean = true): UseUpdateCheckerResult {
    const [updateState, setUpdateState] = useState<UpdateState>('no-update');
    const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
    const [downloadProgress, setDownloadProgress] = useState<DownloadProgress | null>(null);
    const [error, setError] = useState<string | null>(null);

    /**
     * 检查更新
     */
    const checkForUpdates = useCallback(async () => {
        try {
            setError(null);
            logger.info('正在检查更新', {
            module: 'UpdateChecker',
            action: 'check_start'
          });

            const info = await updateService.checkForUpdates();

            if (info) {
                logger.info('发现新版本', {
                module: 'UpdateChecker',
                action: 'update_found',
                version: info.version
              });
                setUpdateInfo(info);
                setUpdateState('update-available');
            } else {
                logger.info('已是最新版本', {
                module: 'UpdateChecker',
                action: 'up_to_date'
              });
                setUpdateState('no-update');
            }
        } catch (err) {
            const errorMsg = err instanceof Error ? err.message : String(err);
            logger.error('检查更新失败', {
            module: 'UpdateChecker',
            action: 'check_failed',
            error: errorMsg
          });
            setError(errorMsg);
            // 不设置错误状态，保持无更新状态，避免显示错误徽章
            setUpdateState('no-update');
        }
    }, []);

    /**
     * 开始下载更新
     */
    const startDownload = useCallback(async () => {
        try {
            setError(null);
            setUpdateState('downloading');
            setDownloadProgress({ downloaded: 0, total: 0, percentage: 0 });

            await updateService.downloadUpdate((progress) => {
                setDownloadProgress(progress);
            });

            setUpdateState('ready-to-install');
        } catch (err) {
            const errorMsg = err instanceof Error ? err.message : String(err);
            logger.error('下载更新失败', {
            module: 'UpdateChecker',
            action: 'download_failed',
            error: errorMsg
          });
            setError(errorMsg);
            // 下载失败，恢复到更新可用状态，让用户可以重试
            setUpdateState('update-available');
        }
    }, []);

    /**
     * 安装更新并重启
     */
    const installAndRelaunch = useCallback(async () => {
        try {
            setError(null);
            await updateService.installAndRelaunch();
            // 如果重启成功，这里的代码不会执行
        } catch (err) {
            const errorMsg = err instanceof Error ? err.message : String(err);
            logger.error('安装更新失败', {
            module: 'UpdateChecker',
            action: 'install_failed',
            error: errorMsg
          });
            setError(errorMsg);
            // 安装失败，恢复到准备安装状态，让用户可以重试
            setUpdateState('ready-to-install');
        }
    }, []);

    /**
     * 忽略此次更新
     */
    const dismissUpdate = useCallback(() => {
        updateService.clearPendingUpdate();
        setUpdateState('no-update');
        setUpdateInfo(null);
        setDownloadProgress(null);
        setError(null);
    }, []);

    /**
     * 自动检查更新（应用启动时）
     */
    useEffect(() => {
        if (autoCheck) {
            // 延迟3秒后检查更新，避免影响应用启动速度
            const timer = setTimeout(() => {
                checkForUpdates();
            }, 3000);

            return () => clearTimeout(timer);
        }
    }, [autoCheck, checkForUpdates]);

    return {
        updateState,
        updateInfo,
        downloadProgress,
        error,
        checkForUpdates,
        startDownload,
        installAndRelaunch,
        dismissUpdate,
    };
}
