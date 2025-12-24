import {
    VSCodePanels,
    VSCodePanelTab,
    VSCodePanelView,
    VSCodeCheckbox
} from '@vscode/webview-ui-toolkit/react';
import React, { useState } from 'react';
import { AccountsTab } from './components/AccountsTab';
import './App.css';

const App: React.FC = () => {
    const [autoAccept, setAutoAccept] = useState(false);

    // Acquire VS Code API
    const vscode = React.useMemo(() => {
        try {
            return (window as any).acquireVsCodeApi();
        } catch {
            return null;
        }
    }, []);

    const toggleAutoAccept = () => {
        const newState = !autoAccept;
        setAutoAccept(newState);
        if (vscode) {
            vscode.postMessage({
                command: 'setAutoAccept',
                enabled: newState
            });
        }
    };

    return (
        <div className="app-container" style={{ position: 'relative' }}>
            <div style={{ position: 'absolute', top: '8px', right: '16px', zIndex: 10 }}>
                <VSCodeCheckbox
                    checked={autoAccept}
                    onChange={toggleAutoAccept}
                    style={{ fontSize: '13px' }}
                >
                    🚁 自动驾驶
                </VSCodeCheckbox>
            </div>
            <VSCodePanels className="panels-full-width">
                <VSCodePanelTab id="tab-accounts">
                    账户列表
                </VSCodePanelTab>
                <VSCodePanelView id="view-accounts" className="panel-view">
                    <AccountsTab />
                </VSCodePanelView>
            </VSCodePanels>
        </div>
    );
};

export default App;
