use actix_web::{ HttpRequest, HttpResponse, web, Error };
use actix_web_actors::ws;
use actix::prelude::*;
use actix::Recipient;
use sqlx::MySqlPool;
use std::collections::HashMap;

use serde::{Deserialize};
use crate::utils::JwtUtil;
use crate::config::AppConfig;
use crate::services::UserService;

#[derive(Message)]
#[rtype(result = "()")]
struct ClientMessage {
    user_id: i64,
    msg: String
}
#[derive(Message, Clone)]
#[rtype(result = "()")]
struct ServerMessage {
    msg: String
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

pub struct ChatServer {
    sessions: HashMap<i64, Recipient<ServerMessage>>,
}
impl ChatServer {
    pub fn new() -> Self {
        ChatServer { sessions: HashMap::new() }
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
}
impl WsSession {
    pub fn new (server: Addr<ChatServer>, user_id: i64, user_name: String) -> Self {
        WsSession { user_id, user_name, server: server }
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
                self.server.do_send(ClientMessage {
                    user_id: self.user_id,
                    msg: text.to_string()
                });
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

        for (_id, recipient) in &self.sessions {
            recipient.do_send(ServerMessage { msg: broadcast_msg.clone() });
        }

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
        println!("用户 {} 已断开，当前在线: {}", msg.user_id, self.sessions.len());
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
            let session = WsSession::new(server.get_ref().clone(), claims.sub, user.username);
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