use actix_web::dev::Server;
use actix_web::{ HttpRequest, HttpResponse, web, Error };
use actix_web_actors::ws;
use actix::prelude::*;
use actix::Recipient;
use serde::Serialize;
use sqlx::MySqlPool;
use std::collections::HashMap;
use std::collections::HashSet;

use serde::{Deserialize};
use crate::error::AppError;
use crate::services::MessageRepository;
use crate::utils::JwtUtil;
use crate::config::AppConfig;
use crate::services::UserService;

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    user_id: i64,
    msg: String,
    conversation_id: i64,
}
#[derive(Message, Clone)]
#[rtype(result = "()")]
struct ServerMessage {
    msg: String
}

// #[derive(Debug, Deserialize, Serialize)]
// pub struct ClientMessageRecieve {
//     conversation_id: i64,
//     msg: String
// }
#[derive(Deserialize)]
#[serde(tag = "action")]
enum ClientAction {
    #[serde(rename = "join")]
    Join { conversation_id: i64 },
    #[serde(rename = "msg")]
    Msg { conversation_id: i64, msg: String }
}

#[derive(Message)]
#[rtype(result = "()")]
struct Connect {
    user_id: i64,
    addr: Recipient<ServerMessage>,
}

#[derive(Message)]
#[rtype(result = "()")]
struct Disconnect {
    user_id: i64,
}

#[derive(Message)]
#[rtype(result = "()")]
struct Join {
    user_id: i64,
    conversation_id: i64
}

pub struct ChatServer {
    /**
     * This i64 is for user_id
     */
    sessions: HashMap<i64, Recipient<ServerMessage>>,
    /**
     * HashMap<i64... The first i64 is for conversation_id
     * HashSet<i64> The second i64 is for user_id
     */
    rooms: HashMap<i64, HashSet<i64>>,  
    pool: MySqlPool
}
impl ChatServer {
    pub fn new(pool: MySqlPool) -> Self {
        ChatServer { 
            sessions: HashMap::new(),
            pool,
            rooms: HashMap::new()
        }
    }
}
impl Actor for ChatServer {
    type Context = Context<Self>;
}

struct WsSession {
    // TODO(human): 把 id: usize 替换为真实用户身份字段，并修改 new() 的参数和构造
    user_id: i64,
    user_name: String,
    server: Addr<ChatServer>,
    pool: MySqlPool
}
impl WsSession {
    pub fn new (server: Addr<ChatServer>, user_id: i64, user_name: String, pool: MySqlPool) -> Self {
        WsSession { user_id, user_name, server, pool }
    }
}
impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // 向chatserver注册
        let addr = ctx.address();

        self.server.do_send(Connect {
            user_id: self.user_id,
            addr: addr.recipient(),
        });

        
    }
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        // 注销
        self.server.do_send(Disconnect {
            user_id: self.user_id,
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(
        &mut self, 
        msg: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context
    ) {
        
        match msg {
            Ok(ws::Message::Text(text)) => {
                // 处理客户端消息
                if let Ok(action) = serde_json::from_str::<ClientAction>(&text) {
                    match action {
                        ClientAction::Join { conversation_id } => {
                            self.server.do_send(Join {
                                user_id: self.user_id,
                                conversation_id
                            });
                        },
                        ClientAction::Msg { conversation_id, msg } => {
                            self.server.do_send(ClientMessage {
                                user_id: self.user_id,
                                msg,
                                conversation_id
                            });
                        }
                    }
                } 
            },
            Ok(ws::Message::Ping(text)) => {
                ctx.pong(&text);
            },
            _ => {}
        }
    }
}

impl Handler<ServerMessage> for WsSession {
    type Result = ();
    fn handle(&mut self, msg: ServerMessage, ctx: &mut Self::Context) {
        ctx.text(msg.msg);
    }
}

impl Handler<ClientMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _ctx: &mut Self::Context) -> Self::Result {
        let broadcast_msg = format!("用户{}: {}", msg.user_id, msg.msg);
        
        if let Some(room_members) = self.rooms.get(&msg.conversation_id) {
            for user_id in room_members {
                if let Some(recipient) = self.sessions.get(user_id) {
                    recipient.do_send(ServerMessage { msg: broadcast_msg.clone() });
                }
            }
        }

        let pool = self.pool.clone();
        actix::spawn(async move {
           if let Err(e) =  MessageRepository::save(&pool, msg.user_id, msg.conversation_id, &msg.msg).await {
            eprintln!("消息保存失败：{}", e);
           }
        });

        println!("广播消息: {}", broadcast_msg)
    }
}

impl Handler<Connect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _ctx: &mut Self::Context) -> Self::Result {

        self.sessions.insert(msg.user_id, msg.addr);
        println!("用户 {} 已连接，当前在线: {}", msg.user_id, self.sessions.len());

    }
}

impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _ctx: &mut Self::Context) -> Self::Result {
        self.sessions.remove(&msg.user_id);
        for (_key, room) in &mut self.rooms {
            room.remove(&msg.user_id);
        }
        println!("用户 {} 已断开，当前在线: {}", msg.user_id, self.sessions.len());
    }
}

impl Handler<Join> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Join, _ctx: &mut Self::Context) -> Self::Result {
        self.rooms.entry(msg.conversation_id)
                .or_insert_with(HashSet::new)
                .insert(msg.user_id);
        
        let pool = self.pool.clone();
        let recipient = self.sessions.get(&msg.user_id).cloned();

        actix::spawn(async move {
           if let Ok(messages) =  MessageRepository::get_recent(&pool, msg.conversation_id, 20).await {
                if let Some(recipient) = recipient {
                    for message in messages {
                        if let Ok(m) = serde_json::to_string(&message) {
                            recipient.do_send(ServerMessage { msg: m });
                        }
                    }
                }
            }
        });
    }
}

#[derive(Deserialize)]
pub struct Token {
    token: String
}


pub async fn chat_route(
    req: HttpRequest,
    stream: web::Payload,
    server: web::Data<Addr<ChatServer>>,
    token: web::Query<Token>,
    config: web::Data<AppConfig>,
    pool: web::Data<MySqlPool>
) -> Result<HttpResponse, Error> {
    let token_handled = JwtUtil::verify_token(&token.token, &config.jwt_secret);

    match token_handled {
        Ok(claims) => {
            let user = UserService::find_by_id(pool.get_ref(), claims.sub).await.map_err(|_| actix_web::error::ErrorUnauthorized("用户不存在"))?;
            let session = WsSession::new(server.get_ref().clone(), claims.sub, user.username, pool.get_ref().clone());
            ws::start(session, &req, stream)
        },
        Err(_e) => {
            Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "code": 401,
                "message": "token 解析失败" 
            })))
        }
    }
    
}