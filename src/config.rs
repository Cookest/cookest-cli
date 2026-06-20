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

    /// Validate configuration values.
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        validate_port(self.network.admin_port, "admin_port")?;
        validate_port(self.network.food_api_port, "food_api_port")?;
        validate_port(self.network.app_api_port, "app_api_port")?;
        validate_port(self.database.food_db_port, "food_db_port")?;
        validate_port(self.database.app_db_port, "app_db_port")?;

        let ports = [
            (self.network.admin_port, "admin_port"),
            (self.network.food_api_port, "food_api_port"),
            (self.network.app_api_port, "app_api_port"),
            (self.database.food_db_port, "food_db_port"),
            (self.database.app_db_port, "app_db_port"),
        ];
        for i in 0..ports.len() {
            for j in (i + 1)..ports.len() {
                if ports[i].0 == ports[j].0 {
                    return Err(format!(
                        "port conflict: {} and {} both use port {}",
                        ports[i].1, ports[j].1, ports[i].0
                    )
                    .into());
                }
            }
        }

        if self.auth.jwt_secret.len() < 32 {
            return Err("jwt_secret must be at least 32 characters".into());
        }

        if self.instance.name.is_empty() {
            return Err("instance name cannot be empty".into());
        }

        Ok(())
    }

    /// Container name prefix derived from instance name.
    pub fn container_prefix(&self) -> String {
        self.instance
            .name
            .to_lowercase()
            .replace(|c: char| !c.is_alphanumeric() && c != '-', "_")
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
                food_api_port: 8081,
                app_api_port: 8080,
            },
            database: DatabaseConfig {
                food_db_password: generate_secret(32),
                app_db_password: generate_secret(32),
                food_db_port: 5432,
                app_db_port: 5433,
            },
            auth: AuthConfig {
                jwt_secret: generate_secret(64),
                access_token_expiry_secs: 900,
                refresh_token_expiry_secs: 604800,
            },
            services: ServicesConfig {
                image_gen_enabled: false,
                stripe_enabled: false,
                stripe_webhook_secret: String::new(),
                pdf_pipeline_enabled: false,
                etl_enabled: true,
            },
            ai: AiConfig {
                enabled: true,
                provider: "ollama".to_string(),
                ollama_model: "llama3.2".to_string(),
                ollama_vision_model: "llava".to_string(),
                ollama_url: "http://ollama:11434".to_string(),
                chat_rate_limit_free: 10,
                chat_rate_limit_pro: 0,
            },
            email: EmailConfig {
                enabled: false,
                provider: "resend".to_string(),
                resend_api_key: String::new(),
                from_address: "noreply@cookest.local".to_string(),
            },
            admin: AdminConfig {
                email: String::new(),
                password: String::new(),
            },
        }
    }
}

/// Validate a port is in the valid range (1–65535).
fn validate_port(port: u16, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    if port == 0 {
        return Err(format!("{name} cannot be 0").into());
    }
    Ok(())
}

/// Generate a cryptographically random hex string of the given byte length.
pub fn generate_secret(len: usize) -> String {
    use rand::Rng;
    let mut rng = rand::rng();
    let bytes: Vec<u8> = (0..len).map(|_| rng.random()).collect();
    hex::encode(&bytes)
}

/// Hex encoding without pulling in another crate — just use the basic approach.
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }
}
