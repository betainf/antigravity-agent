/**
 * # WebSocket 客户端模块
 *
 * 本模块实现了 VSCode 扩展与 Tauri 后端之间的 WebSocket 双向通信。
 *
 * ## 背景
 *
 * 账户切换时，Tauri 后端需要通知 VSCode 扩展重新加载窗口。
 * 传统的 HTTP 轮询无法实现服务端主动推送，因此使用 WebSocket 实现双向通信。
 *
 * ## 架构
 *
 * ```
 * ┌─────────────────────┐         ┌─────────────────────────────┐
 * │  VSCode Extension   │◀──WS──▶│  Tauri App (:18888/ws)      │
 * │                     │         │                             │
 * │  本模块 (Client)     │         │  websocket.rs (Server)      │
 * │  自动连接 + 重连     │         │                             │
 * │                     │         │                             │
 * │  注册方法处理器      │◀───────│  RPC 调用 (reloadWindow)     │
 * └─────────────────────┘         └─────────────────────────────┘
 * ```
 *
 * ## 核心功能
 *
 * - **自动连接**: 扩展激活时自动连接 WebSocket 服务器
 * - **自动重连**: 连接断开后每 5 秒尝试重连
 * - **RPC 处理**: 接收并执行 Rust 发送的 RPC 调用（如 reloadWindow）
 * - **心跳保活**: 响应服务端心跳，防止连接超时
 *
 * ## 使用示例
 *
 * ```typescript
 * import { initializeWebSocket, getWebSocketClient } from './services/websocket-client';
 *
 * // 在 extension.ts 的 activate 函数中初始化
 * initializeWebSocket(context);
 *
 * // 注册自定义方法处理器
 * const client = getWebSocketClient();
 * client.registerHandler('customMethod', (params) => {
 *     console.log('Received:', params);
 * });
 * ```
 *
 * @module websocket-client
 */

import * as vscode from 'vscode';
import { Logger } from '../utils/logger';

// =============================================================================
// 常量配置
// =============================================================================

/**
 * WebSocket 服务器地址
 *
 * 与 Tauri 后端的 Actix 服务器地址一致。
 * 默认运行在 localhost:56789。
 */
const WS_URL = 'ws://127.0.0.1:56789/ws';

/**
 * 重连延迟（毫秒）
 *
 * 连接断开后等待此时间再尝试重连。
 * 设置为 5 秒以避免频繁重连造成的资源浪费。
 */
const RECONNECT_DELAY = 5000;

// =============================================================================
// 消息协议类型定义
// =============================================================================

/**
 * RPC 请求消息
 *
 * 从 Rust 发送到扩展，请求执行指定方法。
 */
interface RpcRequest {
    /** 请求唯一 ID，用于匹配响应 */
    id: string;
    /** 方法名（如 "reloadWindow"） */
    method: string;
    /** 方法参数 */
    params: any;
}

/**
 * RPC 响应消息
 *
 * 从扩展返回到 Rust，包含方法执行结果。
 */
interface RpcResponse {
    /** 对应请求的 ID */
    id: string;
    /** 成功时的返回值 */
    result?: any;
    /** 失败时的错误信息 */
    error?: string;
}

/**
 * WebSocket 消息类型
 *
 * 所有 WebSocket 通信使用此统一格式，通过 `type` 字段区分消息类型。
 *
 * @example
 * // RPC 请求
 * { type: 'rpc_request', id: 'xxx', method: 'reloadWindow', params: {} }
 *
 * // RPC 响应
 * { type: 'rpc_response', id: 'xxx', result: null }
 */
type WsMessage =
    | { type: 'rpc_request' } & RpcRequest
    | { type: 'rpc_response' } & RpcResponse
    | { type: 'event'; name: string; data: any }
    | { type: 'ping' }
    | { type: 'pong' };

/**
 * 方法处理器函数类型
 *
 * RPC 方法处理器可以是同步或异步函数。
 */
type MethodHandler = (params: any) => Promise<any> | any;

// =============================================================================
// WebSocket 客户端类
// =============================================================================

