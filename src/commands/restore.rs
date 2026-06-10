use clap::Args;
#[derive(Args)]
pub struct RestoreArgs {}
pub async fn run(args: RestoreArgs) -> Result<(), Box<dyn std::error::Error>> {
  println!("Restore in progress...");
  Ok(())
}
