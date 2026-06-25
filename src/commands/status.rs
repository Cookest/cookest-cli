use clap::Args;
use colored::Colorize;
use std::path::PathBuf;

use crate::config::CookestConfig;

#[derive(Args)]
pub struct StatusArgs {
    /// Instance directory
    #[arg(short, long, default_value = ".")]
    pub dir: PathBuf,
}

pub async fn run(args: StatusArgs) -> Result<(), Box<dyn std::error::Error>> {
    let instance_dir = std::fs::canonicalize(&args.dir).unwrap_or(args.dir);
    let config = CookestConfig::load(&instance_dir)?;

    println!("{}", "🍳 Cookest Instance Status".green().bold());
    println!("{}", "━".repeat(50).dimmed());
    println!(
        "  Instance: {}",
        config.instance.name.cyan()
    );
    println!(
        "  Domain:   {}",
        config.network.domain.cyan()
    );
    println!();

    // Service health checks
    println!("{}", "Services:".bold());

    let prefix = config.container_prefix();
    let services = [
        ("Food DB", format!("{prefix}_food_db"), true),
        ("App DB", format!("{prefix}_app_db"), true),
        ("Food API", format!("{prefix}_food_api"), true),
        ("App API", format!("{prefix}_app_api"), true),
        ("Admin Panel", format!("{prefix}_admin"), true),
        ("Ollama", format!("{prefix}_ollama"), config.ai.enabled),
        ("Caddy", format!("{prefix}_caddy"), config.network.https_enabled),
    ];

    for (name, container, enabled) in &services {
        if !enabled {
            println!("  {} {} {}", "○".dimmed(), name, "(disabled)".dimmed());
            continue;
        }

        let output = std::process::Command::new("docker")
            .args(["inspect", "--format", "{{.State.Status}}", container])
            .output();

        match output {
            Ok(o) if o.status.success() => {
                let status = String::from_utf8_lossy(&o.stdout).trim().to_string();
                let icon = if status == "running" {
                    "●".green()
                } else {
                    "●".red()
                };
                println!("  {} {} — {}", icon, name, status);
            }
            _ => {
                println!("  {} {} — {}", "●".red(), name, "not found");
            }
        }
    }

    // Health check endpoints
    println!("\n{}", "Health Checks:".bold());

    let endpoints = [
        (
            "Food API",
            format!("http://localhost:{}/health", config.network.food_api_port),
        ),
        (
            "App API",
            format!("http://localhost:{}/health", config.network.app_api_port),
        ),
    ];

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()?;

    for (name, url) in &endpoints {
        match client.get(url).send().await {
            Ok(resp) if resp.status().is_success() => {
                println!("  {} {} — {}", "✓".green(), name, "healthy".green());
            }
            Ok(resp) => {
                println!(
                    "  {} {} — {} ({})",
                    "✗".red(),
                    name,
                    "unhealthy".red(),
                    resp.status()
                );
            }
            Err(_) => {
                println!("  {} {} — {}", "✗".red(), name, "unreachable".red());
            }
        }
    }

    // Feature flags
    println!("\n{}", "Features:".bold());
    println!(
        "  AI:             {}",
        if config.ai.enabled {
            format!("enabled ({})", config.ai.ollama_model).green()
        } else {
            "disabled".to_string().dimmed()
        }
    );

    println!(
        "  Stripe:         {}",
        if config.services.stripe_enabled {
            "enabled".green()
        } else {
            "disabled".dimmed()
        }
    );
    println!(
        "  PDF Pipeline:   {}",
        if config.services.pdf_pipeline_enabled {
            "enabled".green()
        } else {
            "disabled".dimmed()
        }
    );
    println!(
        "  HTTPS:          {}",
        if config.network.https_enabled {
            "enabled".green()
        } else {
            "disabled".dimmed()
        }
    );

    Ok(())
}
