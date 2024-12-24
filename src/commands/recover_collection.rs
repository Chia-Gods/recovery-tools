use crate::chia::client::get_chia_client;
use crate::chia::image::get_image;
use crate::chia::memo::parse_memos;
use anyhow::{anyhow, Result};
use chia::protocol::Program;
use chia::traits::Streamable;
use clap::Args;
use dg_xch_clients::api::full_node::FullnodeAPI;
use dg_xch_core::blockchain::coin::Coin;
use dg_xch_core::blockchain::sized_bytes::{Bytes32, SizedBytes};
use recovery_tools::{is_collection_end, is_collection_start};
use std::env;
use tokio::fs;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

#[derive(Args)]
#[command(about = "Recover a collection of images")]
pub struct RecoverCollection {
    /// The coin ID at the start of the collection
    #[arg(short, long)]
    coin: String,
}

impl RecoverCollection {
    pub async fn execute(&self) -> Result<()> {
        println!("Recovering collection from coin: {}", self.coin);
        let client = get_chia_client(8555);

        let cwd = env::current_dir()?;
        let outputdir = cwd.join("output-images");
        fs::create_dir_all(&outputdir).await?;

        let coinid = hex::decode(&self.coin)?;
        let coinidb32 = Bytes32::new(&coinid);
        let mut current_coin = client
            .get_coin_record_by_name(&coinidb32)
            .await?
            .ok_or(anyhow!("No Coin Record found."))?;
        let mut puzz_solution = client
            .get_puzzle_and_solution(&coinidb32, current_coin.spent_block_index)
            .await?;

        let mut found_collection_start = false;
        let mut current_image_counter = 1;

        while current_coin.spent_block_index > 0 {
            let solution = puzz_solution.solution.clone();
            let puzzle = Program::from_bytes(&puzz_solution.puzzle_reveal.to_bytes())?;
            let solution_program = Program::from_bytes(&solution.to_bytes())?;
            let memo = parse_memos(solution_program, puzzle)?.unwrap();

            if !found_collection_start && !is_collection_start(&memo) {
                anyhow::bail!("Not the start of a collection");
            }
            found_collection_start = true;

            let image_result = get_image(&client, &current_coin, &puzz_solution).await?;
            let final_filename = image_result
                .filename
                .unwrap_or(format!("{}-{}.png", current_image_counter, self.coin));
            let output_file_name = outputdir.join(&final_filename);
            let mut file = OpenOptions::new()
                .write(true) // Open the file for writing
                .create(true) // Create the file if it does not exist
                .truncate(true) // Overwrite any existing content
                .open(&output_file_name)
                .await?;
            file.write_all(&image_result.data).await?;
            println!("Wrote {}", &final_filename);

            if is_collection_end(&image_result.last_memo) {
                println!("Reached end of collection!");
                return Ok(());
            }
            let child_coin = Coin {
                parent_coin_info: image_result.last_coin.coin.coin_id().clone(),
                puzzle_hash: image_result.last_coin.coin.puzzle_hash.clone(),
                amount: image_result.last_coin.coin.amount.clone(),
            };
            current_coin = client
                .get_coin_record_by_name(&child_coin.name())
                .await?
                .unwrap();
            if current_coin.spent_block_index == 0 {
                println!("No more data available on chain, but did not reach end of collection!");
                return Ok(());
            }
            puzz_solution = client
                .get_puzzle_and_solution(&child_coin.name(), current_coin.spent_block_index)
                .await?;
            current_image_counter += 1;
        }

        anyhow::Ok(())
    }
}
