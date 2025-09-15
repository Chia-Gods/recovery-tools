use anyhow::Result;
use clap::Parser;

mod chia;
mod commands;

#[derive(Parser)]
#[command(name = "Chia Gods Recovery Tools")]
#[command(version = "1.0")]
#[command(author = "Chris Marslender; Patrick Maslana")]
#[command(about = "Recover images, collections, and metadata")]
struct Cli {
    /// The port for the Chia full node RPC
    #[arg(short, long, default_value = "8555", global = true)]
    port: u16,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser)]
enum Commands {
    LocateNFTData(commands::locate_nft_data::LocateNFTData),
    RecoverImage(commands::recover_image::RecoverImage),
    RecoverCollection(commands::recover_collection::RecoverCollection),
    RecoverMetadata(commands::recover_metadata::RecoverMetadata),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::LocateNFTData(cmd) => cmd.execute(cli.port).await,
        Commands::RecoverImage(cmd) => cmd.execute(cli.port).await,
        Commands::RecoverCollection(cmd) => cmd.execute(cli.port).await,
        Commands::RecoverMetadata(cmd) => cmd.execute(cli.port).await,
    }
}
