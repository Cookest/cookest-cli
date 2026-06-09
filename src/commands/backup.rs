use clap::Args;
#[derive(Args)]
pub struct BackupArgs {}
pub async fn run(args: BackupArgs) -> Result<(), Box<dyn std::error::Error>> {
  println!("Backup in progress...");
  Ok(())
}
