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
    pub s3: S3Config,
    #[serde(default)]
    pub images: ImagesConfig,
}

/// Controls which Docker images are used when running the stack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImagesConfig {
    /// "ghcr" — pull from ghcr.io/cookest/* (default, no build required)
    /// "local" — use locally-built images produced by `cookest build`
    pub source: String,
}

impl Default for ImagesConfig {
    fn default() -> Self {
        Self { source: "ghcr".to_string() }
    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Config {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket: String,
    pub region: String,
    pub public_url: String,
}

impl CookestConfig {
    pub fn config_path(instance_dir: &Path) -> PathBuf {
        instance_dir.join("cookest.toml")
    }

    pub fn load(instance_dir: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let path = Self::config_path(instance_dir);
        let content = std::fs::read_to_string(&path)?;
        let config: Self = toml::from_str(&content)?;
        config.validate()?;
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
            s3: S3Config {
                endpoint: "http://minio:9000".to_string(),
                access_key: "minioadmin".to_string(),
                secret_key: generate_secret(16),
                bucket: "cookest-images".to_string(),
                region: "us-east-1".to_string(),
                public_url: "".to_string(), // Will be populated dynamically based on network
            },
            images: ImagesConfig::default(),
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
    let bytes: Vec<u8> = (0..len).map(|_| rng.random::<u8>()).collect();
    hex::encode(&bytes)
}

/// Hex encoding without pulling in another crate — just use the basic approach.
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_config() -> CookestConfig {
        CookestConfig {
            instance: InstanceConfig {
                name: "test-instance".to_string(),
                data_dir: PathBuf::from("./data"),
                version: "0.1.0".to_string(),
            },
            network: NetworkConfig {
                domain: "localhost".to_string(),
                https_enabled: false,
                admin_port: 3001,
                food_api_port: 8081,
                app_api_port: 8080,
            },
            database: DatabaseConfig {
                food_db_password: "food_secret_123".to_string(),
                app_db_password: "app_secret_456".to_string(),
                food_db_port: 5432,
                app_db_port: 5433,
            },
            auth: AuthConfig {
                jwt_secret: "a".repeat(64),
                access_token_expiry_secs: 900,
                refresh_token_expiry_secs: 604800,
            },
            services: ServicesConfig {
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
                from_address: "noreply@test.local".to_string(),
            },
            admin: AdminConfig {
                email: "admin@test.local".to_string(),
                password: "test_password".to_string(),
            },
            s3: S3Config {
                endpoint: "http://minio:9000".to_string(),
                access_key: "minioadmin".to_string(),
                secret_key: "minioadmin".to_string(),
                bucket: "cookest-images".to_string(),
                region: "us-east-1".to_string(),
                public_url: "http://localhost:9000/cookest-images".to_string(),
            },
            images: ImagesConfig::default(),
        }
    }

    // ── Secret generation ─────────────────────────────────────────

    #[test]
    fn generate_secret_correct_length() {
        let secret = generate_secret(32);
        assert_eq!(secret.len(), 64);
    }

    #[test]
    fn generate_secret_is_hex() {
        let secret = generate_secret(16);
        assert!(secret.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn generate_secret_unique() {
        let a = generate_secret(32);
        let b = generate_secret(32);
        assert_ne!(a, b);
    }

    #[test]
    fn generate_secret_zero_length() {
        let secret = generate_secret(0);
        assert_eq!(secret, "");
    }

    // ── Container prefix ──────────────────────────────────────────

    #[test]
    fn container_prefix_basic() {
        let config = test_config();
        assert_eq!(config.container_prefix(), "test-instance");
    }

    #[test]
    fn container_prefix_special_chars() {
        let mut config = test_config();
        config.instance.name = "My Instance!".to_string();
        assert_eq!(config.container_prefix(), "my_instance_");
    }

    #[test]
    fn container_prefix_keeps_hyphens() {
        let mut config = test_config();
        config.instance.name = "my-app".to_string();
        assert_eq!(config.container_prefix(), "my-app");
    }

    #[test]
    fn container_prefix_alphanumeric() {
        let mut config = test_config();
        config.instance.name = "cookest123".to_string();
        assert_eq!(config.container_prefix(), "cookest123");
    }

    // ── Config validation ─────────────────────────────────────────

    #[test]
    fn valid_config_passes() {
        let config = test_config();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn port_conflict_detected() {
        let mut config = test_config();
        config.network.admin_port = 8080;
        config.network.app_api_port = 8080;
        let err = config.validate().unwrap_err().to_string();
        assert!(err.contains("port conflict"));
    }

    #[test]
    fn jwt_secret_too_short() {
        let mut config = test_config();
        config.auth.jwt_secret = "short".to_string();
        let err = config.validate().unwrap_err().to_string();
        assert!(err.contains("jwt_secret"));
    }

    #[test]
    fn empty_instance_name_rejected() {
        let mut config = test_config();
        config.instance.name = String::new();
        let err = config.validate().unwrap_err().to_string();
        assert!(err.contains("instance name"));
    }

    // ── Config save/load roundtrip ────────────────────────────────

    #[test]
    fn config_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let config = test_config();
        config.save(dir.path()).unwrap();

        let loaded = CookestConfig::load(dir.path()).unwrap();
        assert_eq!(loaded.instance.name, config.instance.name);
        assert_eq!(loaded.network.domain, config.network.domain);
        assert_eq!(loaded.database.food_db_password, config.database.food_db_password);
        assert_eq!(loaded.auth.jwt_secret, config.auth.jwt_secret);
        assert_eq!(loaded.ai.enabled, config.ai.enabled);
        assert_eq!(loaded.ai.ollama_model, config.ai.ollama_model);
        assert_eq!(loaded.services.stripe_enabled, config.services.stripe_enabled);
        assert_eq!(loaded.email.from_address, config.email.from_address);
        assert_eq!(loaded.admin.email, config.admin.email);
    }

    #[test]
    fn config_load_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        assert!(CookestConfig::load(dir.path()).is_err());
    }

    #[test]
    fn config_load_invalid_toml() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("cookest.toml"), "invalid { toml [").unwrap();
        assert!(CookestConfig::load(dir.path()).is_err());
    }

    #[test]
    fn config_path_correct() {
        let dir = PathBuf::from("/tmp/test");
        assert_eq!(
            CookestConfig::config_path(&dir),
            PathBuf::from("/tmp/test/cookest.toml")
        );
    }

    // ── Default config ────────────────────────────────────────────

    #[test]
    fn default_config_has_valid_secrets() {
        let config = CookestConfig::default_with_secrets();
        assert!(config.auth.jwt_secret.len() >= 64);
        assert!(config.database.food_db_password.len() >= 32);
        assert!(config.database.app_db_password.len() >= 32);
    }

    #[test]
    fn default_config_passes_validation() {
        let config = CookestConfig::default_with_secrets();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn default_config_no_port_conflicts() {
        let config = CookestConfig::default_with_secrets();
        let ports = [
            config.network.admin_port,
            config.network.food_api_port,
            config.network.app_api_port,
            config.database.food_db_port,
            config.database.app_db_port,
        ];
        for i in 0..ports.len() {
            for j in (i + 1)..ports.len() {
                assert_ne!(ports[i], ports[j]);
            }
        }
    }
}
