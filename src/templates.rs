use crate::config::CookestConfig;

/// Generate docker-compose.yml content based on the configuration.
pub fn render_compose(config: &CookestConfig) -> String {
    let mut services = String::new();
    let mut volumes = String::from(
        r#"volumes:
  food_db_data:
  app_db_data:
  pdf_uploads:
"#,
    );

    let prefix = config.container_prefix();

    // ── Food DB ──
    services.push_str(&format!(
        r#"  food-db:
    image: postgres:16-alpine
    container_name: {prefix}_food_db
    restart: unless-stopped
    environment:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: "{food_db_pass}"
      POSTGRES_DB: cookest_food
    ports:
      - "{food_db_port}:5432"
    volumes:
      - food_db_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres -d cookest_food"]
      interval: 5s
      timeout: 5s
      retries: 5
    networks:
      - cookest

"#,
        prefix = prefix,
        food_db_pass = config.database.food_db_password,
        food_db_port = config.database.food_db_port,
    ));

    // ── App DB ──
    services.push_str(&format!(
        r#"  app-db:
    image: postgres:16-alpine
    container_name: {prefix}_app_db
    restart: unless-stopped
    environment:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: "{app_db_pass}"
      POSTGRES_DB: cookest_app
    ports:
      - "{app_db_port}:5432"
    volumes:
      - app_db_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres -d cookest_app"]
      interval: 5s
      timeout: 5s
      retries: 5
    networks:
      - cookest

"#,
        prefix = prefix,
        app_db_pass = config.database.app_db_password,
        app_db_port = config.database.app_db_port,
    ));

    // ── Food API ──
    services.push_str(&format!(
        r#"  food-api:
    image: ghcr.io/cookest/food-api:latest
    container_name: {prefix}_food_api
    restart: unless-stopped
    ports:
      - "{food_api_port}:8081"
    environment:
      FOOD_DATABASE_URL: "postgresql://postgres:{food_db_pass}@food-db:5432/cookest_food"
      FOOD_HOST: "0.0.0.0"
      FOOD_PORT: "8081"
      FOOD_CORS_ORIGIN: "*"
      RUST_LOG: "info,cookest_food_api=debug"
    depends_on:
      food-db:
        condition: service_healthy
    networks:
      - cookest

"#,
        prefix = prefix,
        food_api_port = config.network.food_api_port,
        food_db_pass = config.database.food_db_password,
    ));

    // ── App API ──
    let ollama_url = if config.ai.enabled {
        "http://ollama:11434".to_string()
    } else {
        String::new()
    };

    services.push_str(&format!(
        r#"  app-api:
    image: ghcr.io/cookest/app-api:latest
    container_name: {prefix}_app_api
    restart: unless-stopped
    ports:
      - "{app_api_port}:8080"
    environment:
      APP_DATABASE_URL: "postgresql://postgres:{app_db_pass}@app-db:5432/cookest_app"
      HOST: "0.0.0.0"
      PORT: "8080"
      JWT_SECRET: "{jwt_secret}"
      JWT_ACCESS_EXPIRY_SECONDS: "{access_expiry}"
      JWT_REFRESH_EXPIRY_SECONDS: "{refresh_expiry}"
      CORS_ORIGIN: "{cors_origin}"
      OLLAMA_URL: "{ollama_url}"
      OLLAMA_MODEL: "{ollama_model}"
      OLLAMA_VISION_MODEL: "{ollama_vision_model}"
      OLLAMA_VISION_TIMEOUT_SECS: "120"
      PDF_UPLOAD_DIR: "/data/pdfs"
      STRIPE_WEBHOOK_SECRET: "{stripe_secret}"
      FOOD_API_URL: "http://food-api:8081"
      FOOD_API_KEY: ""
      RESEND_API_KEY: "{resend_key}"
      RESEND_FROM_EMAIL: "{resend_from}"
      RUST_LOG: "info,cookest_app_api=debug"
    volumes:
      - pdf_uploads:/data/pdfs
    depends_on:
      app-db:
        condition: service_healthy
      food-api:
        condition: service_started
    networks:
      - cookest

"#,
        prefix = prefix,
        app_api_port = config.network.app_api_port,
        app_db_pass = config.database.app_db_password,
        jwt_secret = config.auth.jwt_secret,
        access_expiry = config.auth.access_token_expiry_secs,
        refresh_expiry = config.auth.refresh_token_expiry_secs,
        cors_origin = if config.network.https_enabled {
            format!("https://{}", config.network.domain)
        } else {
            format!("http://{}:{}", config.network.domain, config.network.admin_port)
        },
        ollama_url = ollama_url,
        ollama_model = config.ai.ollama_model,
        ollama_vision_model = config.ai.ollama_vision_model,
        stripe_secret = config.services.stripe_webhook_secret,
        resend_key = config.email.resend_api_key,
        resend_from = config.email.from_address,
    ));

    // ── Admin Panel ──
    services.push_str(&format!(
        r#"  admin:
    image: ghcr.io/cookest/admin:latest
    container_name: {prefix}_admin
    restart: unless-stopped
    ports:
      - "{admin_port}:3000"
    environment:
      NEXT_PUBLIC_APP_API_URL: "http://app-api:8080"
      NEXT_PUBLIC_FOOD_API_URL: "http://food-api:8081"
      APP_API_INTERNAL_URL: "http://app-api:8080"
      FOOD_API_INTERNAL_URL: "http://food-api:8081"
      COOKEST_INSTANCE_NAME: "{instance_name}"
      COOKEST_AI_ENABLED: "{ai_enabled}"
      COOKEST_STRIPE_ENABLED: "{stripe_enabled}"
      COOKEST_PDF_PIPELINE_ENABLED: "{pdf_enabled}"
    depends_on:
      - app-api
      - food-api
    networks:
      - cookest

"#,
        prefix = prefix,
        admin_port = config.network.admin_port,
        instance_name = config.instance.name,
        ai_enabled = config.ai.enabled,
        stripe_enabled = config.services.stripe_enabled,
        pdf_enabled = config.services.pdf_pipeline_enabled,
    ));

    // ── Ollama (optional) ──
    if config.ai.enabled {
        services.push_str(&format!(
            r#"  ollama:
    image: ollama/ollama:latest
    container_name: {prefix}_ollama
    restart: unless-stopped
    ports:
      - "11434:11434"
    volumes:
      - ollama_data:/root/.ollama
    networks:
      - cookest

"#,
            prefix = prefix,
        ));
        volumes.push_str("  ollama_data:\n");
    }



    // ── Caddy reverse proxy (optional, for HTTPS) ──
    if config.network.https_enabled {
        services.push_str(&format!(
            r#"  caddy:
    image: caddy:2-alpine
    container_name: {prefix}_caddy
    restart: unless-stopped
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile:ro
      - caddy_data:/data
      - caddy_config:/config
    depends_on:
      - app-api
      - food-api
      - admin
    networks:
      - cookest

"#,
            prefix = prefix,
        ));
        volumes.push_str("  caddy_data:\n  caddy_config:\n");
    }

    format!(
        r#"# Cookest Self-Hosted — generated by cookest-cli v{version}
# Do not edit manually — use `cookest config` to update settings.

name: "cookest"

services:
{services}
{volumes}
networks:
  cookest:
    driver: bridge
"#,
        version = env!("CARGO_PKG_VERSION"),
        services = services,
        volumes = volumes,
    )
}

