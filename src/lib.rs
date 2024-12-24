use std::str::from_utf8;
use chia::{
    protocol::{Bytes},
};

pub const PNG_START: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
const I_END_CHUNK: [u8; 12] = [0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82];
const START_COLLECTION: &[u8; 13] = b"CHIAGODSSTART";
const END_COLLECTION: &[u8; 11] = b"CHIAGODSEND";
const START_META: &[u8; 17] = b"CHIAGODSMETASTART";
const END_META: &[u8; 15] = b"CHIAGODSMETAEND";

fn bytes_contains(haystack: &[u8], needle: &[u8]) -> Option<(usize, usize)> {
    haystack.windows(needle.len())
        .position(|window| window == needle)
        .map(|start| (start, start + needle.len()))
}

pub fn is_meta(memo: &Bytes) -> bool {
    bytes_contains(&memo, START_META).is_some()
}

pub fn is_png_start(memo: &Bytes) -> bool {
    bytes_contains(&memo, &PNG_START[..]).is_some()
}

pub fn is_png_end(memo: &Bytes) -> bool {
    bytes_contains(&memo, &I_END_CHUNK[..]).is_some()
}

pub fn is_collection_start(memo: &Bytes) -> bool {
    bytes_contains(&memo, &START_COLLECTION[..]).is_some()
}

pub fn is_collection_end(memo: &Bytes) -> bool {
    bytes_contains(&memo, &END_COLLECTION[..]).is_some()
}

pub fn filter_png_start(memo: &Bytes) -> Bytes {
    // If we encounter PNG_START we should also strip everything else before it
    if let Some((start,_end)) = bytes_contains(&memo, &PNG_START[..]) {
        return Bytes::new(memo[start..].to_vec());
    }

    memo.clone()
}

pub fn filter_png_end(memo: &Bytes) -> Bytes {
    // If we encounter I_END_CHUNK we should also strip everything else after it
    if let Some((_start,end)) = bytes_contains(&memo, &I_END_CHUNK[..]) {
        return Bytes::new(memo[..end].to_vec());
    }

    memo.clone()
}

pub fn filter_collection_start(memo: &Bytes) -> Bytes {
    // If we encounter START_COLLECTION we should also strip everything else before it
    if let Some((_start,end)) = bytes_contains(&memo, &START_COLLECTION[..]) {
        return Bytes::new(memo[end..].to_vec());
    }

    memo.clone()
}

pub fn filter_collection_end(memo: &Bytes) -> Bytes {
    // If we encounter END_COLLECTION we should also strip everything else before it
    if let Some((start,_end)) = bytes_contains(&memo, &END_COLLECTION[..]) {
        return Bytes::new(memo[..start].to_vec());
    }

    memo.clone()
}

pub fn get_filename(memo: &Bytes) -> Option<String> {
    // if the filename exists, it exists after I_END_CHUNK
    // If the collection end marker also exists, it will be immediately after filename
    // First, we can just strip out the collection end
    let working_memo = filter_collection_end(memo);
    if let Some((_start,end)) = bytes_contains(&working_memo, &I_END_CHUNK[..]) {
        if let Ok(stringfile) = from_utf8(&working_memo[end..]) {
            return Some(String::from(stringfile));
        }
    }

    None
}
