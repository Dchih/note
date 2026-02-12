# Chat Architecture

## Overview

Easynote 的聊天系统基于 **Actix Actor 模型**，通过消息驱动实现 WebSocket 实时通信和房间隔离广播。

### 技术栈
- **Actix-web** — HTTP 框架 + WebSocket 握手
- **Actix Actor** — ChatServer（单例协调者）+ WsSession（每用户一个）
- **sqlx + MySQL** — 消息持久化 + 会话管理
- **serde** — JSON 序列化/反序列化（客户端协议）

---

## Actor 架构

```mermaid
graph TB
    subgraph ChatServer["ChatServer (单例)"]
        CS_DATA["sessions: HashMap&lt;user_id, Recipient&gt;<br/>rooms: HashMap&lt;conv_id, HashSet&lt;user_id&gt;&gt;<br/>pool: MySqlPool"]
        CS_HANDLERS["Handler&lt;Connect&gt; → sessions 加人<br/>Handler&lt;Disconnect&gt; → sessions 删人 + rooms 全清理<br/>Handler&lt;Join&gt; → rooms 加人 + 推送历史消息<br/>Handler&lt;ClientMessage&gt; → 按房间广播 + 异步存库"]
    end

    subgraph WsSessionA["WsSession (用户A)"]
        A_DATA["user_id, server: Addr&lt;ChatServer&gt;"]
        A_LIFE["started() → Connect<br/>stopped() → Disconnect"]
        A_STREAM["StreamHandler: 解析 ClientAction<br/>→ Join / Msg 分发"]
        A_HANDLER["Handler&lt;ServerMessage&gt; → ctx.text()"]
    end

    subgraph WsSessionB["WsSession (用户B)"]
        B_DATA["user_id, server: Addr&lt;ChatServer&gt;"]
        B_LIFE["started() → Connect<br/>stopped() → Disconnect"]
        B_STREAM["StreamHandler: 解析 ClientAction<br/>→ Join / Msg 分发"]
        B_HANDLER["Handler&lt;ServerMessage&gt; → ctx.text()"]
    end

    WsSessionA -- "do_send(Connect/Join/ClientMessage)" --> ChatServer
    WsSessionB -- "do_send(Connect/Join/ClientMessage)" --> ChatServer
    ChatServer -- "do_send(ServerMessage)" --> WsSessionA
    ChatServer -- "do_send(ServerMessage)" --> WsSessionB
```

---

## 消息类型

### Actor 消息（内部通信）

| 消息 | 方向 | 字段 | 作用 |
|------|------|------|------|
| `Connect` | WsSession → ChatServer | user_id, addr | 注册在线用户 |
| `Disconnect` | WsSession → ChatServer | user_id | 注销用户 + 清理所有房间 |
| `Join` | WsSession → ChatServer | user_id, conversation_id | 加入房间 + 推送历史消息 |
| `ClientMessage` | WsSession → ChatServer | user_id, conversation_id, msg | 房间广播 + 消息持久化 |
| `ServerMessage` | ChatServer → WsSession | msg (JSON string) | 推送消息给客户端 |

### 客户端协议（WebSocket JSON）

客户端发送：
```json
{"action": "join", "conversation_id": 5}
{"action": "msg", "conversation_id": 5, "msg": "你好"}
```

对应 Rust 枚举（serde tag 自动分发）：
```rust
#[derive(Deserialize)]
#[serde(tag = "action")]
enum ClientAction {
    #[serde(rename = "join")]
    Join { conversation_id: i64 },
    #[serde(rename = "msg")]
    Msg { conversation_id: i64, msg: String },
}
```

---

## 数据流

### 连接 + 加入房间
```
Client → WebSocket握手(token) → JWT验证 → 创建WsSession
  WsSession::started() → Connect { user_id, addr } → ChatServer
  Client → {"action":"join","conversation_id":5}
    → WsSession 解析为 ClientAction::Join
    → Join { user_id, conversation_id } → ChatServer
    → rooms[5].insert(user_id)
    → actix::spawn: get_recent(5, 20) → 逐条发 ServerMessage → Client
```

### 发送消息
```
Client → {"action":"msg","conversation_id":5,"msg":"你好"}
  → WsSession 解析为 ClientAction::Msg
  → ClientMessage { user_id, conversation_id:5, msg } → ChatServer
  → rooms.get(5) 拿到成员集合
  → 遍历成员, sessions.get(user_id) 拿 Recipient
  → recipient.do_send(ServerMessage) → WsSession → Client
  → actix::spawn: save(pool, user_id, 5, msg) → MySQL
```

### 断开连接
```
Client 关闭连接
  → WsSession::stopped() → Disconnect { user_id } → ChatServer
  → sessions.remove(user_id)
  → 遍历所有 rooms, remove(user_id)
```

---

## HTTP API

### 认证
| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| POST | `/login` | 登录，返回 JWT token | 无 |
| POST | `/register` | 注册新用户 | 无 |

### 笔记
| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| GET | `/notes` | 获取我的笔记列表 | JWT |
| POST | `/notes` | 创建笔记 | JWT |
| GET | `/notes/{id}` | 获取单个笔记 | JWT |
| PUT | `/notes/{id}` | 更新笔记 | JWT |
| DELETE | `/notes/{id}` | 删除笔记 | JWT |

### 会话
| 方法 | 路径 | 说明 | 认证 |
|------|------|------|------|
| POST | `/conversations` | 创建会话 | JWT |
| GET | `/conversations` | 获取我的会话列表 | JWT |
| POST | `/conversations/{id}/members` | 添加成员 | JWT |

