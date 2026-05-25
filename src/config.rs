use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Central configuration for a Cookest self-hosted instance.
/// Persisted as `cookest.toml` in the instance directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookestConfig {
    pub instance: InstanceConfig,
    pub network: NetworkConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub services: ServicesConfig,
    pub ai: AiConfig,
    pub email: EmailConfig,
    pub admin: AdminConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceConfig {
    pub name: String,
    pub data_dir: PathBuf,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub domain: String,
    pub https_enabled: bool,
    pub admin_port: u16,
    pub food_api_port: u16,
    pub app_api_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub food_db_password: String,
    pub app_db_password: String,
    pub food_db_port: u16,
    pub app_db_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub access_token_expiry_secs: u64,
    pub refresh_token_expiry_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicesConfig {
    pub image_gen_enabled: bool,
    pub stripe_enabled: bool,
    pub stripe_webhook_secret: String,
    pub pdf_pipeline_enabled: bool,
    pub etl_enabled: bool,
}