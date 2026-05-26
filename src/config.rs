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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub enabled: bool,
    pub provider: String,
    pub ollama_model: String,
    pub ollama_vision_model: String,
    pub ollama_url: String,
    pub chat_rate_limit_free: u32,
    pub chat_rate_limit_pro: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub enabled: bool,
    pub provider: String,
    pub resend_api_key: String,
    pub from_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminConfig {
    pub email: String,
    pub password: String,
}

impl CookestConfig {
    pub fn config_path(instance_dir: &Path) -> PathBuf {
        instance_dir.join("cookest.toml")
    }

    pub fn load(instance_dir: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let path = Self::config_path(instance_dir);
        let content = std::fs::read_to_string(&path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self, instance_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path(instance_dir);
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn default_with_secrets() -> Self {
        Self {
            instance: InstanceConfig {
                name: "cookest".to_string(),
                data_dir: PathBuf::from("./data"),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            network: NetworkConfig {
                domain: "localhost".to_string(),
                https_enabled: false,
                admin_port: 3001,