# Code Review - 2026-02-10

## 涉及文件
- `src/services/ws.rs` — MessageRepository + ChatMessage
- `src/services/conversation.rs` — ConversationServices
- `src/handlers/ws.rs` — ChatServer + WsSession + WebSocket 路由

---

## 🔴 Critical (Must Fix)

### 1. ChatMessage.send_time 和 SQL 列名不匹配
- **文件:** `src/services/ws.rs:8`
- **问题:** `sqlx::FromRow` 按字段名映射，SQL 里 SELECT 的是 `created_at`，但结构体字段叫 `send_time`，运行时会报错
- **修复:** 改字段名为 `created_at`，或在 SQL 里用 `created_at AS send_time`

### 2. 表名不一致：conversation vs conversations
- **文件:** `src/services/conversation.rs:64`
- **问题:** `create()` 用 `conversations`（复数），`find_by_id()` 用 `conversation`（单数），其中一个会查不到表
- **修复:** 统一为同一个表名（检查数据库里实际建的是哪个）

### 3. .try_into().unwrap() 可能 panic
- **文件:** `src/services/conversation.rs:54`
- **问题:** `last_insert_id()` 返回 `u64`，转 `i64` 时如果值超过 `i64::MAX` 会 panic
- **修复:** 用 `as i64` 或 `.try_into().map_err(|_| AppError::Internal(...))?`

---

## 🟡 Suggestions (Should Consider)

### 4. conversation_id 硬编码为 1
- **文件:** `src/handlers/ws.rs:78`, `src/handlers/ws.rs:146`
- **问题:** `get_recent` 和 `save` 都用了硬编码的 `1`，所有会话共用一个 ID
- **状态:** 已知待办，接入 Conversation 后处理

### 5. conversation_type 硬编码为 0
- **文件:** `src/services/conversation.rs:23`
- **问题:** 没有使用 `ConversationType` 枚举，枚举值变了就不同步
- **修复:** 用 `.bind(ConversationType::Private)` 或根据 `members_num` 动态决定

### 6. 表名 conversation_member vs conversation_members 不一致
- **文件:** `src/services/conversation.rs:31,42,87` vs `src/services/conversation.rs:76`
- **问题:** `get_user_conversations()` 用 `conversation_members`（复数），其他方法用 `conversation_member`（单数）
- **修复:** 检查数据库实际表名，全部统一

### 7. 未使用的 import
- **文件:** `src/services/ws.rs:2` — `handlers::ClientMessage` 未使用
- **修复:** 删掉

### 8. TODO 注释已完成但未清理
- **文件:** `src/handlers/ws.rs:53`
- **修复:** 删掉 `// TODO(human): 把 id: usize...`

### 9. serde_json::to_string().unwrap()
- **文件:** `src/handlers/ws.rs:85`
- **问题:** `unwrap` 不够防御性
- **修复:** 用 `unwrap_or_default()` 或 `if let Ok(json) = ...`

---

## 🟢 Nits (Optional)

### 10. 注释风格
- **文件:** `src/services/conversation.rs:57-61`
- **问题:** `/** */` 不是 Rust 惯用风格，应该用 `///`

### 11. 循环内逐条 INSERT
- **文件:** `src/services/conversation.rs:40-50`
- **问题:** 对于少量成员没问题，人数多了可以考虑批量 INSERT

---

## ✅ What's Good

- **分层设计** — service 层用 `&MySqlPool`，不依赖 `web::Data` 框架类型
- **事务使用正确** — `create()` 多表操作用 `tx`，失败自动 ROLLBACK
- **Actor 异步模式选择恰当** — 存库用 `actix::spawn`(fire and forget)，需要 ctx 用 `ctx.spawn + into_actor`
- **错误处理一致** — 统一 `.map_err(|e| AppError::Internal(e.to_string()))` 模式
- **`save()` 返回 `Result<(), AppError>`** — 隐藏数据库实现细节，调用方只关心成功失败