/**
 * WebSocket 客户端管理器
 *
 * 负责管理与 Tauri 后端的 WebSocket 连接，包括：
 * - 建立和维护连接
 * - 自动重连
 * - 处理 RPC 调用
 * - 发送响应
 *
 * ## 生命周期
 *
 * 1. 调用 `connect()` 建立连接
 * 2. 连接成功后，开始接收消息
 * 3. 收到 RPC 请求时，查找并执行对应处理器
 * 4. 连接断开时，自动尝试重连
 * 5. 调用 `disconnect()` 关闭连接并停止重连
 *
 * @example
 * ```typescript
 * const client = new WebSocketClient();
 *
 * // 注册方法处理器
 * client.registerHandler('reloadWindow', () => {
 *     vscode.commands.executeCommand('workbench.action.reloadWindow');
 * });
 *
 * // 连接
 * client.connect();
 * ```
 */
export class WebSocketClient {
    /** WebSocket 实例 */
    private ws: WebSocket | null = null;

    /** 方法处理器映射表 */
    private handlers: Map<string, MethodHandler> = new Map();

    /** 重连定时器 */
    private reconnectTimer: NodeJS.Timeout | null = null;

    /** 是否正在连接中（防止重复连接） */
    private isConnecting: boolean = false;

    /** 是否已销毁（停止重连） */
    private disposed: boolean = false;

    /**
     * 连接到 WebSocket 服务器
     *
     * 如果已连接或正在连接中，则直接返回。
     * 连接失败时会自动安排重连。
     */
    public connect(): void {
        // 防止重复连接
        if (this.disposed || this.isConnecting || this.ws?.readyState === WebSocket.OPEN) {
            return;
        }

        this.isConnecting = true;
        Logger.log('🔌 正在连接 WebSocket...');

        try {
            this.ws = new WebSocket(WS_URL);

            this.ws.onopen = () => {
                this.isConnecting = false;
                Logger.log('✅ WebSocket 连接成功');
                this.clearReconnectTimer();
            };

            this.ws.onclose = (event) => {
                this.isConnecting = false;
                Logger.log(`WebSocket 连接关闭: ${event.code} ${event.reason}`);
                this.scheduleReconnect();
            };

            this.ws.onerror = (error) => {
                this.isConnecting = false;
                Logger.log(`WebSocket 错误: ${error}`);
            };

            this.ws.onmessage = (event) => {
                this.handleMessage(event.data);
            };
        } catch (error) {
            this.isConnecting = false;
            Logger.log(`WebSocket 连接失败: ${error}`);
            this.scheduleReconnect();
        }
    }

    /**
     * 断开连接并停止重连
     *
     * 调用后，客户端将不再尝试重连。
     * 通常在扩展 deactivate 时调用。
     */
    public disconnect(): void {
        this.disposed = true;
        this.clearReconnectTimer();
        if (this.ws) {
            this.ws.close();
            this.ws = null;
        }
        Logger.log('WebSocket 已断开');
    }

    /**
     * 注册 RPC 方法处理器
     *
     * 当 Rust 发送 RPC 请求调用指定方法时，会执行对应的处理器。
     *
     * @param method - 方法名
     * @param handler - 处理函数（可以是同步或异步）
     *
     * @example
     * ```typescript
     * client.registerHandler('reloadWindow', () => {
     *     vscode.commands.executeCommand('workbench.action.reloadWindow');
     * });
     *
     * client.registerHandler('showMessage', async (params) => {
     *     await vscode.window.showInformationMessage(params.message);
     *     return { shown: true };
     * });
     * ```
     */
    public registerHandler(method: string, handler: MethodHandler): void {
        this.handlers.set(method, handler);
        Logger.log(`📝 注册 RPC 方法: ${method}`);
    }

