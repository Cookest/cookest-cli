use clap::Args;
use colored::Colorize;
use std::path::PathBuf;

use crate::config::CookestConfig;
use crate::docker;

#[derive(Args)]
pub struct BackupArgs {
    /// Instance directory
    #[arg(short, long, default_value = ".")]
    pub dir: PathBuf,

    /// Output directory for backup files
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

pub async fn run(args: BackupArgs) -> Result<(), Box<dyn std::error::Error>> {
    let instance_dir = std::fs::canonicalize(&args.dir).unwrap_or(args.dir);
    let config = CookestConfig::load(&instance_dir)?;

    let backup_dir = args
        .output
        .unwrap_or_else(|| instance_dir.join("backups"));
    std::fs::create_dir_all(&backup_dir)?;

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");

    println!("{}", "Backing up databases...".dimmed());

    let prefix = config.container_prefix();

    // Food DB
    let food_backup = backup_dir.join(format!("food_db_{timestamp}.dump"));
    print!("  Food DB... ");
    docker::backup_database(
        &instance_dir,
        &format!("{prefix}_food_db"),
        "cookest_food",
        &config.database.food_db_password,
        &food_backup,
    )?;
    println!("{}", "✓".green());

    // App DB
    let app_backup = backup_dir.join(format!("app_db_{timestamp}.dump"));
    print!("  App DB... ");
    docker::backup_database(
        &instance_dir,
        &format!("{prefix}_app_db"),
        "cookest_app",
        &config.database.app_db_password,
        &app_backup,
    )?;
    println!("{}", "✓".green());

    println!(
        "\n{} Backups saved to {}",
        "✓".green().bold(),
        backup_dir.display()
    );

    Ok(())
}
