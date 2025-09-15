use anyhow::Result;
use chia::protocol::Bytes;
use dg_xch_core::blockchain::sized_bytes::{Bytes32, SizedBytes};
use flate2::read::GzDecoder;
use std::io::Read;
use std::str::from_utf8;

pub const PNG_START: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
const I_END_CHUNK: [u8; 12] = [
    0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
];
const START_COLLECTION: &[u8; 13] = b"CHIAGODSSTART";
const END_COLLECTION: &[u8; 11] = b"CHIAGODSEND";
const START_META: &[u8; 17] = b"CHIAGODSMETASTART";
const END_META: &[u8; 15] = b"CHIAGODSMETAEND";

fn bytes_contains(haystack: &[u8], needle: &[u8]) -> Option<(usize, usize)> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|start| (start, start + needle.len()))
}

#[must_use]
pub fn is_meta(memo: &Bytes) -> bool {
    bytes_contains(memo, START_META).is_some()
}

#[must_use]
pub fn is_png_start(memo: &Bytes) -> bool {
    bytes_contains(memo, &PNG_START[..]).is_some()
}

#[must_use]
pub fn is_png_end(memo: &Bytes) -> bool {
    bytes_contains(memo, &I_END_CHUNK[..]).is_some()
}

#[must_use]
pub fn is_collection_start(memo: &Bytes) -> bool {
    bytes_contains(memo, &START_COLLECTION[..]).is_some()
}

#[must_use]
pub fn is_collection_end(memo: &Bytes) -> bool {
    bytes_contains(memo, &END_COLLECTION[..]).is_some()
}

#[must_use]
pub fn filter_png_start(memo: &Bytes) -> Bytes {
    // If we encounter PNG_START we should also strip everything else before it
    if let Some((start, _end)) = bytes_contains(memo, &PNG_START[..]) {
        return Bytes::new(memo[start..].to_vec());
    }

    memo.clone()
}

#[must_use]
pub fn filter_png_end(memo: &Bytes) -> Bytes {
    // If we encounter I_END_CHUNK we should also strip everything else after it
    if let Some((_start, end)) = bytes_contains(memo, &I_END_CHUNK[..]) {
        return Bytes::new(memo[..end].to_vec());
    }

    memo.clone()
}

#[must_use]
pub fn filter_collection_start(memo: &Bytes) -> Bytes {
    // If we encounter START_COLLECTION we should also strip everything else before it
    if let Some((_start, end)) = bytes_contains(memo, &START_COLLECTION[..]) {
        return Bytes::new(memo[end..].to_vec());
    }

    memo.clone()
}

#[must_use]
pub fn filter_collection_end(memo: &Bytes) -> Bytes {
    // If we encounter END_COLLECTION we should also strip everything else before it
    if let Some((start, _end)) = bytes_contains(memo, &END_COLLECTION[..]) {
        return Bytes::new(memo[..start].to_vec());
    }

    memo.clone()
}

/// Strips everything before and including the `START_META` marker from a memo.
///
/// This function searches the provided [`Bytes`] buffer for the byte sequence
/// defined by [`START_META`]. If the marker is found, the returned [`Bytes`]
/// contains only the data **after** the marker. If the marker is not found,
/// the original buffer is returned unchanged.
///
/// # Must Use
///
/// The returned [`Bytes`] must be used; calling this function without
/// inspecting its return value will have no effect.
#[must_use]
pub fn filter_meta_start(memo: &Bytes) -> Bytes {
    // If we encounter START_META we should also strip everything else before it
    if let Some((_start, end)) = bytes_contains(memo, &START_META[..]) {
        return Bytes::new(memo[end..].to_vec());
    }

    memo.clone()
}

/// Strips everything after (and including) the `END_META` marker from a memo.
///
/// This function searches the provided [`Bytes`] buffer for the byte sequence
/// defined by [`END_META`]. If the marker is found, the returned [`Bytes`]
/// contains only the data **before** the marker. If the marker is not found,
/// the original buffer is returned unchanged.
///
/// # Must Use
///
/// The returned [`Bytes`] must be used; calling this function without
/// inspecting its return value will have no effect.
#[must_use]
pub fn filter_meta_end(memo: &Bytes) -> Bytes {
    // If we encounter END_META we should also strip everything else before it
    if let Some((start, _end)) = bytes_contains(memo, &END_META[..]) {
        return Bytes::new(memo[..start].to_vec());
    }

    memo.clone()
}

#[must_use]
pub fn get_filename(memo: &Bytes) -> Option<String> {
    // if the filename exists, it exists after I_END_CHUNK
    // If the collection end marker also exists, it will be immediately after filename
    // First, we can just strip out the collection end
    let working_memo = filter_collection_end(memo);
    if let Some((_start, end)) = bytes_contains(&working_memo, &I_END_CHUNK[..]) {
        if let Ok(stringfile) = from_utf8(&working_memo[end..]) {
            return Some(String::from(stringfile));
        }
    }

    None
}

/// Decompresses a gzip-compressed byte slice into raw bytes.
///
/// # Errors
///
/// Returns an [`std::io::Error`] if the input is not valid gzip data
/// or if an error occurs during decompression.
pub fn decompress_gzip_to_bytes(compressed: &Bytes) -> Result<Bytes, std::io::Error> {
    // Create a GzDecoder to decompress the data
    let mut decoder = GzDecoder::new(&compressed[..]);

    // Buffer to hold the decompressed data
    let mut decompressed_data = Vec::new();

    // Decompress the data
    decoder.read_to_end(&mut decompressed_data)?;

    // Convert the uncompressed data into Bytes and return it
    Ok(Bytes::from(decompressed_data))
}

/// # Errors
///
/// Will return `Err` if hex decode fails
pub fn coin_id_from_string(coin_id_str: &str) -> Result<Bytes32> {
    let stripped_coin_id_str = if coin_id_str.to_lowercase().starts_with("0x") {
        &coin_id_str[2..]
    } else {
        coin_id_str
    };
    let coinid = hex::decode(stripped_coin_id_str)?;
    Ok(Bytes32::new(&coinid))
}
