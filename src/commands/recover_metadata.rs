use crate::chia::client::get_chia_client;
use crate::chia::memo::parse_memos;
use anyhow::anyhow;
use base64::{engine::general_purpose, Engine};
use chia::protocol::Program;
use chia::traits::Streamable;
use clap::Args;
use dg_xch_clients::api::full_node::FullnodeAPI;
use recovery_tools::{
    coin_id_from_string, decompress_gzip_to_bytes, filter_meta_end, filter_meta_start, is_meta,
};
use std::env;
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[derive(Args)]
#[command(about = "Recover metadata for the collection")]
pub struct RecoverMetadata {
    /// The coin ID that contains the metadata
    #[arg(short, long)]
    coin: String,
}

impl RecoverMetadata {
    pub async fn execute(&self, port: u16) -> anyhow::Result<()> {
        println!(
            "Recovering metadata for collection from coin: {}",
            self.coin
        );
        let client = get_chia_client(port);

        let coinid = coin_id_from_string(&self.coin)?;
        let current_coin = client
            .get_coin_record_by_name(&coinid)
            .await?
            .ok_or(anyhow!("No Coin Record found."))?;
        let puzz_solution = client
            .get_puzzle_and_solution(&coinid, current_coin.spent_block_index)
            .await?;

        let solution = puzz_solution.solution.clone();
        let puzzle = Program::from_bytes(&puzz_solution.puzzle_reveal.to_bytes())?;
        let solution_program = Program::from_bytes(&solution.to_bytes())?;
        let mut memo = parse_memos(solution_program, puzzle)?.unwrap();

        if !is_meta(&memo) {
            anyhow::bail!("Not a metadata coin")
        }

        let cwd = env::current_dir()?;
        let outputdir = cwd.join("output-metadata");
        fs::create_dir_all(&outputdir).await?;

        // Remove start and end meta markers
        memo = filter_meta_start(&memo);
        memo = filter_meta_end(&memo);

        // Decompress
        let decompressed_data = decompress_gzip_to_bytes(&memo)?;

        // Parse the JSON data as an array of RawMessage (serde_json::Value)
        let all_meta: Vec<String> = serde_json::from_slice(&decompressed_data)?;

        // Iterate over each item in the array and write to a separate JSON file
        for (index, item) in all_meta.iter().enumerate() {
            let output_file = outputdir.join(format!("metadata_{:03}.json", index + 1));
            let mut file = File::create(output_file).await?;

            // Check if the item is a string
            //if let Some(base64_string) = item.as_str() {
            let decoded_bytes = general_purpose::STANDARD.decode(item)?;

            // Write the decoded bytes to a file
            file.write_all(&decoded_bytes).await?;
        }

        anyhow::Ok(())
    }
}
