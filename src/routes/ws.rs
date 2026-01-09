use crate::handlers::echo;
use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
  cfg.route("/ws", web::get().to(echo));
}