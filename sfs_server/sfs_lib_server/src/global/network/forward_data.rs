use std::collections::HashMap;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct WriteData{
    pub path: String,
    pub offset: i64,
    pub host_id: u64,
    pub host_size: u64,
    pub chunk_n: u64,
    pub chunk_start: u64,
    pub chunk_end: u64,
    pub total_chunk_size: u64,
    pub buffers: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReadData{
    pub path: String,
    pub offset: i64,
    pub host_id: u64,
    pub host_size: u64,
    pub chunk_n: u64,
    pub chunk_start: u64,
    pub chunk_end: u64,
    pub total_chunk_size: u64
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ReadResult{
    pub nreads: u64,
    pub data: HashMap<u64, String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateData{
    pub path: String,
    pub mode: u32
}