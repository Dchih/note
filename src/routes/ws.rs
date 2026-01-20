use crate::handlers::{ chat_route };
use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
  cfg.route("/ws", web::get().to(chat_route));
}