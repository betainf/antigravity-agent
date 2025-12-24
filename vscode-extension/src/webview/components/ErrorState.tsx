import React from 'react';
import { VSCodeButton } from '@vscode/webview-ui-toolkit/react';

interface ErrorStateProps {
    error: string;
    onRetry: () => void;
}

export const ErrorState: React.FC<ErrorStateProps> = ({ error, onRetry }) => {
    return (
        <div className="p-5 border border-vscode-error rounded-md flex flex-col items-center gap-3 bg-[var(--vscode-inputValidation-errorBackground)]">
            <div className="text-3xl">⚠️</div>
            <div className="font-bold">无法连接后端</div>
            <div className="text-center opacity-90 text-sm">
                无法连接到 Antigravity Agent。<br />
                请确认 Rust 后端已启动。
            </div>
            <code className="bg-vscode-quote-bg px-2 py-1 rounded text-sm">
                cargo run
            </code>
            <VSCodeButton onClick={onRetry}>重试连接</VSCodeButton>
            <div className="text-xs opacity-50 mt-2">
                详细错误: {error}
            </div>
        </div>
    );
};
