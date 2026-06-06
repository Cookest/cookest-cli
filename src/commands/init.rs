use clap::Args;
use colored::Colorize;
use dialoguer::{Confirm, Input, Select};
use std::path::PathBuf;

use crate::config::{
    AdminConfig, AiConfig, AuthConfig, CookestConfig, DatabaseConfig, EmailConfig, InstanceConfig,
    NetworkConfig, ServicesConfig, generate_secret,
};
use crate::docker;
use crate::templates;

#[derive(Args)]
pub struct InitArgs {
    /// Directory to initialize the Cookest instance in (default: current directory)
    #[arg(short, long, default_value = ".")]
    pub dir: PathBuf,

    /// Skip interactive prompts and use defaults
    #[arg(long)]
    pub defaults: bool,
}

pub async fn run(args: InitArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!();
    println!("{}", "🍳 Cookest Self-Hosted Setup".green().bold());
    println!("{}", "━".repeat(40).dimmed());
    println!(
        "{}",
        "Welcome to Cookest! Let's configure your instance.\n".dimmed()
    );

    // Check prerequisites
    docker::check_prerequisites()?;
    println!("{} Docker and Docker Compose detected\n", "✓".green());

    let instance_dir = std::fs::canonicalize(&args.dir).unwrap_or(args.dir.clone());
    std::fs::create_dir_all(&instance_dir)?;

    // Check if already initialized
    if crate::config::CookestConfig::config_path(&instance_dir).exists() {
        let overwrite = Confirm::new()
            .with_prompt("This directory already has a Cookest config. Overwrite?")
            .default(false)
            .interact()?;
        if !overwrite {
            println!("{}", "Aborted.".yellow());
            return Ok(());
        }
    }

    let config = if args.defaults {
        let mut c = CookestConfig::default_with_secrets();
        c.admin.email = "admin@cookest.local".to_string();
        c.admin.password = generate_secret(16);
        c
    } else {
        interactive_setup()?
    };

    // Write files
    println!("\n{}", "Generating configuration files...".dimmed());

    // cookest.toml
    config.save(&instance_dir)?;
    println!("  {} cookest.toml", "✓".green());

    // docker-compose.yml
    let compose = templates::render_compose(&config);
    std::fs::write(instance_dir.join("docker-compose.yml"), &compose)?;
    println!("  {} docker-compose.yml", "✓".green());

    // .env
    let env_content = templates::render_env(&config);
    std::fs::write(instance_dir.join(".env"), &env_content)?;
    println!("  {} .env", "✓".green());

    // Caddyfile (if HTTPS)
    if config.network.https_enabled {
        let caddyfile = templates::render_caddyfile(&config);
        std::fs::write(instance_dir.join("Caddyfile"), &caddyfile)?;
        println!("  {} Caddyfile", "✓".green());
    }

    // .gitignore
    std::fs::write(
        instance_dir.join(".gitignore"),
        ".env\ncookest.toml\ndata/\nbackups/\n",
    )?;
    println!("  {} .gitignore", "✓".green());

    // Data directories
    std::fs::create_dir_all(instance_dir.join("data"))?;
    std::fs::create_dir_all(instance_dir.join("backups"))?;

    println!("\n{}", "━".repeat(40).dimmed());
    println!("{}", "✓ Instance configured successfully!".green().bold());
    println!();
    println!("{}", "Next steps:".bold());
    println!("  {} Start all services", "cookest up".cyan());
    println!("  {} Check service health", "cookest status".cyan());
    println!(
        "  {} View admin panel",
        format!(
            "http://{}:{}",
            config.network.domain, config.network.admin_port
        )
        .cyan()
    );
    println!();

    if !config.admin.password.is_empty() {
        println!("{}", "Admin credentials:".bold());
        println!("  Email:    {}", config.admin.email.cyan());
        println!("  Password: {}", config.admin.password.cyan());
        println!(
            "  {}",
            "⚠ Save these credentials — they won't be shown again.".yellow()
        );
    }

    Ok(())
}

fn interactive_setup() -> Result<CookestConfig, Box<dyn std::error::Error>> {
    let mut config = CookestConfig::default_with_secrets();

    // ── Instance ──────────────────────────────────────────
    println!("{}", "── Instance ──".bold());

    config.instance.name = Input::new()
        .with_prompt("Instance name")
        .default("cookest".to_string())
        .interact_text()?;

    // ── Network ───────────────────────────────────────────
    println!("\n{}", "── Network ──".bold());

    config.network.domain = Input::new()
        .with_prompt("Domain (or localhost for local)")
        .default("localhost".to_string())
        .interact_text()?;

    if config.network.domain != "localhost" {
        config.network.https_enabled = Confirm::new()
            .with_prompt("Enable HTTPS via Let's Encrypt?")
            .default(true)
            .interact()?;
    }

    // ── Admin ─────────────────────────────────────────────
    println!("\n{}", "── Admin Account ──".bold());

    config.admin.email = Input::new()
        .with_prompt("Admin email")
        .interact_text()?;

    config.admin.password = Input::new()
        .with_prompt("Admin password")
        .default(generate_secret(16))
        .interact_text()?;

    // ── AI ────────────────────────────────────────────────
    println!("\n{}", "── AI Features ──".bold());

    config.ai.enabled = Confirm::new()
        .with_prompt("Enable AI features (Ollama)?")
        .default(true)
        .interact()?;

    if config.ai.enabled {
        let models = vec!["llama3.2", "llama3.1:8b", "mistral", "gemma2", "phi3"];
        let selection = Select::new()
            .with_prompt("Chat model")
            .items(&models)
            .default(0)
            .interact()?;
        config.ai.ollama_model = models[selection].to_string();

        let vision_models = vec!["llava", "qwen2.5vl:7b", "llava:13b"];
        let selection = Select::new()
            .with_prompt("Vision model (for PDF processing)")
            .items(&vision_models)
            .default(0)
            .interact()?;
        config.ai.ollama_vision_model = vision_models[selection].to_string();
    }

    // ── Services ──────────────────────────────────────────
    println!("\n{}", "── Optional Services ──".bold());

    config.services.image_gen_enabled = Confirm::new()
        .with_prompt("Enable AI image generation?")
        .default(false)
        .interact()?;

    config.services.stripe_enabled = Confirm::new()
        .with_prompt("Enable Stripe payments?")
        .default(false)
        .interact()?;

    if config.services.stripe_enabled {
        config.services.stripe_webhook_secret = Input::new()
            .with_prompt("Stripe webhook secret (whsec_...)")
            .allow_empty(true)
            .interact_text()?;
    }

    config.services.pdf_pipeline_enabled = Confirm::new()
        .with_prompt("Enable PDF price scraping pipeline?")
        .default(false)
        .interact()?;

    // ── Email ─────────────────────────────────────────────
    println!("\n{}", "── Email ──".bold());

    config.email.enabled = Confirm::new()
        .with_prompt("Enable outbound email (Resend)?")
        .default(false)
        .interact()?;

    if config.email.enabled {
        config.email.resend_api_key = Input::new()
            .with_prompt("Resend API key")
            .interact_text()?;

        config.email.from_address = Input::new()
            .with_prompt("From email address")
            .default("noreply@cookest.local".to_string())
            .interact_text()?;
    }

    Ok(config)
}
