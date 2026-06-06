use clap::Args;
#[derive(Args)]
pub struct InitArgs {}
pub async fn run(args: InitArgs) -> Result<(), Box<dyn std::error::Error>> {
  Ok(())
}
