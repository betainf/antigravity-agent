//! # WebSocket 服务器模块
//!
//! 本模块实现了 Antigravity Agent 与 VSCode 扩展之间的 WebSocket 双向通信。
//!
//! ## 背景
//!
//! 账户切换需要让 VSCode 中的 Antigravity 扩展重新加载，以便加载新的账户凭证。
//! 传统的 HTTP 轮询方式无法实现服务端主动推送，因此使用 WebSocket 实现双向通信。
//!
//! ## 架构
//!
//! ```text
//! ┌─────────────────────┐         ┌─────────────────────────────┐
//! │  VSCode Extension   │◀──WS──▶│  Tauri App (:18888/ws)      │
//! │                     │         │                             │
//! │  WebSocket Client   │         │  WebSocket Server           │
//! │  自动连接 + 重连     │         │  多客户端管理               │
//! │                     │         │                             │
//! │  注册方法处理器      │◀───────│  RPC 调用 (reloadWindow等)   │
//! └─────────────────────┘         └─────────────────────────────┘
//! ```
//!
//! ## 核心功能
//!
//! - **多客户端管理**: 支持多个 VSCode 实例同时连接，通过 `ConnectionManager` 统一管理
//! - **RPC 调用**: Rust 可主动调用扩展注册的方法（如 `reloadWindow`）
//! - **心跳检测**: 自动检测客户端断开，防止僵尸连接
//! - **广播机制**: 一次调用可推送到所有连接的 VSCode 实例
//!
//! ## 使用示例
//!
//! ```rust,ignore
//! use crate::server::websocket::{has_extension_connections, call_all_extensions};
//!
//! // 检查是否有扩展连接
//! if has_extension_connections() {
//!     // 调用所有扩展的 reloadWindow 方法
//!     call_all_extensions("reloadWindow", serde_json::json!({}));
//! }
//! ```

use actix::{Actor, ActorContext, Addr, AsyncContext, Handler, Message, StreamHandler};
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use uuid::Uuid;

// =============================================================================
// 常量配置
// =============================================================================

/// 心跳检测间隔（秒）
///
/// 服务端每隔此时间向客户端发送 Ping 包，用于检测连接是否存活。
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// 客户端超时时间（秒）
///
/// 如果客户端在此时间内没有响应心跳，则认为连接已断开。
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

// =============================================================================
// 消息协议定义
// =============================================================================

/// RPC 请求消息
///
/// 从 Rust 发送到 VSCode 扩展，请求执行指定方法。
///
/// # 字段
///
/// - `id`: 请求唯一标识符，用于匹配响应
/// - `method`: 要调用的方法名（如 "reloadWindow"）
/// - `params`: 方法参数（JSON 格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    /// 请求唯一 ID（UUID v4）
    pub id: String,
    /// 方法名
    pub method: String,
    /// 方法参数
    #[serde(default)]
    pub params: Value,
}

/// RPC 响应消息
///
/// 从 VSCode 扩展返回到 Rust，包含方法执行结果。
///
/// # 字段
///
/// - `id`: 对应请求的 ID
/// - `result`: 成功时的返回值
/// - `error`: 失败时的错误信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    /// 对应请求的 ID
    pub id: String,
    /// 成功时的返回值
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// 失败时的错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// WebSocket 消息类型枚举
///
/// 所有 WebSocket 通信使用此统一格式，通过 `type` 字段区分消息类型。
///
/// # JSON 格式示例
///
/// ```json
/// // RPC 请求
/// {"type": "rpc_request", "id": "xxx", "method": "reloadWindow", "params": {}}
///
/// // RPC 响应
/// {"type": "rpc_response", "id": "xxx", "result": null}
///
/// // 事件通知
/// {"type": "event", "name": "account_changed", "data": {"email": "..."}}
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    /// RPC 请求（Rust → 扩展）
    #[serde(rename = "rpc_request")]
    RpcRequest(RpcRequest),

    /// RPC 响应（扩展 → Rust）
    #[serde(rename = "rpc_response")]
    RpcResponse(RpcResponse),

    /// 事件通知（单向广播，不需要响应）
    #[serde(rename = "event")]
    Event { name: String, data: Value },

    /// 心跳 Ping
    #[serde(rename = "ping")]
    Ping,

    /// 心跳 Pong
    #[serde(rename = "pong")]
    Pong,
}

// =============================================================================
// 连接管理器
// =============================================================================

/// 扩展客户端信息
///
/// 存储每个连接的 VSCode 扩展实例的信息。
#[derive(Clone)]
pub struct ExtensionClient {
    /// 客户端唯一标识符
    #[allow(dead_code)]
    pub id: String,
    /// Actor 地址，用于发送消息
    pub addr: Addr<WsSession>,
}

