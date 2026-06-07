use clap::Args;
#[derive(Args)]
pub struct StatusArgs {}
pub async fn run(args: StatusArgs) -> Result<(), Box<dyn std::error::Error>> {
  println!("Checking status...");
  Ok(())
}
