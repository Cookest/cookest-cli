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