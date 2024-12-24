use clap::Args;

#[derive(Args)]
#[command(about = "Recover a single image")]
pub struct RecoverImage {
    /// The coin ID at the start of the image
    #[arg(short, long)]
    coin: String,
}

impl RecoverImage {
    pub fn execute(&self) {
        println!("Recovering image from coin: {}", self.coin);

    }
}