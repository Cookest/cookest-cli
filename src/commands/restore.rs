use clap::Args;
use colored::Colorize;
use std::path::PathBuf;

use crate::config::CookestConfig;
use crate::docker;

#[derive(Args)]
pub struct RestoreArgs {
    /// Instance directory
    #[arg(short, long, default_value = ".")]
    pub dir: PathBuf,

    /// Path to food database backup file
    #[arg(long)]
    pub food_db: Option<PathBuf>,

    /// Path to app database backup file
    #[arg(long)]
    pub app_db: Option<PathBuf>,
}

pub async fn run(args: RestoreArgs) -> Result<(), Box<dyn std::error::Error>> {
    if args.food_db.is_none() && args.app_db.is_none() {
        return Err("Specify at least one of --food-db or --app-db backup files".into());
    }

    let instance_dir = std::fs::canonicalize(&args.dir).unwrap_or(args.dir);
    let config = CookestConfig::load(&instance_dir)?;

    println!(
        "{}",
        "⚠ This will overwrite existing database data!".yellow().bold()
    );
    let confirm = dialoguer::Confirm::new()
        .with_prompt("Continue?")
        .default(false)
        .interact()?;
    if !confirm {
        println!("{}", "Aborted.".yellow());
        return Ok(());
    }

    if let Some(ref food_path) = args.food_db {
        print!("  Restoring Food DB... ");
        docker::restore_database(
            &instance_dir,
            "cookest_food_db",
            "cookest_food",
            &config.database.food_db_password,
            food_path,
        )?;
        println!("{}", "✓".green());
    }

    if let Some(ref app_path) = args.app_db {
        print!("  Restoring App DB... ");
        docker::restore_database(
            &instance_dir,
            "cookest_app_db",
            "cookest_app",
            &config.database.app_db_password,
            app_path,
        )?;
        println!("{}", "✓".green());
    }

    println!("\n{}", "✓ Restore complete!".green().bold());
    Ok(())
}
