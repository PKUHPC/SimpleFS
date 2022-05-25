#[allow(non_camel_case_types)]
pub enum MessageType {
    MSG_MR,
    MSG_READY,
    MSG_DONE,
}
pub struct Message {
    pub mtype: MessageType,
    pub addr: u64,
    pub rkey: u32,
    pub data: u64,
}

unsafe impl Send for Message {}

pub struct ChunkTransferTask {
    pub chunk_id: Vec<u64>,
    pub metadata: ChunkMetadata,
    pub addr: u64,
}
#[derive(Clone, Debug)]
pub struct TransferMetadata {
    pub path: [u8; 256],
    pub chunk_start: u64,
    pub offset: u64,
    pub size: u64,
    pub path_len: usize,
}
impl TransferMetadata {
    pub fn default() -> TransferMetadata {
        TransferMetadata {
            path: [0; 256],
            chunk_start: 0,
            offset: 0,
            size: 0,
            path_len: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ChunkMetadata {
    pub path: String,
    pub chunk_start: u64,
    pub offset: u64,
    pub size: u64,
}
impl ChunkMetadata {
    pub fn default() -> ChunkMetadata {
        ChunkMetadata {
            path: "".to_string(),
            chunk_start: 0,
            offset: 0,
            size: 0,
        }
    }
}