/// 全局连接管理器
///
/// 负责管理所有已连接的 VSCode 扩展客户端。
/// 使用 `RwLock` 保证线程安全，支持并发读取和互斥写入。
///
/// # 设计说明
///
/// - 使用 `parking_lot::RwLock` 而非 `std::sync::RwLock`，性能更好
/// - 通过 `lazy_static` 实现全局单例
/// - 所有公共 API 都是线程安全的
pub struct ConnectionManager {
    /// 客户端映射表：client_id -> ExtensionClient
    clients: RwLock<HashMap<String, ExtensionClient>>,
}

impl ConnectionManager {
    /// 创建新的连接管理器
    pub fn new() -> Self {
        Self {
            clients: RwLock::new(HashMap::new()),
        }
    }

    /// 注册新客户端
    ///
    /// 当 VSCode 扩展建立 WebSocket 连接时调用。
    ///
    /// # 参数
    ///
    /// - `id`: 客户端唯一标识符（UUID）
    /// - `addr`: WebSocket Session Actor 的地址
    pub fn register(&self, id: String, addr: Addr<WsSession>) {
        let mut clients = self.clients.write();
        clients.insert(id.clone(), ExtensionClient { id, addr });
        tracing::info!(client_count = clients.len(), "WebSocket 客户端已连接");
    }

    /// 移除客户端
    ///
    /// 当 WebSocket 连接断开时调用。
    ///
    /// # 参数
    ///
    /// - `id`: 要移除的客户端 ID
    pub fn unregister(&self, id: &str) {
        let mut clients = self.clients.write();
        clients.remove(id);
        tracing::info!(client_count = clients.len(), "WebSocket 客户端已断开");
    }

    /// 获取当前连接的客户端数量
    pub fn client_count(&self) -> usize {
        self.clients.read().len()
    }

    /// 检查是否有扩展连接
    ///
    /// 用于账户切换逻辑判断是否可以使用扩展模式。
    pub fn has_connections(&self) -> bool {
        !self.clients.read().is_empty()
    }

    /// 广播消息到所有已连接的客户端
    ///
    /// # 参数
    ///
    /// - `message`: 要广播的 WebSocket 消息
    pub fn broadcast(&self, message: WsMessage) {
        let clients = self.clients.read();
        let json = serde_json::to_string(&message).unwrap();
        for client in clients.values() {
            client.addr.do_send(TextMessage(json.clone()));
        }
    }

    /// 调用所有扩展的指定方法
    ///
    /// 这是一个「发射后不管」(fire-and-forget) 的调用方式，
    /// 不会等待扩展返回响应。适用于 `reloadWindow` 等不需要返回值的操作。
    ///
    /// # 参数
    ///
    /// - `method`: 方法名
    /// - `params`: 方法参数（JSON 格式）
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// manager.call_all("reloadWindow", serde_json::json!({}));
    /// ```
    pub fn call_all(&self, method: &str, params: Value) {
        let request = RpcRequest {
            id: Uuid::new_v4().to_string(),
            method: method.to_string(),
            params,
        };
        self.broadcast(WsMessage::RpcRequest(request));
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

// 全局连接管理器单例
lazy_static::lazy_static! {
    /// 全局连接管理器实例
    ///
    /// 通过 `CONNECTION_MANAGER` 访问，在整个应用生命周期内有效。
    pub static ref CONNECTION_MANAGER: Arc<ConnectionManager> = Arc::new(ConnectionManager::new());
}

// =============================================================================
// WebSocket Session Actor
// =============================================================================

/// Actor 内部消息：发送文本
///
/// 用于从连接管理器向特定客户端发送消息。
#[derive(Message)]
#[rtype(result = "()")]
pub struct TextMessage(pub String);

/// WebSocket Session Actor
///
/// 每个 VSCode 扩展连接对应一个 `WsSession` 实例。
/// 实现了 Actix Actor 模式，处理 WebSocket 消息收发和心跳检测。
///
/// # 生命周期
///
/// 1. 客户端连接 → `started()` → 注册到 `ConnectionManager`
/// 2. 收发消息 → `handle()` 处理各种消息类型
/// 3. 连接断开 → `stopped()` → 从 `ConnectionManager` 移除
pub struct WsSession {
    /// 客户端唯一 ID（UUID v4）
    id: String,
    /// 最后一次收到消息的时间戳，用于心跳超时检测
    hb: Instant,
}

impl WsSession {
    /// 创建新的 WebSocket Session
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            hb: Instant::now(),
        }
    }

    /// 启动心跳检测定时器
    ///
    /// 每隔 `HEARTBEAT_INTERVAL` 发送 Ping 包。
    /// 如果超过 `CLIENT_TIMEOUT` 没有收到客户端响应，则断开连接。
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                tracing::warn!(client_id = %act.id, "WebSocket 客户端心跳超时");
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    /// Session 启动时调用
    fn started(&mut self, ctx: &mut Self::Context) {
        // 启动心跳检测
        self.hb(ctx);
        // 注册到全局连接管理器
        CONNECTION_MANAGER.register(self.id.clone(), ctx.address());
        tracing::debug!(client_id = %self.id, "WebSocket Session 启动");
    }

