use clap::Args;
use colored::Colorize;
use std::path::PathBuf;

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
        println!("\n{}", "✓ All services started!".green().bold());
        println!("  Run {} to check health", "cookest status".cyan());
    }

    Ok(())
}
