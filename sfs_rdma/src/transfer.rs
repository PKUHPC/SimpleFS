use rdma_sys::ibv_mr;



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
}

unsafe impl Send for Message {}

pub struct ChunkTransferTask {
    pub chunk_id: *mut u64,
    pub chunks: u64,
    pub addr: u64,
    pub size: u64,
}
pub struct SenderContext {
    pub chunk_id: *mut u64,
    pub chunks: u64,
    pub addr: u64,
    pub size: u64,

    pub buffer: *mut u8,
    pub buffer_mr: *mut ibv_mr,

    pub msg: *mut Message,
    pub msg_mr: *mut ibv_mr,

    pub peer_addr: u64,
    pub peer_rkey: u32,
}

pub struct ReceiverContext {
    pub buffer: *mut u8,
    pub buffer_mr: *mut ibv_mr,

    pub msg: *mut Message,
    pub msg_mr: *mut ibv_mr,
}

