use crate::chia::client::get_chia_client;
use crate::chia::coins::conditions_for_coin;
use crate::chia::memo::parse_memos_from_conditions;
use anyhow::{anyhow, Result};
use chia::sha2::Sha256;
use chia_wallet_sdk::Condition;
use clap::Args;
use dg_xch_clients::api::full_node::FullnodeAPI;
use dg_xch_clients::rpc::full_node::FullnodeClient;
use dg_xch_core::blockchain::coin_record::CoinRecord;
use dg_xch_core::blockchain::sized_bytes::{Bytes32, SizedBytes};
use recovery_tools::{is_collection_end, is_collection_start, is_meta, is_png_start};

#[derive(Args)]
#[command(about = "Finds key coins starting from an NFT in the collection")]
pub struct LocateNFTData {
    /// The NFT ID to start searching from
    #[arg(short, long)]
    nft_id: String,
}

impl LocateNFTData {
    pub async fn execute(&self) -> Result<()> {
        println!("Locating NFT data for: {}", self.nft_id);
        let client = get_chia_client(8555);

        let (_hrp, launcher_id) = bech32::decode(&self.nft_id)?;
        let coinid = Bytes32::new(&launcher_id[..]);
        let launcher_coin = client
            .get_coin_record_by_name(&coinid)
            .await?
            .ok_or(anyhow!("Launcher Coin Record found."))?;

        let eph_coin = client
            .get_coin_record_by_name(&launcher_coin.coin.parent_coin_info)
            .await?
            .ok_or(anyhow!("Ephemeral Coin Record found."))?;

        let direct_parent = client
            .get_coin_record_by_name(&eph_coin.coin.parent_coin_info)
            .await?
            .ok_or(anyhow!("Parent of ephemeral not found"))?;

        // Now, the conditions on the spend of the direct parent should enable us to find the other parent of the ephemeral coin
        // linked by CREATE_COIN_ANNOUNCEMENT/ASSERT_COIN_ANNOUNCEMENT
        let conditions = conditions_for_coin(&client, &direct_parent).await?;

        // Now, we need to find the CREATE_COIN_ANNOUNCEMENT
        let create_coin_announcements = conditions
            .into_iter()
            .flat_map(Condition::into_create_coin_announcement);

        let mut input_coins: Vec<CoinRecord> = vec![];

        for announcement in create_coin_announcements {
            let mut hasher = Sha256::new();
            hasher.update(&eph_coin.coin.parent_coin_info);
            hasher.update(&announcement.message);
            let message = hasher.finalize().to_vec();

            // now we have to find coins with ASSERT_COIN_ANNOUNCEMENT of `message`
            // Get all the coin creations in the block, and find the matching assert
            let block = client
                .get_block_record_by_height(eph_coin.confirmed_block_index)
                .await?;
            let (_additions, removals) = client
                .get_additions_and_removals(&block.header_hash)
                .await?;
            for removal in removals {
                let conditions = conditions_for_coin(&client, &removal).await?;
                let assert_coin_announcements = conditions
                    .into_iter()
                    .flat_map(Condition::into_assert_coin_announcement);
                for assert_coin_announcement in assert_coin_announcements {
                    if assert_coin_announcement.announcement_id[..] == message {
                        input_coins.push(removal.clone())
                    }
                }
            }
        }

        if input_coins.len() != 1 {
            anyhow::bail!("Unexpected number of input coins found");
        }

        // At this point, we found the parent of the NFT that directly leads back to the metadata (without tracing asserts)
        let mut current_coin = input_coins.pop().ok_or(anyhow!("Missing input coin"))?;

        let mut found_gap = false; // 🏴‍☠️ 💰 🗺️ 💎 🏆 🎁 📜
        let mut found_meta = false;
        let mut found_collection_end = false;

        while current_coin.spent_block_index > 0 {
            // First, we don't care about ephemeral coins
            if current_coin.spent_block_index == current_coin.confirmed_block_index {
                current_coin = advance_parent(&client, &current_coin).await?;
                continue;
            }
            // Keep going until we find the first non-hint memo
            // First memo is a gap between meta and minting
            // Then meta coin
            // Then the "End Collection" image
            // <image data in between>
            // Then the "Start Collection" image

            let conditions = conditions_for_coin(&client, &current_coin).await?;
            let memo_opt = parse_memos_from_conditions(conditions)?;
            if memo_opt.is_none() {
                if found_gap {
                    // If we found the gap and have no memo before the collection is done, that is not expected
                    anyhow::bail!("Unexpected spend with no memo in the collection data")
                }
                current_coin = advance_parent(&client, &current_coin).await?;
                continue;
            }
            let memo = memo_opt.unwrap();

            if !found_gap {
                if memo.len() != 32 {
                    found_gap = true;
                    println!("Found a spend before the NFT mints with a memo that isn't the metadata, but something seems to be here. Continuing to parent coin...");
                }
                current_coin = advance_parent(&client, &current_coin).await?;
                continue;
            }

            if !found_meta {
                if !is_meta(&memo) {
                    println!("{}", memo);
                    anyhow::bail!("Did not find the metadata at the expected location");
                }
                found_meta = true;
                println!("Found metadata at coin: {}", current_coin.coin.name());
                current_coin = advance_parent(&client, &current_coin).await?;
                continue;
            }
            // Was checked for none earlier
            if !found_collection_end {
                if !is_collection_end(&memo) {
                    anyhow::bail!(
                        "Did not find the end of the collection at the expected location"
                    );
                }
                found_collection_end = true;
                println!("End of collection at coin: {}", current_coin.coin.name());
                current_coin = advance_parent(&client, &current_coin).await?;
                continue;
            }

            if is_png_start(&memo) {
                println!("Found an image coin at: {}", current_coin.coin.name());
            }

            if is_collection_start(&memo) {
                println!("Found collection start at: {}", current_coin.coin.name());
                return anyhow::Ok(());
            }

            current_coin = advance_parent(&client, &current_coin).await?;
            continue;
        }

        anyhow::Ok(())
    }
}

async fn advance_parent(client: &FullnodeClient, coin: &CoinRecord) -> Result<CoinRecord> {
    client
        .get_coin_record_by_name(&coin.coin.parent_coin_info)
        .await?
        .ok_or(anyhow!("No coin found"))
}
