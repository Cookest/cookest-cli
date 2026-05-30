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

    // ── Food DB ──
    services.push_str(&format!(
        r#"  food-db:
    image: postgres:16-alpine
    container_name: cookest_food_db
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
        food_db_pass = config.database.food_db_password,
        food_db_port = config.database.food_db_port,
    ));

    // ── App DB ──
    services.push_str(&format!(
        r#"  app-db:
    image: postgres:16-alpine
    container_name: cookest_app_db
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
        app_db_pass = config.database.app_db_password,
        app_db_port = config.database.app_db_port,
    ));

    // ── Food API ──
    services.push_str(&format!(
        r#"  food-api:
    image: ghcr.io/cookest/food-api:latest
    container_name: cookest_food_api
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
    container_name: cookest_app_api
    restart: unless-stopped
    ports:
      - "{app_api_port}:8080"
    environment:
      APP_DATABASE_URL: "postgresql://postgres:{app_db_pass}@app-db:5432/cookest_app"
      HOST: "0.0.0.0"
      PORT: "8080"
      JWT_SECRET: "{jwt_secret}"
      CORS_ORIGIN: "{cors_origin}"
      OLLAMA_URL: "{ollama_url}"
      OLLAMA_MODEL: "{ollama_model}"
      OLLAMA_VISION_MODEL: "{ollama_vision_model}"
      OLLAMA_VISION_TIMEOUT_SECS: "120"
      PDF_UPLOAD_DIR: "/data/pdfs"
      STRIPE_WEBHOOK_SECRET: "{stripe_secret}"
      FOOD_API_URL: "http://food-api:8081"
      FOOD_API_KEY: ""
      IMAGE_GEN_URL: "{image_gen_url}"
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
        app_api_port = config.network.app_api_port,
        app_db_pass = config.database.app_db_password,
        jwt_secret = config.auth.jwt_secret,
        cors_origin = if config.network.https_enabled {
            format!("https://{}", config.network.domain)
        } else {
            format!("http://{}:{}", config.network.domain, config.network.admin_port)
        },
        ollama_url = ollama_url,
        ollama_model = config.ai.ollama_model,
        ollama_vision_model = config.ai.ollama_vision_model,
        stripe_secret = config.services.stripe_webhook_secret,
        image_gen_url = if config.services.image_gen_enabled {
            "http://image-gen:8090".to_string()
        } else {
            String::new()
        },
    ));

    // ── Admin Panel ──
    services.push_str(&format!(
        r#"  admin:
    image: ghcr.io/cookest/admin:latest
    container_name: cookest_admin
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
      COOKEST_IMAGE_GEN_ENABLED: "{image_gen_enabled}"
      COOKEST_PDF_PIPELINE_ENABLED: "{pdf_enabled}"
    depends_on:
      - app-api
      - food-api
    networks:
      - cookest

"#,
        admin_port = config.network.admin_port,
        instance_name = config.instance.name,
        ai_enabled = config.ai.enabled,
        stripe_enabled = config.services.stripe_enabled,
        image_gen_enabled = config.services.image_gen_enabled,
        pdf_enabled = config.services.pdf_pipeline_enabled,
    ));

    // ── Ollama (optional) ──
    if config.ai.enabled {
        services.push_str(
            r#"  ollama:
    image: ollama/ollama:latest
    container_name: cookest_ollama
    restart: unless-stopped
    ports:
      - "11434:11434"
    volumes:
      - ollama_data:/root/.ollama
    networks:
      - cookest

"#,
        );
        volumes.push_str("  ollama_data:\n");
    }

    // ── Image Gen (optional) ──
    if config.services.image_gen_enabled {
        services.push_str(
            r#"  image-gen:
    image: ghcr.io/cookest/image-gen:latest
    container_name: cookest_image_gen
    restart: unless-stopped
    ports:
      - "8090:8090"
    environment:
      APP_API_URL: "http://app-api:8080"
    networks:
      - cookest

"#,
        );
    }

    // ── Caddy reverse proxy (optional, for HTTPS) ──
    if config.network.https_enabled {
        services.push_str(&format!(
            r#"  caddy:
    image: caddy:2-alpine
    container_name: cookest_caddy
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
