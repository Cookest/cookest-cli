use crate::config::CookestConfig;

/// Generate docker-compose.yml content based on the configuration.
pub fn render_compose(config: &CookestConfig) -> String {
    let mut services = String::new();
    let mut volumes = String::from(
        r#"volumes:
  food_db_data:
  app_db_data: