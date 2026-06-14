mod commands;
mod config;
mod docker;
mod templates;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "cookest",
    version,
    about = "🍳 Cookest — self-hosted meal planning platform",
    long_about = "CLI tool for deploying and managing a self-hosted Cookest instance.\n\nRun `cookest init` to get started."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Interactive setup wizard — configure and deploy your Cookest instance
    Init(commands::init::InitArgs),

    /// Start all services
    Up(commands::up::UpArgs),

    /// Stop all services
    Down(commands::down::DownArgs),

    /// Show service health and status
    Status(commands::status::StatusArgs),

    /// Tail service logs
    Logs(commands::logs::LogsArgs),

    /// Pull latest images and restart services
    Update(commands::update::UpdateArgs),

    /// Backup PostgreSQL databases
    Backup(commands::backup::BackupArgs),

    /// Restore databases from a backup file
    Restore(commands::restore::RestoreArgs),

    /// Read or write configuration values
    Config(commands::config::ConfigArgs),
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init(args) => commands::init::run(args).await,
        Commands::Up(args) => commands::up::run(args).await,
        Commands::Down(args) => commands::down::run(args).await,
        Commands::Status(args) => commands::status::run(args).await,
        Commands::Logs(args) => commands::logs::run(args),
        Commands::Update(args) => commands::update::run(args).await,
        Commands::Backup(args) => commands::backup::run(args).await,
        Commands::Restore(args) => commands::restore::run(args).await,
        Commands::Config(args) => commands::config::run(args),
    };

    if let Err(e) = result {
        eprintln!("{} {}", colored::Colorize::red("error:"), e);
        std::process::exit(1);
    }
}
