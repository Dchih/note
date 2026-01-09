use actix_web::{ Error, HttpRequest, HttpResponse, rt, web };
use actix_ws::AggregatedMessage;
use futures_util::StreamExt as _;

pub async fn echo(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;

    let peer = req.connection_info().peer_addr().map(|s| s.to_string());
    if let Some(host) = peer {
      tracing::info!("🔗 WebSocket 连接建立: {:?}", host);
    }
    

    let mut stream = stream.aggregate_continuations().max_continuation_size(2_usize.pow(20));

    rt::spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(AggregatedMessage::Text(text)) => {
                    tracing::info!("📨 收到文本消息: {}", text);
                    session.text(text).await.unwrap();
                }
                Ok(AggregatedMessage::Binary(bin)) => {
                    tracing::info!("📦 收到二进制消息: {} bytes", bin.len());
                    session.binary(bin).await.unwrap();
                }
                Ok(AggregatedMessage::Ping(msg)) => {
                    tracing::debug!("🏓 收到 Ping");
                    session.pong(&msg).await.unwrap();
                }
                Ok(AggregatedMessage::Close(reason)) => {
                    tracing::info!("👋 WebSocket 关闭: {:?}", reason);
                    break;
                }
                Err(e) => {
                    tracing::error!("❌ WebSocket 错误: {:?}", e);
                    break;
                }
                _ => {}
            }
        }
        tracing::info!("🔌 WebSocket 连接断开");
    });

    Ok(res)
}
