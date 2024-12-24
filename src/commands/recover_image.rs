use std::env;
use anyhow::{anyhow, Result};
use chia::protocol::{Program};
use chia::traits::Streamable;
use clap::Args;
use dg_xch_clients::api::full_node::FullnodeAPI;
use dg_xch_core::blockchain::coin::Coin;
use dg_xch_core::blockchain::sized_bytes::{Bytes32, SizedBytes};
use tokio::fs;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use recovery_tools::{filter_collection_end, filter_collection_start, filter_png_end, filter_png_start, get_filename, is_png_end, is_png_start};
use crate::chia::client::get_chia_client;
use crate::chia::memo::parse_memos;

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
        let mut current_coin = client.get_coin_record_by_name(&coinidb32).await?.ok_or(anyhow!("No Coin Record found."))?;
        let mut puzz_solution = client.get_puzzle_and_solution(&coinidb32, current_coin.spent_block_index).await?;

        let cwd = env::current_dir()?;
        let outputdir = cwd.join("output-images");
        fs::create_dir_all(&outputdir).await?;

        let mut found_start = false;
        while current_coin.spent_block_index > 0 {
            let solution = puzz_solution.solution.clone();
            let puzzle = Program::from_bytes(&puzz_solution.puzzle_reveal.to_bytes())?;
            let solution_program = Program::from_bytes(&solution.to_bytes())?;
            let mut memo = parse_memos(solution_program, puzzle)?.unwrap();
            if !found_start && !is_png_start(&memo) {
                anyhow::bail!("Not the start of an image");
            }
            found_start = true;
            println!("Found start of image, reassembling...");

            // Check for the filename before we strip it out of the memo
            let file_name = get_filename(&memo);

            // Filter known prefixes and suffixes that might be in the data
            memo = filter_png_start(&memo);
            memo = filter_png_end(&memo);
            memo = filter_collection_start(&memo);
            memo = filter_collection_end(&memo);

            let output_file_name = outputdir.join(format!("{}.png", self.coin));
            let mut file = OpenOptions::new()
                .write(true)  // Open the file for writing
                .create(true) // Create the file if it does not exist
                .append(true) // Append to the file instead of truncating
                .open(&output_file_name).await?;
            file.write_all(&memo).await?;

            if is_png_end(&memo) {
                println!("Found end of image!");
                if let Some(filename) = file_name {
                    println!("Renaming to {}", filename);
                    fs::rename(&output_file_name, outputdir.join(filename)).await?;
                }
                return anyhow::Ok(());
            }

            let child_coin = Coin{
                parent_coin_info: current_coin.coin.coin_id().clone(),
                puzzle_hash: current_coin.coin.puzzle_hash.clone(),
                amount: current_coin.coin.amount.clone(),
            };
            current_coin = client.get_coin_record_by_name(&child_coin.name()).await?.ok_or(anyhow!("Unable to get child coin"))?;
            if current_coin.spent_block_index == 0 {
                anyhow::bail!("No more data available on chain, but did not reach end of the image!");
            }

            puzz_solution = client.get_puzzle_and_solution(&child_coin.name(), current_coin.spent_block_index).await?;
        }

        anyhow::Ok(())
    }
}