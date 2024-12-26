use crate::chia::coins::conditions_for_puzz_solution;
use anyhow::Result;
use chia::protocol::{Bytes, Program};
use chia_wallet_sdk::Condition;

pub fn parse_memos(solution_program: Program, reveal_program: Program) -> Result<Option<Bytes>> {
    let conditions = conditions_for_puzz_solution(solution_program, reveal_program)?;
    parse_memos_from_conditions(conditions)
}

pub fn parse_memos_from_conditions(conditions: Vec<Condition>) -> Result<Option<Bytes>> {
    let create_coin = conditions
        .into_iter()
        .flat_map(Condition::into_create_coin)
        .next();
    let memos = create_coin.map(|create_coin| create_coin.memos);
    match memos {
        Some(vec) if !vec.is_empty() => Ok(Some(vec.into_iter().next().unwrap())), // First item
        Some(_) => Ok(None), // Vec exists but is empty
        None => Ok(None),    // No memos (Option is None)
    }
}
