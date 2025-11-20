import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

const RestoreSection = ({ backups, onRestoreSuccess, showStatus }) => {
  const [selectedBackup, setSelectedBackup] = useState('');
  const [targetPath, setTargetPath] = useState('');
  const [isRestoreLoading, setIsRestoreLoading] = useState(false);

  const browseDirectory = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
      });
      if (selected) {
        setTargetPath(selected);
      }
    } catch (error) {
      showStatus(`选择目录失败: ${error}`, true);
    }
  };

  const restoreBackup = async () => {
    if (!selectedBackup) {
      showStatus('请选择要还原的备份', true);
      return;
    }

    if (!targetPath.trim()) {
      showStatus('请选择目标目录', true);
      return;
    }

    if (confirm(`确定要将备份 "${selectedBackup}" 还原到 "${targetPath}" 吗？`)) {
      try {
        setIsRestoreLoading(true);
        const result = await invoke('restore_profile', {
          name: selectedBackup,
          targetPath: targetPath.trim()
        });

        showStatus(result);
        setTargetPath('');
        onRestoreSuccess();
      } catch (error) {
        showStatus(`还原失败: ${error}`, true);
      } finally {
        setIsRestoreLoading(false);
      }
    }
  };

  return (
    <section className="card section-full">
      <h2>还原备份</h2>
      <div className="form-group">
        <label htmlFor="backup-list" className="form-label">选择备份:</label>
        <select
          id="backup-list"
          value={selectedBackup}
          onChange={(e) => setSelectedBackup(e.target.value)}
          className="form-input"
        >
          <option value="">-- 选择备份 --</option>
          {backups.map((backup) => (
            <option key={backup} value={backup}>
              {backup}
            </option>
          ))}
        </select>
      </div>

      <div className="form-group">
        <label htmlFor="target-path" className="form-label">目标目录路径:</label>
        <div className="input-group">
          <input
            type="text"
            id="target-path"
            value={targetPath}
            onChange={(e) => setTargetPath(e.target.value)}
            placeholder="选择还原到的目录"
            className="form-input"
          />
          <button onClick={browseDirectory} className="btn btn-secondary">浏览</button>
        </div>
      </div>

      <button
        className="btn btn-primary"
        onClick={restoreBackup}
        disabled={isRestoreLoading}
      >
        {isRestoreLoading ? '还原中...' : '还原备份'}
      </button>
    </section>
  );
};

export default RestoreSection;