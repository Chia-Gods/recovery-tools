use clap::Args;

#[derive(Args)]
#[command(about = "Recover a collection of images")]
pub struct RecoverCollection {
    /// The coin ID at the start of the collection
    #[arg(short, long)]
    coin: String,
}

impl RecoverCollection {
    pub fn execute(&self) {
        println!("Recovering collection from coin: {}", self.coin);

    }
}