    /// Session 停止时调用
    fn stopped(&mut self, _: &mut Self::Context) {
        // 从全局连接管理器移除
        CONNECTION_MANAGER.unregister(&self.id);
        tracing::debug!(client_id = %self.id, "WebSocket Session 停止");
    }
}

/// 处理来自客户端的 WebSocket 消息
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            // WebSocket 协议级 Ping/Pong
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            // 文本消息（JSON 格式）
            Ok(ws::Message::Text(text)) => {
                self.hb = Instant::now();
                // 解析 JSON 消息
                match serde_json::from_str::<WsMessage>(&text) {
                    Ok(WsMessage::Pong) => {
                        // 应用层心跳响应，已更新 hb
                    }
                    Ok(WsMessage::RpcResponse(response)) => {
                        tracing::debug!(
                            request_id = %response.id,
                            "收到 RPC 响应"
                        );
                        // TODO: 如果需要同步等待响应，可在此处理
                    }
                    Ok(msg) => {
                        tracing::debug!(?msg, "收到 WebSocket 消息");
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "无法解析 WebSocket 消息");
                    }
                }
            }
            Ok(ws::Message::Binary(_)) => {
                tracing::warn!("不支持二进制消息");
            }
            Ok(ws::Message::Close(reason)) => {
                tracing::debug!(?reason, "客户端关闭连接");
                ctx.stop();
            }
            Err(e) => {
                tracing::error!(error = %e, "WebSocket 协议错误");
                ctx.stop();
            }
            _ => {}
        }
    }
}

/// 处理从连接管理器发送的文本消息
impl Handler<TextMessage> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: TextMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

// =============================================================================
// HTTP 路由处理
// =============================================================================

/// WebSocket 升级处理函数
///
/// 当客户端请求 `/ws` 路径时，将 HTTP 连接升级为 WebSocket 连接。
///
/// # 路由
///
/// ```text
/// GET ws://127.0.0.1:18888/ws
/// ```
pub async fn ws_handler(
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, actix_web::Error> {
    tracing::info!("新的 WebSocket 连接请求");
    ws::start(WsSession::new(), &req, stream)
}

// =============================================================================
// 公共 API
// =============================================================================

/// 检查是否有 VSCode 扩展连接
///
/// 用于账户切换逻辑判断：
/// - 返回 `true` 时，可以使用扩展模式（reloadWindow）
/// - 返回 `false` 时，需要判断 Antigravity 是否运行来决定下一步操作
///
/// # 示例
///
/// ```rust,ignore
/// if has_extension_connections() {
///     // 场景 1: 使用扩展模式切换
/// } else if is_antigravity_running() {
///     // 场景 2: 提示用户安装扩展
/// } else {
///     // 场景 3: 使用进程启动模式
/// }
/// ```
pub fn has_extension_connections() -> bool {
    CONNECTION_MANAGER.has_connections()
}

/// 获取当前连接的 VSCode 扩展数量
///
/// 可用于日志记录和用户提示。
pub fn extension_client_count() -> usize {
    CONNECTION_MANAGER.client_count()
}

/// 调用所有已连接扩展的指定方法
///
/// 这是账户切换流程的核心操作之一。当用户在 Tauri 应用中切换账户后，
/// 调用此函数通知所有连接的 VSCode 实例重新加载窗口。
///
/// # 参数
///
/// - `method`: 方法名（扩展需要预先注册对应的处理器）
/// - `params`: 方法参数（JSON 格式）
///
/// # 当前支持的方法
///
/// - `reloadWindow`: 重新加载 VSCode 窗口
///
/// # 示例
///
/// ```rust,ignore
/// // 切换账户后，广播 reloadWindow
/// call_all_extensions("reloadWindow", serde_json::json!({}));
/// ```
pub fn call_all_extensions(method: &str, params: Value) {
    tracing::info!(
        method = %method,
        client_count = CONNECTION_MANAGER.client_count(),
        "调用所有扩展方法"
    );
    CONNECTION_MANAGER.call_all(method, params);
}

/// 广播事件到所有已连接的扩展
///
/// 用于发送不需要响应的单向通知，如账户变更事件。
///
/// # 参数
///
/// - `name`: 事件名称
/// - `data`: 事件数据（JSON 格式）
///
/// # 示例
///
/// ```rust,ignore
/// broadcast_event("account_changed", serde_json::json!({
///     "email": "user@example.com"
/// }));
/// ```
#[allow(dead_code)]
pub fn broadcast_event(name: &str, data: Value) {
    CONNECTION_MANAGER.broadcast(WsMessage::Event {
        name: name.to_string(),
        data,
    });
}
