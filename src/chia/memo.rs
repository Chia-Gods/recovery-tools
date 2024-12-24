use anyhow::anyhow;
use chia::clvm_traits::{FromClvm, ToClvm};
use chia::protocol::{Bytes, Program};
use chia_wallet_sdk::{run_puzzle, Condition};
use clvmr::Allocator;

pub fn parse_memos(solution_program: Program, reveal_program: Program) -> anyhow::Result<Option<Bytes>> {
    let mut allocator = Allocator::new();
    let puzzle = reveal_program.to_clvm(&mut allocator)?;
    let solution = solution_program.to_clvm(&mut allocator)?;
    let output = run_puzzle(&mut allocator, puzzle, solution)?;
    let conditions = Vec::<Condition>::from_clvm(&allocator, output)?;
    let create_coin = conditions
        .into_iter()
        .flat_map(Condition::into_create_coin)
        .next();
    let memos = create_coin.map(|create_coin| create_coin.memos);
    match memos {
        Some(vec) if !vec.is_empty() => Ok(Some(vec.into_iter().next().unwrap())), // First item
        Some(_) => Err(anyhow!("Vector is empty")), // Vec exists but is empty
        None => Ok(None),                           // No memos (Option is None)
    }
}