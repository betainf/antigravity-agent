import * as vscode from 'vscode';

export class Logger {
    private static _outputChannel: vscode.OutputChannel;

    public static initialize(context: vscode.ExtensionContext) {
        this._outputChannel = vscode.window.createOutputChannel('Antigravity Agent');
        context.subscriptions.push(this._outputChannel);
    }

    public static log(message: string, ...args: any[]) {
        const timestamp = new Date().toLocaleTimeString();
        const prefix = `[${timestamp}] ${message}`;

        // 1. Format for Output Channel (Strings only)
        let stringifiedArgs = '';
        if (args.length > 0) {
            stringifiedArgs = ' ' + args.map(arg =>
                typeof arg === 'object' ? JSON.stringify(arg, null, 2) : arg
            ).join(' ');
        }
        this._outputChannel.appendLine(prefix + stringifiedArgs);

        // 2. Format for Debug Console (Keep objects alive for "Intelligent" inspection)
        // We pass the prefix and the raw args so the browser/debugger console can format them interactively
        console.log(prefix, ...args);
    }

    public static show() {
        this._outputChannel.show(true); // Preserve focus
    }
}
