use crate::chia::client::get_chia_client;
use crate::chia::image::get_image;
use anyhow::{anyhow, Result};
use clap::Args;
use dg_xch_clients::api::full_node::FullnodeAPI;
use recovery_tools::coin_id_from_string;
use std::env;
use tokio::fs;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

#[derive(Args)]
#[command(about = "Recover a single image")]
pub struct RecoverImage {
    /// The coin ID at the start of the image
    #[arg(short, long)]
    coin: String,
}

impl RecoverImage {
    pub async fn execute(&self, port: u16) -> Result<()> {
        let Self { coin } = self;
        println!("Recovering image from coin: {coin}");
        let client = get_chia_client(port);

        let coinid = coin_id_from_string(coin)?;
        let current_coin = client
            .get_coin_record_by_name(&coinid)
            .await?
            .ok_or(anyhow!("No Coin Record found."))?;
        let puzz_solution = client
            .get_puzzle_and_solution(&coinid, current_coin.spent_block_index)
            .await?;

        let cwd = env::current_dir()?;
        let outputdir = cwd.join("output-images");
        fs::create_dir_all(&outputdir).await?;

        let image_result = get_image(&client, &current_coin, &puzz_solution).await?;
        let final_filename = image_result.filename.unwrap_or(format!("{coin}.png"));
        let output_file_name = outputdir.join(&final_filename);
        let mut file = OpenOptions::new()
            .write(true) // Open the file for writing
            .create(true) // Create the file if it does not exist
            .truncate(true) // Overwrite any existing content
            .open(&output_file_name)
            .await?;
        file.write_all(&image_result.data).await?;
        println!("Wrote {}", &final_filename);

        anyhow::Ok(())
    }
}
