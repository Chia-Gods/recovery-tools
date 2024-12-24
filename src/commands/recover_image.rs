use std::env;
use anyhow::{anyhow, Result};
use clap::Args;
use dg_xch_clients::api::full_node::FullnodeAPI;
use dg_xch_core::blockchain::sized_bytes::{Bytes32, SizedBytes};
use tokio::fs;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use crate::chia::client::get_chia_client;
use crate::chia::image::get_image;

#[derive(Args)]
#[command(about = "Recover a single image")]
pub struct RecoverImage {
    /// The coin ID at the start of the image
    #[arg(short, long)]
    coin: String,
}

impl RecoverImage {
    pub async fn execute(&self) -> Result<()> {
        println!("Recovering image from coin: {}", &self.coin);
        let client = get_chia_client(8555);

        let coinid = hex::decode(&self.coin)?;
        let coinidb32 = Bytes32::new(&coinid);
        let current_coin = client.get_coin_record_by_name(&coinidb32).await?.ok_or(anyhow!("No Coin Record found."))?;
        let puzz_solution = client.get_puzzle_and_solution(&coinidb32, current_coin.spent_block_index).await?;

        let cwd = env::current_dir()?;
        let outputdir = cwd.join("output-images");
        fs::create_dir_all(&outputdir).await?;

        let (image_data, filename) = get_image(client, current_coin, puzz_solution).await?;

        let final_filename = filename.unwrap_or(format!("{}.png", self.coin));
        let output_file_name = outputdir.join(final_filename);
        let mut file = OpenOptions::new()
            .write(true)  // Open the file for writing
            .create(true) // Create the file if it does not exist
            .truncate(true)// Overwrite any existing content
            .open(&output_file_name).await?;
        file.write_all(&image_data).await?;

        anyhow::Ok(())
    }
}