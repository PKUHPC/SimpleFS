use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct WriteData<'a> {
    pub path: &'a str,
    pub offset: i64,
    pub chunk_id: u64,
    pub write_size: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReadData<'a> {
    pub path: &'a str,
    pub offset: i64,
    pub chunk_id: u64,
    pub read_size: u64,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ReadResult {
    pub nreads: u64,
    pub chunk_id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateData<'a> {
    pub path: &'a str,
    pub mode: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateMetadentryData<'a> {
    pub path: &'a str,
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
pub struct DecrData<'a> {
    pub path: &'a str,
    pub new_size: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TruncData<'a> {
    pub path: &'a str,
    pub new_size: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DirentData<'a> {
    pub path: &'a str,
    // RDMA buffer?
}
#[derive(Serialize, Deserialize, Debug)]
pub struct SerdeString<'a> {
    pub str: &'a str,
    // RDMA buffer?
}
