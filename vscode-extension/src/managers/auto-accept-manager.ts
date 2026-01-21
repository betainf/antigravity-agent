import * as vscode from 'vscode';
import { Logger } from '../utils/logger';
import { AutomationEngine } from '../services/automation-engine';

const STATE_KEY = 'autoPilotEnabled';

/**
 * Manages the "Auto Pilot" mode which automatically accepts suggestions and runs commands.
 * Orchestrates the automation cycle with randomized timing.
 * State is persisted globally across sessions using globalState.
 */
export class AutoAcceptManager {
    private static enabled = false;
    private static timer: NodeJS.Timeout | undefined;
    private static context: vscode.ExtensionContext | undefined;

    /**
     * Initializes the manager and reads configuration state.
     * @param context The extension context.
     */
    public static initialize(context: vscode.ExtensionContext) {
        this.context = context;

        const config = vscode.workspace.getConfiguration('antigravity-agent');
        const autoPilotEnabled = config.get<boolean>('autoPilot', false);

        if (autoPilotEnabled) {
            Logger.log('🚁 Auto Pilot: Enabled via Configuration');
            this.toggle(true);
        }

        // Listen for configuration changes to sync state
        context.subscriptions.push(vscode.workspace.onDidChangeConfiguration(e => {
            if (e.affectsConfiguration('antigravity-agent.autoPilot')) {
                const newState = vscode.workspace.getConfiguration('antigravity-agent').get<boolean>('autoPilot', false);
                Logger.log(`🚁 Auto Pilot: Config changed to ${newState}`);
                this.toggle(newState);
            }
        }));
    }

    /**
     * Returns the current enabled state.
     */
    public static isEnabled(): boolean {
        return this.enabled;
    }

    /**
     * Toggles the Smart Pilot mode (Auto-Accept).
     * @param enabled Whether to enable or disable the pilot.
     */
    public static toggle(enabled: boolean) {
        // Just update internal state, persistence is handled via Settings update in Panel or User Action
        this.enabled = enabled;

        if (this.enabled) {
            Logger.log('🚁 Smart Pilot: Engaged');
            this.scheduleNextRun();
        } else {
            Logger.log('🚁 Smart Pilot: Disengaged');
            this.stop();
        }
    }

    private static stop() {
        if (this.timer) {
            clearTimeout(this.timer);
            this.timer = undefined;
        }
    }

    private static scheduleNextRun() {
        if (!this.enabled) return;

        // Randomized Jitter: 400ms to 900ms
        // Simulates human reaction time to avoid conflicting with rapid updates or looking robotic
        const jitter = Math.floor(Math.random() * 500) + 400;

        this.timer = setTimeout(async () => {
            await this.performPilotActions();
            // Schedule the next one recursively
            this.scheduleNextRun();
        }, jitter);
    }

    private static async performPilotActions() {
        if (!this.enabled) return;

        try {
            await AutomationEngine.runCycle();
        } catch (e) {
            Logger.log(`❌ Automation Cycle Error: ${e}`);
        }
    }
}

