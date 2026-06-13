use clap::Args;
use colored::Colorize;
use std::path::PathBuf;

use crate::docker;

#[derive(Args)]
pub struct UpdateArgs {
    /// Instance directory
    #[arg(short, long, default_value = ".")]
    pub dir: PathBuf,
}

pub async fn run(args: UpdateArgs) -> Result<(), Box<dyn std::error::Error>> {
    let instance_dir = std::fs::canonicalize(&args.dir).unwrap_or(args.dir);

    println!("{}", "Pulling latest images...".dimmed());
    docker::compose_pull(&instance_dir)?;

    println!("{}", "Restarting services...".dimmed());
    docker::compose_down(&instance_dir, false)?;
    docker::compose_up(&instance_dir, true)?;

    println!("{}", "✓ Update complete!".green().bold());
    Ok(())
}
