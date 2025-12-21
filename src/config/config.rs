use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
}


impl AppConfig {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let cfg = config::Config::builder()
            .add_source(config::Environment::default())
            .build()?;
        
        Ok(AppConfig { 
            host: cfg.get_string("app_host")?, 
            port: cfg.get_int("app_port")? as u16, 
            database_url: cfg.get_string("database_url")?,
            jwt_secret: cfg.get_string("jwt_secret")?,
        })
    }
}