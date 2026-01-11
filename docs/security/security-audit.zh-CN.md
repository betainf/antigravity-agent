# Antigravity Agent 安全审计报告（初版）

更新时间：2026-01-11  
审计范围：本仓库当前工作区代码（Tauri 后端 + 前端 + VSCode 扩展）

## 结论概览

当前实现里存在多处高风险问题，核心集中在：

- 本地 HTTP 服务对外暴露敏感数据且启用宽松 CORS，导致“任意网页可读写本地服务数据”的风险
- 多个后端命令/功能允许前端输入直接影响文件路径（存在路径穿越与任意文件写入风险）
- 将 OAuth `client_secret` 以明文硬编码在前端/后端，属于不可接受的密钥暴露
- 账户导入导出“加密”实现为 XOR + Base64，不具备任何现实安全性
- Tauri 权限配置过宽（`fs:write-all`、`shell:default`、`process:*` 等），一旦 WebView/前端出现 XSS 或被注入，后果会被放大为本机级别的破坏

建议优先级：先修复“本地服务 + CORS + 敏感数据泄漏”与“任意写文件/路径穿越”，再处理密钥与加密方案，最后收敛权限与防护基线。

## 威胁模型（简化）

- 攻击者能力假设：能在用户机器上运行任意本地进程，或诱导用户打开恶意网页/恶意 VSCode 扩展/恶意 WebView 内容
- 重点资产：Antigravity 账户 access_token / refresh_token、账户状态数据库、备份文件、应用日志、可执行文件/配置目录
- 高风险边界：
  - `127.0.0.1:18888` 的 Actix HTTP + WebSocket 服务
  - Tauri `invoke` 暴露的后端命令（尤其是文件系统、进程、shell 相关）
  - 账户备份与导入导出链路（文件名/路径、加密/解密）

## 发现清单

### AG-SEC-001（严重/Critical）：本地 HTTP API 使用宽松 CORS 且无鉴权，可被任意网页读取敏感数据并触发敏感操作

**位置**

- `src-tauri/src/server/mod.rs`：`Cors::permissive()` + 多个 `/api/*` 路由对 localhost 暴露

**问题描述**

HTTP 服务绑定 `127.0.0.1:18888`，并对所有来源启用宽松 CORS。这意味着：

- 任意网页 JS 都可以向 `http://127.0.0.1:18888/api/*` 发起跨域请求
- 由于 CORS 宽松，网页可以读取响应内容（不仅仅是发请求）
- 当前 API 包含获取账户列表、切换账户、获取 metrics 等功能

**影响**

- 远程网页可在用户打开页面时，读取本地账户信息并触发切换账户等操作
- 若账户数据中包含 token（见 AG-SEC-006），将直接导致凭证泄露

**证据/参考**

- `src-tauri/src/server/mod.rs`：CORS 放行过宽（已改为仅允许 `vscode-webview://`、`tauri://`、本地来源）

**修复建议**

- 移除 `Cors::permissive()`，默认不启用 CORS；若必须启用，严格允许来源（而不是 *）
- 为所有 HTTP API 增加鉴权（例如随机生成的 bearer token，首次安装生成并保存到受限目录；客户端/扩展请求必须携带）
- 或者直接移除该 HTTP 服务，改为仅通过 Tauri command 与 VSCode 扩展通信（或使用更受控的 IPC）

---

### AG-SEC-002（严重/Critical）：硬编码 OAuth client_secret，属于明文密钥泄露

**位置**

- `src/services/cloudcode-api.ts`
- `src-tauri/src/commands/account_trigger_commands.rs`
- `src-tauri/src/commands/account_metrics_commands.rs`

**问题描述**

仓库中曾包含 Google OAuth `client_secret` 明文常量。任何获取到应用安装包或源码的人都能提取并滥用该 secret。

**影响**

- `client_secret` 一旦泄露，将被用于滥用 OAuth 客户端身份（视 OAuth 配置而定），导致配额损耗、风控触发、甚至影响合法用户

**修复建议**

- 对于桌面/前端应用，应使用“公开客户端”模型（PKCE）而不是在客户端保存 `client_secret`
- 若业务必须使用 `client_secret`，应把 token refresh 放到受控的服务端（你们自己的后端），客户端不接触 secret
- 立刻轮换/吊销当前泄露的 secret，并审计相关 OAuth 应用配置
- 代码层面已移除硬编码 secret：后端改从环境变量 `ANTIGRAVITY_OAUTH_CLIENT_ID` / `ANTIGRAVITY_OAUTH_CLIENT_SECRET` 读取；前端移除 token refresh API

