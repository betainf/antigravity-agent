import * as vscode from 'vscode';
import { AntigravityPanel } from './AntigravityPanel';
import { Logger } from './logger';
import { StatusBarManager } from './status-bar-manager';

export let statusBarItem: vscode.StatusBarItem;

export async function activate(context: vscode.ExtensionContext) {
    Logger.initialize(context);
    Logger.log("Antigravity Extension Activated");

    // Register the command to open the panel
    context.subscriptions.push(
        vscode.commands.registerCommand('antigravity.openDialog', () => {
            AntigravityPanel.createOrShow(context);
        })
    );

    // Initialize Status Bar
    // Priority 10000 ensures it's on the far right (or far left depending on layout logic, generally high priority = closer to core items)
    statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 10000);
    statusBarItem.text = "$(coffee) Antigravity Agent";
    statusBarItem.command = "antigravity.openDialog";
    statusBarItem.show();

    // Initialize Manager (Handles Polling & Tooltip)
    StatusBarManager.initialize(statusBarItem, context);

    context.subscriptions.push(statusBarItem);

    // --- 🎯 TARGETED HIJACK: ANALYTICS ---
    // Specifically intercepting 'antigravity.sendAnalyticsAction' to capture model usage for Status Bar
    try {
        const disposable = vscode.commands.registerCommand('antigravity.sendAnalyticsAction', (...args: any[]) => {
            // Check for Chat Message events where model info is present
            if (args.length > 1 && args[0] === 'CASCADE_MESSAGE_SENT') {
                const payload = args[1];
                if (payload && payload.model_name) {
                    const modelName = payload.model_name;
                    // Update Status Bar with Quota for this model
                    StatusBarManager.updateWithModelUsage(modelName);

                    // Optional: Log detected model to output for verification (useful feature feedback)
                    Logger.log(`🤖 Model Detected: ${modelName}`);
                }
            }
        });
        context.subscriptions.push(disposable);
        Logger.log('✅ Analytics Interceptor Ready');
    } catch (e) {
        Logger.log('❌ Failed to register Analytics Interceptor', e);
    }
}

export function updateStatusBar(text: string) {
    if (statusBarItem) {
        statusBarItem.text = text;
        statusBarItem.show();
    }
}

export function deactivate() { }
