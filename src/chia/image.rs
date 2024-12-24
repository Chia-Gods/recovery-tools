use anyhow::{anyhow, Result, Ok};
use chia::protocol::{Program};
use chia::traits::Streamable;
use dg_xch_clients::api::full_node::FullnodeAPI;
use dg_xch_clients::rpc::full_node::FullnodeClient;
use dg_xch_core::blockchain::coin::Coin;
use dg_xch_core::blockchain::coin_record::CoinRecord;
use dg_xch_core::blockchain::coin_spend::CoinSpend;
use recovery_tools::{filter_collection_end, filter_collection_start, filter_png_end, filter_png_start, get_filename, is_png_end, is_png_start};
use crate::chia::memo::parse_memos;

pub async fn get_image(client: FullnodeClient, initial_coin: CoinRecord, initial_puzzle_solution: CoinSpend) -> Result<(Vec<u8>, Option<String>)> {
    let mut current_coin = initial_coin.clone();
    let mut puzz_solution = initial_puzzle_solution.clone();

    let mut final_image: Vec<u8> = Vec::new();

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

        final_image.extend(memo.as_ref());

        if is_png_end(&memo) {
            println!("Found end of image!");
            return Ok((final_image, file_name));
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

    anyhow::bail!("No image found");
}