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