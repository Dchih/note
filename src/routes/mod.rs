use actix_web::web;

mod health;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(health::configure)
       .configure(crate::handlers::auth_configure)
       .configure(crate::handlers::note_configure);
}