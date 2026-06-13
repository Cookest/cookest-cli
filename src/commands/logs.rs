use clap::Args;
use std::path::PathBuf;

use crate::docker;

#[derive(Args)]
pub struct LogsArgs {
    /// Instance directory
    #[arg(short, long, default_value = ".")]
    pub dir: PathBuf,

    /// Service name to filter (e.g. app-api, food-api, ollama)
    pub service: Option<String>,

    /// Follow log output
    #[arg(short, long)]
    pub follow: bool,

    /// Number of lines to show from end of logs
    #[arg(short = 'n', long)]
    pub tail: Option<u32>,
}

pub fn run(args: LogsArgs) -> Result<(), Box<dyn std::error::Error>> {
    let instance_dir = std::fs::canonicalize(&args.dir).unwrap_or(args.dir);

    docker::compose_logs(
        &instance_dir,
        args.service.as_deref(),
        args.follow,
        args.tail,
    )
}
