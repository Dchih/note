use actix_web::web;

mod health;
mod ws;


pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(health::configure)
        .configure(ws::configure)
        .configure(crate::handlers::auth_configure)
        .configure(crate::handlers::note_configure)
        .configure(crate::handlers::conversation_configure)
        .configure(crate::handlers::friendship_configure)
        .configure(crate::handlers::user_configure);
}