import * as vscode from 'vscode';
import { AntigravityPanel } from './managers/antigravity-panel';
import { Logger } from './utils/logger';
import { StatusBarManager } from './managers/status-bar-manager';
import { AutoAcceptManager } from './managers/auto-accept-manager';
import { initializeWebSocket } from './services/websocket-client';


// export let statusBarItem: vscode.StatusBarItem; // Removed global export as now passed into manager directly

/**
 * Activates the Antigravity VS Code Extension.
 * Defines commands, initializes the status bar, and sets up analytics/browser interception.
 */
export async function activate(context: vscode.ExtensionContext) {
    Logger.initialize(context);
    Logger.log("Antigravity Extension Activated");

    // Register the command to open the panel
    context.subscriptions.push(
        vscode.commands.registerCommand('antigravity.agent.open_dialog', () => {
            AntigravityPanel.createOrShow(context);
        })
    );
    // Alias for backward compatibility if needed, or just standard command
    context.subscriptions.push(
        vscode.commands.registerCommand('antigravity.openDialog', () => {
            AntigravityPanel.createOrShow(context);
        })
    );
    // User requested dashboard command
    context.subscriptions.push(
        vscode.commands.registerCommand('antigravity.agent.open_dashboard', () => {
            AntigravityPanel.createOrShow(context);
        })
    );

    // Initialize Status Bar Items

    // 1. Metrics Item (Left): Shows Logo + Model/Quota
    // Priority 10000
    const metricsItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 10000);
    metricsItem.text = "$(antigravity-logo) Loading...";
    metricsItem.command = "antigravity.agent.open_dialog"; // or open_dashboard
    metricsItem.show();

    // 2. User Item (Right): Shows Account Info
    // Priority 9999 (To the right of Metrics)
    const userItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 9999);
    userItem.text = "$(account) Loading...";
    userItem.command = "antigravity.agent.open_dialog";
    userItem.show();

    // Initialize Managers
    StatusBarManager.initialize(metricsItem, userItem, context);
    StatusBarManager.registerAnalyticsInterceptor(context);

    AutoAcceptManager.initialize(context);

    // Initialize WebSocket for bidirectional communication with Tauri backend
    initializeWebSocket(context);

    context.subscriptions.push(metricsItem);
    context.subscriptions.push(userItem);
}

export function updateStatusBar(text: string) {
    // Legacy support or remove if unused
}

export function deactivate() { }
