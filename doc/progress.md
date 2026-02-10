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

## 待完成

### 会话管理 — Handler + 路由 (下一步)
- [ ] 新建 `src/handlers/conversation.rs`
  - `POST /conversations` — 创建会话
  - `GET /conversations` — 获取我的会话列表
  - `POST /conversations/{id}/members` — 添加成员
- [ ] 注册路由 — `src/routes/mod.rs`, `src/handlers/mod.rs`, `src/services/mod.rs`

### 会话接入 WebSocket
- [ ] ClientMessage 加 conversation_id 字段 (去掉硬编码的 1)
- [ ] 客户端发消息时指定 conversation_id
- [ ] 按 conversation_id 隔离消息广播 (目前是全局广播)

### 其他待办
- [ ] 群聊创建逻辑 (create 中 members_num > 1 分支)
- [ ] 清理未使用的 import 和 warning
- [ ] Conversation models 中 `ConversationType` 枚举实际使用
- [ ] 生产环境配置 (CORS 限制、JWT_SECRET 更换)

## 关键设计决策
1. **分层原则** — Service/Repository 层用 `&MySqlPool`，不依赖 `web::Data`
2. **Actor 异步模式** — 不需要 ctx 用 `actix::spawn`，需要 ctx 用 `ctx.spawn + into_actor`
3. **MySqlPool.clone()** — 内部是 Arc，clone 只是引用计数 +1
4. **事务** — `pool.begin()` 开启，`tx.commit()` 提交，中途 ? 返回自动 ROLLBACK
5. **私聊角色** — 双方都是 Member，Owner 只在群聊使用
6. **conversation_type** — API 参数中省略，由 member_ids.len() 推导

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
│   └── conversation.rs       — (待创建) 会话 API
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
