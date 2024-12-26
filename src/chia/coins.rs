use anyhow::Result;
use chia::clvm_traits::{FromClvm, ToClvm};
use chia::protocol::Program;
use chia::traits::Streamable;
use chia_wallet_sdk::{run_puzzle, Condition};
use clvmr::Allocator;
use dg_xch_clients::api::full_node::FullnodeAPI;
use dg_xch_clients::rpc::full_node::FullnodeClient;
use dg_xch_core::blockchain::coin_record::CoinRecord;

pub async fn conditions_for_coin(
    client: &FullnodeClient,
    coin: &CoinRecord,
) -> Result<Vec<Condition>> {
    let puzz_solution = client
        .get_puzzle_and_solution(&coin.coin.name(), coin.spent_block_index)
        .await?;

    let solution = puzz_solution.solution.clone();
    let puzzle_program = Program::from_bytes(&puzz_solution.puzzle_reveal.to_bytes())?;
    let solution_program = Program::from_bytes(&solution.to_bytes())?;

    conditions_for_puzz_solution(solution_program, puzzle_program)
}

pub fn conditions_for_puzz_solution(
    solution_program: Program,
    reveal_program: Program,
) -> Result<Vec<Condition>> {
    let mut allocator = Allocator::new();
    let puzzle = reveal_program.to_clvm(&mut allocator)?;
    let solution = solution_program.to_clvm(&mut allocator)?;
    let output = run_puzzle(&mut allocator, puzzle, solution)?;
    let conditions = Vec::<Condition>::from_clvm(&allocator, output)?;
    Ok(conditions)
}
