use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct RDMAConfig{
    pub addr: String
}
pub const CHUNK_SIZE: u64 = 524288;
pub const DIRENT_BUF_SIZE: u64 = 8 * 1024 * 1024;
