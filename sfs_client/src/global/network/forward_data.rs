use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct WriteData {
    pub path: String,
    pub offset: i64,
    pub host_id: u64,
    pub host_size: u64,
    pub chunk_n: u64,
    pub chunk_start: u64,
    pub chunk_end: u64,
    pub total_chunk_size: u64,
    pub buffers: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReadData {
    pub path: String,
    pub offset: i64,
    pub host_id: u64,
    pub host_size: u64,
    pub chunk_n: u64,
    pub chunk_start: u64,
    pub chunk_end: u64,
    pub total_chunk_size: u64,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ReadResult {
    pub nreads: u64,
    pub data: HashMap<u64, String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateData {
    pub path: String,
    pub mode: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateMetadentryData {
    pub path: String,
    pub size: u64,
    pub offset: i64,
    pub append: bool,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ChunkStat {
    pub chunk_size: u64,
    pub chunk_total: u64,
    pub chunk_free: u64,
}
impl ChunkStat {
    pub fn new() -> ChunkStat {
        ChunkStat {
            chunk_size: 0,
            chunk_total: 0,
            chunk_free: 0,
        }
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub struct DecrData {
    pub path: String,
    pub new_size: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TruncData {
    pub path: String,
    pub new_size: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DirentData {
    pub path: String,
    // RDMA buffer?
}
#[derive(Serialize, Deserialize, Debug)]
pub struct SerdeString {
    pub str: String,
    // RDMA buffer?
}
