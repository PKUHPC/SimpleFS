use std::ptr::null_mut;

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
    pub chunk_id: Vec<u64>,
    pub chunk_start: u64,
    pub offset: u64,
    pub addr: u64,
    pub size: u64,
}
pub struct SenderContext {
    pub chunk_id: Vec<u64>,
    pub chunk_start: u64,
    pub offset: u64,
    pub addr: u64,
    pub size: u64,

    pub buffer: *mut u8,
    pub buffer_mr: *mut ibv_mr,

    pub msg: *mut Message,
    pub msg_mr: *mut ibv_mr,

    pub peer_addr: u64,
    pub peer_rkey: u32,
}
impl SenderContext{
    pub fn new() -> Self{
        SenderContext{
            chunk_id: Vec::new(),
            chunk_start: 0,
            offset: 0,
            addr: 0,
            size: 0,
            buffer: null_mut(),
            buffer_mr: null_mut(),
            msg: null_mut(),
            msg_mr: null_mut(),
            peer_addr: 0,
            peer_rkey: 0,
        }
    }
}

pub struct ReceiverContext {
    pub buffer: *mut u8,
    pub buffer_mr: *mut ibv_mr,

    pub msg: *mut Message,
    pub msg_mr: *mut ibv_mr,
}