/// Generate a Caddyfile for HTTPS reverse proxy.
pub fn render_caddyfile(config: &CookestConfig) -> String {
    format!(
        r#"{{
    email {admin_email}
}}

{domain} {{
    handle /api/food/* {{
        reverse_proxy food-api:8081
    }}

    handle /api/* {{
        reverse_proxy app-api:8080
    }}

    handle {{
        reverse_proxy admin:3000
    }}
}}
"#,
        admin_email = config.admin.email,
        domain = config.network.domain,
    )
}

/// Generate the .env file content.
pub fn render_env(config: &CookestConfig) -> String {
    format!(
        r#"# Cookest Self-Hosted Environment
# Generated by cookest-cli — use `cookest config` to modify

# Database
FOOD_DB_PASSWORD={food_db_pass}
APP_DB_PASSWORD={app_db_pass}

# Auth
JWT_SECRET={jwt_secret}

# Stripe (optional)
STRIPE_WEBHOOK_SECRET={stripe_secret}

# AI
OLLAMA_MODEL={ollama_model}
OLLAMA_VISION_MODEL={ollama_vision_model}

# Email (optional)
RESEND_API_KEY={resend_key}
RESEND_FROM_EMAIL={from_email}
"#,
        food_db_pass = config.database.food_db_password,
        app_db_pass = config.database.app_db_password,
        jwt_secret = config.auth.jwt_secret,
        stripe_secret = config.services.stripe_webhook_secret,
        ollama_model = config.ai.ollama_model,
        ollama_vision_model = config.ai.ollama_vision_model,
        resend_key = config.email.resend_api_key,
        from_email = config.email.from_address,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::*;
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
                food_db_password: "food_pass".to_string(),
                app_db_password: "app_pass".to_string(),
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
                enabled: false,
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
        }
    }

    // ── Docker Compose rendering ──────────────────────────────────

    #[test]
    fn compose_contains_core_services() {
        let config = test_config();
        let compose = render_compose(&config);
        assert!(compose.contains("food-db:"));
        assert!(compose.contains("app-db:"));
        assert!(compose.contains("food-api:"));
        assert!(compose.contains("app-api:"));
        assert!(compose.contains("admin:"));
    }

    #[test]
    fn compose_uses_instance_prefix() {
        let config = test_config();
        let compose = render_compose(&config);
        assert!(compose.contains("container_name: test-instance_food_db"));
        assert!(compose.contains("container_name: test-instance_app_api"));
        assert!(compose.contains("container_name: test-instance_admin"));
    }

    #[test]
    fn compose_includes_db_passwords() {
        let config = test_config();
        let compose = render_compose(&config);
        assert!(compose.contains("food_pass"));
        assert!(compose.contains("app_pass"));
    }

    #[test]
    fn compose_includes_jwt_secret() {
        let config = test_config();
        let compose = render_compose(&config);
        assert!(compose.contains(&config.auth.jwt_secret));
    }

    #[test]
    fn compose_includes_jwt_expiry() {
        let config = test_config();
        let compose = render_compose(&config);
        assert!(compose.contains("JWT_ACCESS_EXPIRY_SECONDS: \"900\""));
        assert!(compose.contains("JWT_REFRESH_EXPIRY_SECONDS: \"604800\""));
    }

    #[test]
    fn compose_includes_email_env_vars() {
        let mut config = test_config();
        config.email.resend_api_key = "re_test_key".to_string();
        config.email.from_address = "noreply@example.com".to_string();
        let compose = render_compose(&config);
        assert!(compose.contains("RESEND_API_KEY: \"re_test_key\""));
        assert!(compose.contains("RESEND_FROM_EMAIL: \"noreply@example.com\""));
    }

    #[test]
    fn compose_excludes_ollama_when_disabled() {
        let mut config = test_config();
        config.ai.enabled = false;
        let compose = render_compose(&config);
        assert!(!compose.contains("ollama:"));
        assert!(!compose.contains("ollama_data:"));
    }

    #[test]
    fn compose_includes_ollama_when_enabled() {
        let mut config = test_config();
        config.ai.enabled = true;
        let compose = render_compose(&config);
        assert!(compose.contains("ollama:"));
        assert!(compose.contains("ollama/ollama:latest"));
        assert!(compose.contains("ollama_data:"));
    }



    #[test]
    fn compose_excludes_caddy_when_no_https() {
        let config = test_config();
        let compose = render_compose(&config);
        assert!(!compose.contains("caddy:"));
    }

    #[test]
    fn compose_includes_caddy_when_https() {
        let mut config = test_config();
        config.network.https_enabled = true;
        let compose = render_compose(&config);
        assert!(compose.contains("caddy:"));
        assert!(compose.contains("caddy:2-alpine"));
        assert!(compose.contains("caddy_data:"));
    }

    #[test]
    fn compose_has_correct_ports() {
        let config = test_config();
        let compose = render_compose(&config);
        assert!(compose.contains("\"5432:5432\""));
        assert!(compose.contains("\"5433:5432\""));
        assert!(compose.contains("\"8081:8081\""));
        assert!(compose.contains("\"8080:8080\""));
        assert!(compose.contains("\"3001:3000\""));
    }

    #[test]
    fn compose_has_network() {
        let config = test_config();
        let compose = render_compose(&config);
        assert!(compose.contains("networks:"));
        assert!(compose.contains("cookest:"));
        assert!(compose.contains("driver: bridge"));
    }

    #[test]
    fn compose_has_healthchecks() {
        let config = test_config();
        let compose = render_compose(&config);
        assert!(compose.contains("healthcheck:"));
        assert!(compose.contains("pg_isready"));
    }

    #[test]
    fn compose_has_depends_on() {
        let config = test_config();
        let compose = render_compose(&config);
        assert!(compose.contains("depends_on:"));
        assert!(compose.contains("service_healthy"));
    }

    #[test]
    fn compose_cors_origin_http_for_localhost() {
        let config = test_config();
        let compose = render_compose(&config);
        assert!(compose.contains("CORS_ORIGIN: \"http://localhost:3001\""));
    }

    #[test]
    fn compose_cors_origin_https_for_domain() {
        let mut config = test_config();
        config.network.domain = "cookest.example.com".to_string();
        config.network.https_enabled = true;
        let compose = render_compose(&config);
        assert!(compose.contains("CORS_ORIGIN: \"https://cookest.example.com\""));
    }

    #[test]
    fn compose_feature_flags_in_admin() {
        let mut config = test_config();
        config.ai.enabled = true;
        config.services.stripe_enabled = true;
        let compose = render_compose(&config);
        assert!(compose.contains("COOKEST_AI_ENABLED: \"true\""));
        assert!(compose.contains("COOKEST_STRIPE_ENABLED: \"true\""));
    }

    // ── Caddyfile rendering ───────────────────────────────────────

    #[test]
    fn caddyfile_contains_domain() {
        let mut config = test_config();
        config.network.domain = "example.com".to_string();
        let caddyfile = render_caddyfile(&config);
        assert!(caddyfile.contains("example.com {"));
    }

    #[test]
    fn caddyfile_contains_reverse_proxies() {
        let config = test_config();
        let caddyfile = render_caddyfile(&config);
        assert!(caddyfile.contains("reverse_proxy food-api:8081"));
        assert!(caddyfile.contains("reverse_proxy app-api:8080"));
        assert!(caddyfile.contains("reverse_proxy admin:3000"));
    }

    #[test]
    fn caddyfile_has_admin_email() {
        let config = test_config();
        let caddyfile = render_caddyfile(&config);
        assert!(caddyfile.contains("email admin@test.local"));
    }

    // ── .env rendering ────────────────────────────────────────────

    #[test]
    fn env_contains_secrets() {
        let config = test_config();
        let env = render_env(&config);
        assert!(env.contains("FOOD_DB_PASSWORD=food_pass"));
        assert!(env.contains("APP_DB_PASSWORD=app_pass"));
        assert!(env.contains(&format!("JWT_SECRET={}", config.auth.jwt_secret)));
    }

    #[test]
    fn env_contains_ai_models() {
        let config = test_config();
        let env = render_env(&config);
        assert!(env.contains("OLLAMA_MODEL=llama3.2"));
        assert!(env.contains("OLLAMA_VISION_MODEL=llava"));
    }

    #[test]
    fn env_contains_email_config() {
        let mut config = test_config();
        config.email.resend_api_key = "re_key123".to_string();
        let env = render_env(&config);
        assert!(env.contains("RESEND_API_KEY=re_key123"));
        assert!(env.contains("RESEND_FROM_EMAIL=noreply@test.local"));
    }
}
