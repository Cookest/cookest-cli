use clap::Args;
use colored::Colorize;
use std::path::PathBuf;

use crate::docker;

#[derive(Args)]
pub struct DownArgs {
    /// Instance directory
    #[arg(short, long, default_value = ".")]
    pub dir: PathBuf,

    /// Also remove volumes (WARNING: deletes all data)
    #[arg(long)]
    pub volumes: bool,
}

pub async fn run(args: DownArgs) -> Result<(), Box<dyn std::error::Error>> {
    let instance_dir = std::fs::canonicalize(&args.dir).unwrap_or(args.dir);

    if args.volumes {
        println!(
            "{}",
            "⚠ This will delete ALL data including databases!".yellow().bold()
        );
        let confirm = dialoguer::Confirm::new()
            .with_prompt("Are you sure?")
            .default(false)
            .interact()?;
        if !confirm {
            println!("{}", "Aborted.".yellow());
            return Ok(());
        }
    }

    println!("{}", "Stopping Cookest services...".dimmed());
    docker::compose_down(&instance_dir, args.volumes)?;
    println!("{}", "✓ All services stopped.".green().bold());

    Ok(())
}
