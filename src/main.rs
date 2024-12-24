use anyhow::Result;
use clap::Parser;

mod commands;
mod chia;

#[derive(Parser)]
#[command(name = "Chia Gods Recovery Tools")]
#[command(version = "1.0")]
#[command(author = "Chris Marslender; Patrick Maslana")]
#[command(about = "Recover images, collections, and metadata")]
enum Cli {
    RecoverImage(commands::recover_image::RecoverImage),
    RecoverCollection(commands::recover_collection::RecoverCollection),
    RecoverMetadata(commands::recover_metadata::RecoverMetadata),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli {
        Cli::RecoverImage(cmd) => cmd.execute().await,
        Cli::RecoverCollection(cmd) => cmd.execute().await,
        Cli::RecoverMetadata(cmd) => cmd.execute().await,
    }
}