---

### AG-SEC-003（严重/Critical）：导入导出“加密”实现为 XOR + Base64，不具备安全性

**位置**

- `src-tauri/src/commands/account_manage_commands.rs`：`encrypt_config_data` / `decrypt_config_data`

**问题描述**

当前实现仅对明文做密码重复 XOR，然后 Base64 编码。这种方案：

- 不提供机密性（可被已知明文/频率分析/穷举快速恢复）
- 不提供完整性（攻击者可篡改密文导致可控明文翻转）

**影响**

- 用户以为“加密导出”安全，但实际上导出文件可被轻易解密/篡改，导致 token 等敏感数据泄露或被植入恶意内容

**修复建议**

- 使用标准、带认证的加密：AES-256-GCM 或 ChaCha20-Poly1305
- 密钥派生使用 Argon2id / scrypt / PBKDF2（带 salt、合理参数）
- 输出格式包含：版本号、salt、nonce、ciphertext、auth tag；解密时进行认证校验

---

### AG-SEC-004（高/High）：Tauri 命令 `write_text_file` 允许任意路径写入（潜在本机破坏）

**位置**

- `src-tauri/src/commands/logging_commands.rs`：`write_text_file(path, content)`

**问题描述**

命令接受任意字符串路径并直接 `fs::write`。在 Tauri 应用中，如果前端被注入（XSS、依赖投毒、恶意插件等），攻击者可通过此命令写入：

- 启动项/配置文件
- 其它应用的脚本/配置
- 用户文档目录中的诱导文件

**影响**

- 本地持久化、破坏用户数据、进一步 RCE 的踏板

**修复建议**

- 彻底移除此命令，或将写入范围限制在应用自身目录（例如 `config_dir/logs`）
- 对 path 做 canonicalize 并校验必须在允许目录下（`starts_with(allowed_root)`）
- 限制文件扩展名与最大写入大小

---

### AG-SEC-005（高/High）：备份恢复/删除存在路径穿越，可写入/删除任意文件

**位置**

- `src-tauri/src/commands/account_manage_commands.rs`
  - `restore_backup_files`：`antigravity_dir.join(&account_file.filename)`
  - `delete_backup`：`antigravity_dir.join(format!("{}.json", name))`
- `src-tauri/src/commands/account_commands.rs`
  - `restore_antigravity_account`：`accounts_dir.join(format!("{account_name}.json"))`

**问题描述**

上述接口将来自前端/输入的 `filename`/`name`/`account_name` 直接参与路径拼接。Rust 的 `PathBuf::join` 在遇到包含分隔符的相对路径（如 `..\..\Windows\...`）或绝对路径时，可能导致跳出预期目录。

**影响**

- 攻击者可通过导入/恢复/删除功能在任意路径写入或删除文件（取决于进程权限）
- 与 AG-SEC-004、AG-SEC-008（权限过宽）叠加时风险进一步放大

**修复建议**

- 对文件名进行严格白名单校验：仅允许 `[a-zA-Z0-9@._-]` 等，并拒绝任何路径分隔符
- 强制追加 `.json` 后缀并校验最终 canonicalize 后仍在 `antigravity-accounts` 目录内
- 只接受 `Path::file_name()` 结果，拒绝带目录的输入
- 代码层面已加入白名单校验：拒绝带分隔符/冒号/非法字符的 `filename`/`name`/`account_name`

---

### AG-SEC-006（高/High）：账户数据解码后包含 access_token/refresh_token，存在泄露与误用风险（已缓解）

**位置**

- `src-tauri/src/antigravity/account.rs`：`session_response_to_json`（已改为只返回 token 存在性，不返回明文）
- `src-tauri/src/commands/account_commands.rs`：`get_antigravity_accounts_logic` 返回解码后的 `SessionResponse` JSON
- `src-tauri/src/server/mod.rs`：HTTP `get_accounts` 直接返回该 JSON

**问题描述**

账户列表接口曾把 token 作为 JSON 返回到前端/HTTP API。结合 AG-SEC-001（CORS 过宽）、AG-SEC-009（日志风险）会导致 token 暴露面扩大。

**影响**

- refresh_token 泄露后通常可长期换取 access_token，影响最严重

**修复建议**

- 默认不向前端/HTTP API 返回 token 字段；只返回 UI 必要字段（邮箱、头像、订阅信息等）
- 如确需在后端使用 token，保持在后端内存/受控存储，不经由前端透传
- 增加敏感字段脱敏（例如仅展示前后 2-4 位）

---

### AG-SEC-007（中/Medium）：WebSocket 服务无鉴权与来源校验

