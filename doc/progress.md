# Easynote 开发进度

## 项目概述
Rust + Actix-web 的 WebSocket 聊天应用，带笔记功能。

## 已完成的功能

### 基础设施 (之前完成)
- [x] JWT 认证 (登录/注册/中间件)
- [x] 用户服务 (UserService: register, find_by_id, find_by_username)
- [x] 笔记 CRUD (NoteService + Handler + 路由)
- [x] WebSocket 基础 (ChatServer + WsSession + 握手)
- [x] 错误处理 (AppError: NotFound, BadRequest, Internal, Unauthorized)
- [x] 配置管理 (.env + AppConfig)

### 消息持久化 (本次完成)
- [x] **MessageRepository.save()** — `src/services/ws.rs`
  - 异步写入 MySQL，返回 `Result<(), AppError>`
  - 参数：pool, sender_id, conversation_id, content(&str)

- [x] **ChatServer 集成 MessageRepository** — `src/handlers/ws.rs`
  - ChatServer 持有 `pool: MySqlPool` 字段
  - Handler<ClientMessage> 中用 `actix::spawn` 异步存库 (fire and forget)
  - 错误用 `if let Err(e)` + `eprintln!` 记录

- [x] **历史消息加载** — `src/services/ws.rs` + `src/handlers/ws.rs`
  - `MessageRepository::get_recent()` 查询最近 N 条消息
  - 返回 `Vec<ChatMessage>` (自定义 FromRow + Serialize 结构体)
  - WsSession::started() 中用 `ctx.spawn(async.into_actor(self).map(...))` 加载并推送
  - 消息以 JSON 格式发给客户端

### 会话管理 — Service 层 (本次完成)
- [x] **ConversationServices** — `src/services/conversation.rs`
  - `create()` — 事务中创建会话 + 插入成员，返回 conversation_id
  - `find_by_id()` — query_as + fetch_optional
  - `get_user_conversations()` — JOIN 查询用户参与的所有会话
  - `add_member()` — 插入成员记录
  - 自定义 `ConversationRes` 结构体 (FromRow)

### Code Review 修复 (本次完成)
- [x] `ChatMessage.send_time` → `created_at` (字段名与 SQL 列名对齐)
- [x] 表名统一为复数 `conversations`、单数 `conversation_member`
- [x] `.try_into().unwrap()` → `as i64` (安全类型转换)
- [x] `serde_json::to_string().unwrap()` → `if let Ok(json)` (防御性编程)
- [x] `ConversationType` 加 `Serialize` derive

### 会话管理 — Handler + 路由 (本次完成)
- [x] 新建 `src/handlers/conversation.rs`
  - `POST /conversations` — 创建会话 (create)
  - `GET /conversations` — 获取我的会话列表 (list)
  - `POST /conversations/{id}/members` — 添加成员 (add_member)
- [x] 注册路由 — `src/routes/mod.rs`, `src/handlers/mod.rs`, `src/services/mod.rs`

### 会话接入 WebSocket (已完成)
- [x] `ClientMessage` 加 `conversation_id` 字段
- [x] `ClientAction` 枚举替代 `ClientMessageRecieve` (serde tag 自动分发 Join/Msg)
- [x] `WsSession::StreamHandler` 解析 JSON，match 分发 Join 和 Msg
- [x] `ChatServer` 加 `rooms: HashMap<i64, HashSet<i64>>` 房间系统
- [x] `Join` 消息 + `Handler<Join>` — 加入房间 + 加载历史消息
- [x] `Handler<ClientMessage>` 按房间广播 (遍历 room members → sessions 查 addr)
- [x] `Handler<Disconnect>` 从所有 rooms 中清理用户
- [x] `WsSession::started()` 去掉硬编码，历史消息加载移至 Handler<Join>

## 待完成

### 其他待办
- [ ] 群聊创建逻辑 (create 中 members_num > 1 分支)
- [ ] 清理未使用的 import 和 warning
- [ ] 生产环境配置 (CORS 限制、JWT_SECRET 更换)

## 关键设计决策
1. **分层原则** — Service/Repository 层用 `&MySqlPool`，不依赖 `web::Data`
2. **Actor 异步模式** — 不需要 ctx 用 `actix::spawn`，需要 ctx 用 `ctx.spawn + into_actor`
3. **MySqlPool.clone()** — 内部是 Arc，clone 只是引用计数 +1
4. **事务** — `pool.begin()` 开启，`tx.commit()` 提交，中途 ? 返回自动 ROLLBACK
5. **私聊角色** — 双方都是 Member，Owner 只在群聊使用
6. **conversation_type** — API 参数中省略，由 member_ids.len() 推导
7. **房间系统 (方案 A)** — 客户端主动发 Join 加入房间，不自动加入全部会话
8. **ChatServer 双 HashMap** — `sessions` (user_id→addr) + `rooms` (conversation_id→HashSet\<user_id\>)，一个用户可同时在多个房间

## 项目结构
```
src/
├── main.rs
├── config/config.rs          — 环境变量配置
├── error/error.rs            — AppError 自定义错误
├── handlers/
│   ├── auth.rs               — 登录/注册
│   ├── note.rs               — 笔记 CRUD
│   ├── ws.rs                 — ChatServer + WsSession + WebSocket 路由
│   └── conversation.rs       — 会话 API (create/list/add_member)
├── middleware/auth.rs        — JWT 认证中间件
├── models/
│   ├── user.rs               — User
│   ├── note.rs               — Note, CreateNote, UpdateNote
│   └── conversation.rs       — Conversation, ConversationMember, Message, 枚举
├── routes/mod.rs             — 路由配置
├── services/
│   ├── user.rs               — UserService
│   ├── note.rs               — NoteService
│   ├── ws.rs                 — MessageRepository + ChatMessage
│   └── conversation.rs       — ConversationServices + ConversationRes
└── utils/jwt.rs              — JwtUtil (generate/verify token)
```
