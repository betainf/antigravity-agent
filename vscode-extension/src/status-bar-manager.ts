import * as vscode from 'vscode';
import { Logger } from './logger';
import { AccountMetrics } from '@/commands/types/account.types';

interface CurrentAccount {
    context: {
        email: string;
        plan?: {
            slug: string;
        };
    };
}

export class StatusBarManager {
    private static interval: NodeJS.Timeout | undefined;
    private static statusBarItem: vscode.StatusBarItem;
    private static readonly API_BASE = 'http://127.0.0.1:18888/api';

    private static currentMetrics: AccountMetrics | null = null;
    private static lastModelName: string = 'Gemini 3 Pro (High)'; // Default Model Name

    public static initialize(item: vscode.StatusBarItem, context: vscode.ExtensionContext) {
        this.statusBarItem = item;
        this.startPolling();
        context.subscriptions.push({ dispose: () => this.stopPolling() });
    }

    private static startPolling() {
        // Initial fetch
        this.update();
        // Poll every 30 seconds
        this.interval = setInterval(() => this.update(), 30 * 1000);
    }

    private static stopPolling() {
        if (this.interval) {
            clearInterval(this.interval);
            this.interval = undefined;
        }
    }

    public static async updateWithModelUsage(modelName: string) {
        this.lastModelName = modelName;

        // Trigger an update to refresh UI with new category
        // If we have cached metrics, we can update immediately
        if (this.currentMetrics) {
            this.render(this.currentMetrics);
        } else {
            await this.update();
        }
    }

    public static async update() {
        try {
            // 1. Get Current Account
            const accRes = await fetch(`${this.API_BASE}/get_current_antigravity_account_info`);
            if (!accRes.ok) return;
            const currentAccount = await accRes.json() as CurrentAccount | null;

            if (!currentAccount || !currentAccount.context?.email) {
                this.statusBarItem.tooltip = "No active Antigravity account";
                this.statusBarItem.text = "$(account) Antigravity: None";
                return;
            }

            const email = currentAccount.context.email;

            // 2. Get Metrics
            const metricRes = await fetch(`${this.API_BASE}/get_account_metrics`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ email })
            });

            if (!metricRes.ok) {
                this.statusBarItem.tooltip = `Current: ${email}\n(Failed to load metrics)`;
                return;
            }

            this.currentMetrics = await metricRes.json() as AccountMetrics;
            this.render(this.currentMetrics, currentAccount);

        } catch (error) {
            Logger.log(`Failed to update status bar: ${error}`);
        }
    }

    private static render(metrics: AccountMetrics, currentAccount?: CurrentAccount) {
        if (!metrics) return;

        // 3. Build Tooltip
        const md = new vscode.MarkdownString();
        md.isTrusted = true;

        if (currentAccount) {
            const email = currentAccount.context.email;
            const plan = currentAccount.context.plan?.slug || 'UNKNOWN';
            md.appendMarkdown(`**User**: ${email}\n\n`);
            md.appendMarkdown(`**Plan**: ${plan}\n\n`);
            md.appendMarkdown(`---\n\n`);
        }

        if (metrics.quotas && metrics.quotas.length > 0) {
            md.appendMarkdown(`| Model | Usage | Reset |\n`);
            md.appendMarkdown(`|---|---|---|\n`);

            metrics.quotas.forEach(q => {
                const usage = Math.round(q.percentage * 100);
                const isWarning = q.percentage < 0.2;
                const usageStr = isWarning ? `**${usage}%** 🔴` : `${usage}%`;

                md.appendMarkdown(`| ${q.model_name} | ${usageStr} | ${q.reset_text || '-'} |\n`);
            });
        } else {
            md.appendMarkdown(`*No quota info available*`);
        }

        this.statusBarItem.tooltip = md;

        // 4. Update Status Bar Text
        // Resolve category for quota lookup
        const { getQuotaCategory } = require('./constants/model-mappings');
        const category = getQuotaCategory(this.lastModelName);

        // Find quota for that category
        const targetQuota = metrics.quotas.find(q => q.model_name.includes(category));

        if (targetQuota) {
            const percentage = Math.round(targetQuota.percentage * 100);
            // Display: $(coffee) [Model Name]: [Quota]%
            this.statusBarItem.text = `$(coffee) ${this.lastModelName}: ${percentage}%`;
        } else {
            // Fallback
            this.statusBarItem.text = `$(coffee) ${this.lastModelName}`;
        }
    }
}
