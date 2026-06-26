use clap::Args;
use colored::Colorize;
use std::path::PathBuf;

use crate::config::CookestConfig;
use crate::docker;

#[derive(Args)]
pub struct UpArgs {
    /// Instance directory
    #[arg(short, long, default_value = ".")]
    pub dir: PathBuf,

    /// Run in foreground (don't detach)
    #[arg(long)]
    pub foreground: bool,
}

pub async fn run(args: UpArgs) -> Result<(), Box<dyn std::error::Error>> {
    let instance_dir = std::fs::canonicalize(&args.dir).unwrap_or(args.dir);

    if !instance_dir.join("docker-compose.yml").exists() {
        return Err(
            "No docker-compose.yml found. Run `cookest init` first to configure your instance."
                .into(),
        );
    }

    println!("{}", "🍳 Starting Cookest services...".green().bold());
    docker::compose_up(&instance_dir, !args.foreground)?;

    if !args.foreground {
        // Auto-provision the admin account using credentials from cookest.toml
        if let Ok(config) = CookestConfig::load(&instance_dir) {
            if !config.admin.email.is_empty() && !config.admin.password.is_empty() {
                provision_admin(&config).await;
            }
        }

        println!("\n{}", "✓ All services started!".green().bold());
        println!("  Run {} to check health", "cookest status".cyan());
    }

    Ok(())
}

/// Wait for the API to be ready, then call POST /admin/setup with the stored credentials.
async fn provision_admin(config: &CookestConfig) {
    let api_url = format!(
        "http://{}:{}",
        config.network.domain, config.network.app_api_port
    );
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_default();

    // Poll /health up to 30 seconds
    print!("\n  Waiting for API");
    let mut ready = false;
    for _ in 0..30 {
        match client.get(format!("{}/health", api_url)).send().await {
            Ok(r) if r.status().is_success() => {
                ready = true;
                break;
            }
            _ => {}
        }
        print!(".");
        let _ = std::io::Write::flush(&mut std::io::stdout());
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
    println!();

    if !ready {
        println!(
            "  {} API not ready after 30s — skipping auto-setup.",
            "⚠".yellow()
        );
        println!("    Open the admin panel and complete setup there.");
        return;
    }

    let body = serde_json::json!({
        "adminEmail": config.admin.email,
        "adminPassword": config.admin.password,
        "instanceName": config.instance.name,
    });

    match client
        .post(format!("{}/admin/setup", api_url))
        .json(&body)
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => {
            println!(
                "  {} Admin account ready: {}",
                "✓".green(),
                config.admin.email.cyan()
            );
        }
        Ok(r) if r.status() == 409 => {
            // Already set up on a previous run — not an error
            println!("  {} Admin account already exists.", "ℹ".cyan());
        }
        Ok(r) => {
            println!(
                "  {} Admin setup returned {} — open the admin panel to complete setup manually.",
                "⚠".yellow(),
                r.status()
            );
        }
        Err(e) => {
            println!(
                "  {} Could not reach API for setup: {}",
                "⚠".yellow(),
                e
            );
        }
    }
}