### WebSocket
| 路径 | 参数 | 说明 |
|------|------|------|
| `/ws?token=xxx` | JWT token (query param) | WebSocket 连接入口 |

---

## 分层架构

```mermaid
graph TB
    Routes["<b>Routes</b><br/>routes/mod.rs"]

    subgraph Handlers["Handlers (请求处理)"]
        H_AUTH["auth.rs<br/>登录/注册"]
        H_NOTE["note.rs<br/>笔记 CRUD"]
        H_CONV["conversation.rs<br/>会话管理"]
        H_WS["ws.rs<br/>ChatServer + WsSession"]
    end

    subgraph Services["Services (业务逻辑)"]
        S_USER["UserService<br/>注册/查询/密码验证"]
        S_NOTE["NoteService<br/>笔记 CRUD"]
        S_CONV["ConversationServices<br/>会话/成员管理"]
        S_MSG["MessageRepository<br/>消息存取"]
    end

    subgraph Models["Models (数据模型)"]
        M["User, Note, Conversation<br/>ConversationMember, Message<br/>枚举: ConversationType, MemberRole"]
    end

    DB[("MySQL<br/>users, notes, conversations<br/>conversation_member, messages")]

    Routes --> Handlers
    H_AUTH --> S_USER
    H_NOTE --> S_NOTE
    H_CONV --> S_CONV
    H_WS --> S_MSG
    Services --> Models
    Models --> DB
```

---

## Class Diagram

```mermaid
classDiagram
    class ChatServer {
        -sessions: HashMap~i64, Recipient~ServerMessage~~
        -rooms: HashMap~i64, HashSet~i64~~
        -pool: MySqlPool
        +new(pool) ChatServer
        +handle(Connect)
        +handle(Disconnect)
        +handle(Join)
        +handle(ClientMessage)
    }

    class WsSession {
        -user_id: i64
        -user_name: String
        -server: Addr~ChatServer~
        +new(server, user_id, user_name, pool) WsSession
        +started(ctx) Connect
        +stopped(ctx) Disconnect
        +StreamHandler: parse ClientAction
        +handle(ServerMessage) ctx.text()
    }

    class ClientAction {
        <<enum>>
        Join: conversation_id
        Msg: conversation_id, msg
    }

    class Connect {
        +user_id: i64
        +addr: Recipient~ServerMessage~
    }

    class Disconnect {
        +user_id: i64
    }

    class Join {
        +user_id: i64
        +conversation_id: i64
    }

    class ClientMessage {
        +user_id: i64
        +conversation_id: i64
        +msg: String
    }

    class ServerMessage {
        +msg: String
    }

    class MessageRepository {
        +save(pool, sender_id, conversation_id, content)
        +get_recent(pool, conversation_id, limit)
    }

    class ConversationServices {
        +create(pool, user_id, name, member_ids)
        +get_user_conversations(pool, user_id)
        +add_member(pool, user_id, conversation_id, role)
    }

    ChatServer "1" <-- "*" WsSession : server addr
    WsSession ..> ClientAction : parse client JSON
    WsSession ..> Connect : on started
    WsSession ..> Disconnect : on stopped
    WsSession ..> Join : join room
    WsSession ..> ClientMessage : forward message
    ChatServer ..> ServerMessage : broadcast to room
    ChatServer ..> MessageRepository : persist + load history
```

## Sequence Diagram

```mermaid
sequenceDiagram
    participant C as Client
    participant WS as WsSession
    participant CS as ChatServer
    participant DB as MySQL

    Note over C,DB: === 1. Connect ===
    C->>WS: WebSocket handshake (token)
    WS->>WS: JWT verify, create session
    WS->>CS: Connect { user_id, addr }
    CS->>CS: sessions.insert(user_id, addr)

    Note over C,DB: === 2. Join Room ===
    C->>WS: {"action":"join", "conversation_id": 5}
    WS->>WS: parse ClientAction::Join
    WS->>CS: Join { user_id, conversation_id: 5 }
    CS->>CS: rooms[5].insert(user_id)
    CS->>DB: get_recent(pool, 5, 20)
    DB-->>CS: Vec~ChatMessage~
    CS->>WS: ServerMessage { history JSON } (via Recipient)
    WS->>C: history messages

    Note over C,DB: === 3. Send Message ===
    C->>WS: {"action":"msg", "conversation_id": 5, "msg": "hello"}
    WS->>WS: parse ClientAction::Msg
    WS->>CS: ClientMessage { user_id, conversation_id: 5, msg }
    CS->>CS: rooms.get(5) → member set
    loop each member in room
        CS->>WS: ServerMessage (via Recipient)
        WS->>C: push message
    end
    CS-->>DB: spawn save(pool, user_id, 5, msg)

    Note over C,DB: === 4. Disconnect ===
    C->>WS: close connection
    WS->>CS: Disconnect { user_id }
    CS->>CS: sessions.remove(user_id)
    CS->>CS: remove user_id from all rooms
```

---

## 待完成

- [ ] 群聊创建逻辑 (create 中 members_num > 1 分支)
- [ ] Leave 消息 (退出房间但不断开连接)
- [ ] 清理未使用的 import 和 warning
- [ ] 消息格式增强 (ServerMessage 加 type 字段区分聊天/历史/系统消息)
- [ ] 生产环境配置 (CORS 限制、JWT_SECRET 更换)
