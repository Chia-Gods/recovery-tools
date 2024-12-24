use clap::Args;

#[derive(Args)]
#[command(about = "Recover metadata for the collection")]
pub struct RecoverMetadata {
    /// The coin ID that contains the metadata
    #[arg(short, long)]
    coin: String,
}

impl RecoverMetadata {
    pub async fn execute(&self) -> anyhow::Result<()> {
        println!("Recovering metadata for collection from coin: {}", self.coin);

        anyhow::Ok(())
    }
}