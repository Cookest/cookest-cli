use clap::{Args, Subcommand};
use colored::Colorize;
use std::path::PathBuf;

use crate::config::CookestConfig;

#[derive(Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommand,

    /// Instance directory
    #[arg(short, long, default_value = ".", global = true)]
    pub dir: PathBuf,
}

#[derive(Subcommand)]
pub enum ConfigCommand {
    /// Get a config value by dotted path (e.g. ai.enabled)
    Get { key: String },

    /// Set a config value by dotted path
    Set { key: String, value: String },

    /// Show the full configuration
    Show,
}

pub fn run(args: ConfigArgs) -> Result<(), Box<dyn std::error::Error>> {
    let instance_dir = std::fs::canonicalize(&args.dir).unwrap_or(args.dir);

    match args.command {
        ConfigCommand::Get { key } => {
            let config = CookestConfig::load(&instance_dir)?;
            let toml_value: toml::Value = toml::to_string(&config)?.parse()?;
            match get_nested_value(&toml_value, &key) {
                Some(v) => println!("{}", format_toml_value(&v)),
                None => {
                    return Err(format!("key '{}' not found", key).into());
                }
            }
        }
        ConfigCommand::Set { key, value } => {
            let mut config = CookestConfig::load(&instance_dir)?;
            let mut toml_str = toml::to_string(&config)?;
            let mut toml_value: toml::Value = toml_str.parse()?;

            set_nested_value(&mut toml_value, &key, &value)?;

            // Re-serialize and reload to validate
            toml_str = toml::to_string_pretty(&toml_value)?;
            config = toml::from_str(&toml_str)?;
            config.save(&instance_dir)?;

            println!("{} {} = {}", "✓".green(), key.cyan(), value);
            println!(
                "{}",
                "Run `cookest up` to apply changes.".dimmed()
            );
        }
        ConfigCommand::Show => {
            let config = CookestConfig::load(&instance_dir)?;
            let mut output = toml::to_string_pretty(&config)?;

            // Mask secrets in display
            let config_clone = config.clone();
            output = output.replace(&config_clone.auth.jwt_secret, "****");
            output = output.replace(&config_clone.database.food_db_password, "****");
            output = output.replace(&config_clone.database.app_db_password, "****");
            if !config_clone.services.stripe_webhook_secret.is_empty() {
                output = output.replace(&config_clone.services.stripe_webhook_secret, "****");
            }

            println!("{}", output);
        }
    }

    Ok(())
}

fn get_nested_value<'a>(value: &'a toml::Value, path: &str) -> Option<toml::Value> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = value;
    for part in parts {
        current = current.get(part)?;
    }
    Some(current.clone())
}

fn set_nested_value(
    value: &mut toml::Value,
    path: &str,
    new_value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = value;

    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            // Last part — set the value
            if let Some(table) = current.as_table_mut() {
                let existing = table.get(*part);
                let parsed = match existing {
                    Some(toml::Value::Boolean(_)) => {
                        toml::Value::Boolean(new_value.parse()?)
                    }
                    Some(toml::Value::Integer(_)) => {
                        toml::Value::Integer(new_value.parse()?)
                    }
                    _ => toml::Value::String(new_value.to_string()),
                };
                table.insert(part.to_string(), parsed);
            } else {
                return Err(format!("cannot set value at '{path}'").into());
            }
        } else {
            current = current
                .get_mut(*part)
                .ok_or_else(|| format!("key '{}' not found", parts[..=i].join(".")))?;
        }
    }

    Ok(())
}

fn format_toml_value(value: &toml::Value) -> String {
    match value {
        toml::Value::String(s) => s.clone(),
        toml::Value::Integer(i) => i.to_string(),
        toml::Value::Float(f) => f.to_string(),
        toml::Value::Boolean(b) => b.to_string(),
        other => toml::to_string_pretty(other).unwrap_or_default(),
    }
}
