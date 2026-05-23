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