    /**
     * 发送消息到服务器
     *
     * @param message - 要发送的消息
     */
    private send(message: WsMessage): void {
        if (this.ws?.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify(message));
        }
    }

    /**
     * 处理收到的消息
     *
     * 根据消息类型分发到不同的处理逻辑。
     *
     * @param data - 收到的原始消息数据
     */
    private async handleMessage(data: string): Promise<void> {
        try {
            const message: WsMessage = JSON.parse(data);

            switch (message.type) {
                case 'rpc_request':
                    // 处理 RPC 调用
                    await this.handleRpcRequest(message);
                    break;
                case 'ping':
                    // 响应心跳
                    this.send({ type: 'pong' });
                    break;
                case 'event':
                    // 处理事件通知
                    Logger.log(`📨 收到事件: ${message.name}`);
                    // TODO: 可以添加事件处理器机制
                    break;
                default:
                    Logger.log(`未知消息类型: ${(message as any).type}`);
            }
        } catch (error) {
            Logger.log(`解析 WebSocket 消息失败: ${error}`);
        }
    }

    /**
     * 处理 RPC 请求
     *
     * 查找并执行对应的方法处理器，将结果或错误发送回服务器。
     *
     * @param request - RPC 请求消息
     */
    private async handleRpcRequest(request: RpcRequest & { type: 'rpc_request' }): Promise<void> {
        Logger.log(`📥 收到 RPC 调用: ${request.method}`);

        const handler = this.handlers.get(request.method);
        if (!handler) {
            Logger.log(`⚠️ 未找到方法处理器: ${request.method}`);
            this.send({
                type: 'rpc_response',
                id: request.id,
                error: `Method not found: ${request.method}`
            });
            return;
        }

        try {
            // 执行处理器
            const result = await handler(request.params);
            // 发送成功响应
            this.send({
                type: 'rpc_response',
                id: request.id,
                result: result ?? null
            });
            Logger.log(`✅ RPC 调用完成: ${request.method}`);
        } catch (error) {
            // 发送错误响应
            Logger.log(`❌ RPC 调用失败: ${request.method} - ${error}`);
            this.send({
                type: 'rpc_response',
                id: request.id,
                error: error instanceof Error ? error.message : String(error)
            });
        }
    }

    /**
     * 计划重连
     *
     * 连接断开后，等待 `RECONNECT_DELAY` 毫秒再尝试重连。
     * 如果已标记为 disposed 或已有重连定时器，则不执行。
     */
    private scheduleReconnect(): void {
        if (this.disposed || this.reconnectTimer) {
            return;
        }
        Logger.log(`⏳ ${RECONNECT_DELAY / 1000} 秒后重连...`);
        this.reconnectTimer = setTimeout(() => {
            this.reconnectTimer = null;
            this.connect();
        }, RECONNECT_DELAY);
    }

    /**
     * 清除重连定时器
     */
    private clearReconnectTimer(): void {
        if (this.reconnectTimer) {
            clearTimeout(this.reconnectTimer);
            this.reconnectTimer = null;
        }
    }

    /**
     * 检查是否已连接
     *
     * @returns 如果 WebSocket 处于 OPEN 状态则返回 true
     */
    public isConnected(): boolean {
        return this.ws?.readyState === WebSocket.OPEN;
    }
}

// =============================================================================
// 模块级 API
// =============================================================================

/** 全局 WebSocket 客户端单例 */
let globalClient: WebSocketClient | null = null;

/**
 * 获取 WebSocket 客户端单例
 *
 * 如果单例不存在，则创建新实例。
 *
 * @returns WebSocket 客户端实例
 */
export function getWebSocketClient(): WebSocketClient {
    if (!globalClient) {
        globalClient = new WebSocketClient();
    }
    return globalClient;
}

/**
 * 初始化 WebSocket 并注册默认方法
 *
 * 在 extension.ts 的 `activate` 函数中调用此函数完成初始化。
 *
 * ## 默认注册的方法
 *
 * - `reloadWindow`: 重新加载 VSCode 窗口
 *
 * ## 自动清理
 *
 * 函数会向 `context.subscriptions` 添加清理逻辑，
 * 确保扩展停用时断开 WebSocket 连接。
 *
 * @param context - VSCode 扩展上下文
 * @returns WebSocket 客户端实例
 *
 * @example
 * ```typescript
 * // extension.ts
 * export function activate(context: vscode.ExtensionContext) {
 *     // 初始化 WebSocket
 *     initializeWebSocket(context);
 * }
 * ```
 */
export function initializeWebSocket(context: vscode.ExtensionContext): WebSocketClient {
    const client = getWebSocketClient();

    // 注册默认方法：重载窗口
    // 当 Rust 调用 call_all_extensions("reloadWindow", {}) 时，
    // 此处理器会执行 VSCode 的窗口重载命令
    client.registerHandler('reloadWindow', () => {
        Logger.log('🔄 执行 reloadWindow');
        vscode.commands.executeCommand('workbench.action.reloadWindow');
    });

    // 连接到 WebSocket 服务器
    client.connect();

    // 注册清理逻辑：扩展停用时断开连接
    context.subscriptions.push({
        dispose: () => {
            client.disconnect();
            globalClient = null;
        }
    });

    return client;
}
