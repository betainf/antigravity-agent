import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

const BackupSection = ({ onBackupSuccess, showStatus }) => {
  const [profileName, setProfileName] = useState('');
  const [sourcePath, setSourcePath] = useState('');
  const [isBackupLoading, setIsBackupLoading] = useState(false);

  const browseDirectory = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
      });
      if (selected) {
        setSourcePath(selected);
      }
    } catch (error) {
      showStatus(`选择目录失败: ${error}`, true);
    }
  };

  const createBackup = async () => {
    if (!profileName.trim()) {
      showStatus('请输入配置文件名称', true);
      return;
    }

    if (!sourcePath.trim()) {
      showStatus('请选择源目录', true);
      return;
    }

    try {
      setIsBackupLoading(true);
      const result = await invoke('backup_profile', {
        name: profileName.trim(),
        sourcePath: sourcePath.trim()
      });

      showStatus(result);
      setProfileName('');
      setSourcePath('');
      onBackupSuccess();
    } catch (error) {
      showStatus(`备份失败: ${error}`, true);
    } finally {
      setIsBackupLoading(false);
    }
  };

  return (
    <section className="card section-full">
      <h2>创建备份</h2>
      <div className="form-group">
        <label htmlFor="profile-name" className="form-label">配置文件名称:</label>
        <input
          type="text"
          id="profile-name"
          value={profileName}
          onChange={(e) => setProfileName(e.target.value)}
          placeholder="例如: work-account"
          className="form-input"
        />
      </div>

      <div className="form-group">
        <label htmlFor="source-path" className="form-label">源目录路径:</label>
        <div className="input-group">
          <input
            type="text"
            id="source-path"
            value={sourcePath}
            onChange={(e) => setSourcePath(e.target.value)}
            placeholder="选择要备份的目录"
            className="form-input"
          />
          <button onClick={browseDirectory} className="btn btn-secondary">浏览</button>
        </div>
      </div>

      <button
        className="btn btn-primary"
        onClick={createBackup}
        disabled={isBackupLoading}
      >
        {isBackupLoading ? '备份中...' : '创建备份'}
      </button>
    </section>
  );
};

export default BackupSection;