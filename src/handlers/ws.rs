use actix_web::{ HttpRequest, HttpResponse, web, Error };
use actix_web_actors::ws;
use actix::prelude::*;
use actix::ActorFutureExt;
use actix::Recipient;
use std::collections::HashMap;

#[derive(Message)]
#[rtype(result = "()")]
struct ClientMessage {
    id: usize,
    msg: String
}
#[derive(Message, Clone)]
#[rtype(result = "()")]
struct ServerMessage {
    msg: String
}

#[derive(Message)]
#[rtype(result = "usize")]
struct Connect {
    addr: Recipient<ServerMessage>,
}

#[derive(Message)]
#[rtype(result = "()")]
struct Disconnect {
    id: usize,
}

pub struct ChatServer {
    sessions: HashMap<usize, Recipient<ServerMessage>>,
    next_id: usize,
}
impl ChatServer {
    pub fn new() -> Self {
        ChatServer { sessions: HashMap::new(), next_id: 1 }
    }
}
impl Actor for ChatServer {
    type Context = Context<Self>;
}

struct WsSession {
    id: usize,
    server: Addr<ChatServer>,
}
impl WsSession {
    pub fn new (server: Addr<ChatServer>) -> Self {
        WsSession { id: 0, server: server }
    }
}
impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // 向chatserver注册
        let addr = ctx.address();

        self.server.send(Connect {
            addr: addr.recipient(),
        })
        .into_actor(self)
        .then(|res, act, ctx| {
            match res {
                Ok(id) => {
                    act.id = id;
                },
                Err(_) => {
                    println!("注册失败");
                    ctx.stop();  // 添加分号，让这个分支也返回 ()
                }
            }
            fut::ready(())  // 去掉分号，返回 Future
        })
        .wait(ctx);        
    }
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        // 注销
        self.server.do_send(Disconnect {
            id: self.id,
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
                    id: self.id,
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
        let broadcast_msg = format!("用户{}: {}", msg.id, msg.msg);

        for (_id, recipient) in &self.sessions {
            recipient.do_send(ServerMessage { msg: broadcast_msg.clone() });
        }

        println!("广播消息: {}", broadcast_msg)
    }
}

impl Handler<Connect> for ChatServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _ctx: &mut Self::Context) -> Self::Result {
        let id = self.next_id;
        self.next_id += 1;

        self.sessions.insert(id, msg.addr);
        println!("用户 {} 已连接，当前在线: {}", id, self.sessions.len());

        id
    }
}

impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _ctx: &mut Self::Context) -> Self::Result {
        self.sessions.remove(&msg.id);
        println!("用户 {} 已断开，当前在线: {}", msg.id, self.sessions.len());
    }
}


// HTTP升级到WS
pub async fn chat_route(
    req: HttpRequest,
    stream: web::Payload,
    server: web::Data<Addr<ChatServer>>
) -> Result<HttpResponse, Error> {
    let session = WsSession::new(server.get_ref().clone());
    ws::start(session, &req, stream)
}