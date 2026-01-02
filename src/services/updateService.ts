import { check, Update, DownloadEvent } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { logger } from '../lib/logger.ts';
import i18n from '@/i18n';

export interface UpdateInfo {
    version: string;
    currentVersion: string;
    date: string;
    body: string;
}

export type UpdateState =
    | 'no-update'
    | 'update-available'
    | 'downloading'
    | 'ready-to-install'
    | 'error';

export interface DownloadProgress {
    downloaded: number;
    total: number;
    percentage: number;
}

class UpdateService {
    private pendingUpdate: Update | null = null;

    /**
     * 检查是否有可用更新
     */
    async checkForUpdates(): Promise<UpdateInfo | null> {
        try {
            const update = await check();

            if (update === null) {
                logger.info('没有可用更新', {
                    module: 'UpdateService',
                    action: 'no_update_available'
                });
                return null;
            }

            this.pendingUpdate = update;

            return {
                version: update.version,
                currentVersion: update.currentVersion,
                date: update.date,
                body: update.body || '',
            };
        } catch (error) {
            logger.error('检查更新失败', {
                module: 'UpdateService',
                action: 'check_failed',
                error: error
            });
            throw new Error(i18n.t('update:error.checkFailed'));
        }
    }

    /**
     * 下载更新包
     * @param onProgress 进度回调
     */
    async downloadUpdate(
        onProgress: (progress: DownloadProgress) => void
    ): Promise<void> {
        if (!this.pendingUpdate) {
            throw new Error(i18n.t('update:error.noUpdateToDownload'));
        }

        let downloaded = 0;
        let total = 0;

        try {
            await this.pendingUpdate.download((event: DownloadEvent) => {
                switch (event.event) {
                    case 'Started':
                        total = event.data.contentLength || 0;
                        logger.info('开始下载', {
                            module: 'UpdateService',
                            action: 'download_started',
                            totalBytes: total
                        });
                        onProgress({ downloaded: 0, total, percentage: 0 });
                        break;

                    case 'Progress':
                        downloaded += event.data.chunkLength;
                        const percentage = total > 0 ? Math.round((downloaded / total) * 100) : 0;
                        logger.debug('下载进度', {
                            module: 'UpdateService',
                            action: 'download_progress',
                            downloaded,
                            total,
                            percentage
                        });
                        onProgress({ downloaded, total, percentage });
                        break;

                    case 'Finished':
                        logger.info('下载完成', {
                            module: 'UpdateService',
                            action: 'download_completed',
                            totalBytes: total
                        });
                        onProgress({ downloaded: total, total, percentage: 100 });
                        break;
                }
            });
        } catch (error) {
            logger.error('下载更新失败', {
                module: 'UpdateService',
                action: 'download_failed',
                error: error
            });
            throw new Error(i18n.t('update:error.downloadFailed'));
        }
    }

    /**
     * 安装更新并重启应用
     */
    async installAndRelaunch(): Promise<void> {
        if (!this.pendingUpdate) {
            throw new Error(i18n.t('update:error.noUpdateToInstall'));
        }

        try {
            logger.info('开始安装更新', {
                module: 'UpdateService',
                action: 'install_started'
            });
            await this.pendingUpdate.install();

            logger.info('安装完成，准备重启', {
                module: 'UpdateService',
                action: 'install_completed'
            });
            // 等待一小段时间确保安装完成
            await new Promise(resolve => setTimeout(resolve, 500));

            // 重启应用
            await relaunch();
        } catch (error) {
            logger.error('安装更新失败', {
                module: 'UpdateService',
                action: 'install_failed',
                error: error
            });
            throw new Error(i18n.t('update:error.installFailed'));
        }
    }

    /**
     * 清除待处理的更新
     */
    clearPendingUpdate(): void {
        this.pendingUpdate = null;
    }
}

export const updateService = new UpdateService();
