mod config;
mod error;
mod routes;
mod models;
mod services;
mod handlers;
mod utils;
mod middleware;

use crate::config::AppConfig;

use actix_web::{App, HttpServer, web};
use tracing_actix_web::TracingLogger;
use sqlx::mysql::MySqlPoolOptions;
use actix_cors::Cors;



#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("server is starting");

    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()))
        .init();

    let config = AppConfig::from_env().expect("Failed to load Config");
    let addr = format!("{}:{}", config.host, config.port);

    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
    .expect("Failed to connect to database");

    tracing::info!("✅ Database connected");

    let config_data = web::Data::new(config);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()      // 开发环境允许所有来源
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        
        // prod
        // let cors = Cors::default()
        // .allowed_origin("https://your-frontend.com")
        // .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
        // .allowed_headers(vec!["Content-Type", "Authorization"])
        // .max_age(3600);

        App::new()
            .wrap(TracingLogger::default())
            .wrap(cors)
            .app_data(config_data.clone())
            .app_data(web::Data::new(pool.clone()))
            .configure(routes::configure) 
    })
    .bind(&addr)?
    .run()
    .await
}