**位置**

- `src-tauri/src/server/websocket.rs`：`ws_handler` 接受任意连接

**问题描述**

任意本地进程/网页都可以连接 `ws://127.0.0.1:18888/ws`。虽然当前服务端主要向客户端广播 `rpc_request`，但缺少鉴权会带来：

- 未授权客户端被动监听 RPC/事件（未来如果广播更多敏感事件会变严重）
- 作为攻击面长期存在（后续迭代可能引入可被滥用的 RPC）

**修复建议**

- 与 HTTP 一致：使用随机 token 鉴权（查询参数、Header 或首包认证）
- 限制连接来源（例如校验 Origin/子协议；注意 VSCode 扩展/Node 环境可能没有浏览器 Origin）
 - 代码层面已加入基础 Origin 校验：仅允许 `vscode-webview://` / `tauri://`

---

### AG-SEC-008（中/Medium）：Tauri capability 权限配置过宽，降低整体防线

**位置**

- `src-tauri/tauri.conf.json`：`fs:write-all`、`shell:default`、`process:*` 等

**问题描述**

当前 capability 授权包含较多高危能力。一旦 WebView/前端发生注入，攻击者更容易把问题升级到系统级破坏（写任意文件、执行外部命令、重启进程等）。

**修复建议**

- 以最小权限原则收敛：仅保留确实需要的权限与目录白名单
- 禁用 `fs:write-all`，仅允许写入应用目录（或特定导出目录）
- 如无明确需求，禁用 `shell:*` 与高危 `process:*`

---

### AG-SEC-009（中/Medium）：日志链路可能写入敏感信息且缺少强制脱敏

**位置**

- `src-tauri/src/commands/logging_commands.rs`：`write_frontend_log` 直接记录 `details`
- 前端曾存在 `console.log("insertOrUpdateCurrentAccount", currentInfo)`（见 AG-SEC-011，已移除）

**问题描述**

`write_frontend_log` 接收任意 JSON 并写入 tracing 日志。若前端在错误对象或调试日志中带出 token/路径/用户信息，将落盘进日志目录。仓库虽有 `log_sanitizer.rs`，但此命令本身未强制调用脱敏逻辑。

**修复建议**

- 在 `write_frontend_log` 内部强制对 message/details 进行脱敏与长度限制
- 发布版禁用/降低前端调试日志，避免 token/PII 进入日志

---

### AG-SEC-010（中/Medium）：DatabaseMonitor 向前端推送完整数据库键值，可能包含敏感字段

**位置**

- `src-tauri/src/db_monitor.rs`：`get_complete_data` 读取 `ItemTable` 全量并推送 `newData/oldData`

**问题描述**

该模块会将数据库 `ItemTable` 全量内容转换为 JSON 并推送到前端事件。若表中包含认证信息、token、用户标识等，等同于扩大敏感数据暴露面。

**修复建议**

- 仅监控必要 key 的变更（白名单 key）
- 推送 diff 摘要即可，避免发送全量数据
- 对敏感字段做脱敏或完全不发送

---

### AG-SEC-011（低/Low）：前端存在包含敏感对象的 console.log

**位置**

- `src/modules/use-antigravity-account.ts`：`console.log("insertOrUpdateCurrentAccount", currentInfo)`（已移除）

**影响**

在调试环境或用户导出日志时可能泄露包含 token 的对象内容（取决于 `currentInfo` 结构）。

**修复建议**

- 移除或在生产环境下禁用此类日志

---

### AG-SEC-012（低/Low）：VSCode Webview 传入 openExternal URL 未做协议/域名限制

**位置**

- `vscode-extension/src/managers/antigravity-panel.ts`：`vscode.env.openExternal(vscode.Uri.parse(message.url))`

**影响**

若 Webview 内容被注入或消息通道被滥用，可能引导用户打开恶意链接。

**修复建议**

- 仅允许 `https:` 协议
- 可选：加入域名白名单（例如项目官网、GitHub 等）

## 修复优先级建议（路线图）

1. 立即：修复 AG-SEC-001 + AG-SEC-006（移除宽松 CORS、为本地 API 加鉴权、避免返回 token）
2. 立即：修复 AG-SEC-004 + AG-SEC-005（限制文件写入范围、杜绝路径穿越）
3. 尽快：修复 AG-SEC-002（移除 client_secret、改 PKCE/服务端换取）
4. 尽快：修复 AG-SEC-003（使用标准加密方案替换 XOR）
5. 后续：收敛 capability 权限、增加 CSP、安全日志与监控策略（AG-SEC-008/009/010）

