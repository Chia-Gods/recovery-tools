use clap::Args;

#[derive(Args)]
#[command(about = "Recover metadata for the collection")]
pub struct RecoverMetadata {
    /// The coin ID that contains the metadata
    #[arg(short, long)]
    coin: String,
}

impl RecoverMetadata {
    pub fn execute(&self) {
        println!("Recovering metadata for collection from coin: {}", self.coin);

    }
}