use std::{ptr::null_mut, ffi::CStr};

use libc::c_void;
use rdma_sys::{ibv_wc, ibv_pd, ibv_comp_channel, ibv_context, ibv_cq, ibv_get_cq_event, ibv_ack_cq_events, ibv_req_notify_cq, ibv_poll_cq, ibv_wc_status::IBV_WC_SUCCESS, ibv_wc_status_str};

use crate::{transfer::ChunkTransferTask, chunk_operation::ChunkOp};

pub struct CQPoller{
    pub comp_channel: u64,
    pub pd: u64,
    pub on_completion: fn(*mut ibv_wc, *mut ibv_pd, &ChunkOp) -> Result<i64, i32>,
    pub op: ChunkOp
}
impl CQPoller{
    pub fn new(comp_channel: *mut ibv_comp_channel, pd: *mut ibv_pd, func: fn(*mut ibv_wc, *mut ibv_pd, op: &ChunkOp) -> Result<i64, i32>, op: ChunkOp) -> Self{
        CQPoller { comp_channel: comp_channel as u64, pd: pd as u64, on_completion: func, op}
    }
    pub fn poll(&self) -> Result<i64, i32>{
        unsafe{
            let comp_channel = self.comp_channel as *mut ibv_comp_channel;
            let pd = self.pd as *mut ibv_pd;
            let mut cq: *mut ibv_cq = null_mut();
            let mut wc: ibv_wc = std::mem::zeroed();
            let mut context: *mut c_void = null_mut();
            let mut result = 0;
            loop{
                assert_eq!(ibv_get_cq_event(comp_channel, &mut cq, &mut context), 0);
                ibv_ack_cq_events(cq, 1);
                assert_eq!(ibv_req_notify_cq(cq, 0), 0);

                while ibv_poll_cq(cq, 1, &mut wc) != 0{
                    if wc.status != IBV_WC_SUCCESS{
                        println!(
                            "work completion has error status '{}'",
                            CStr::from_ptr(ibv_wc_status_str(wc.status))
                                .to_string_lossy()
                                .into_owned()
                        );
                        continue;
                    }
                    let ret = (self.on_completion)(&mut wc, pd, &self.op);
                    if let Err(e) = ret {
                        return Err(e);
                    }
                    let ok = ret.unwrap();
                    if ok < 0{
                        return Ok(result);
                    }
                    result += ok;
                }      
            }

        }
    }
}
pub struct RDMA {
    pub ctx: u64,
    pub pd: u64,
    pub cq: u64,
    pub comp_channel: u64,
    //pub poll_handle: Option<JoinHandle<()>>,
}
impl RDMA {
    pub fn ctx(&self) -> *mut ibv_context {
        self.ctx as *mut ibv_context
    }
    pub fn pd(&self) -> *mut ibv_pd {
        self.pd as *mut ibv_pd
    }
    pub fn cq(&self) -> *mut ibv_cq {
        self.cq as *mut ibv_cq
    }
    pub fn comp_channel(&self) -> *mut ibv_comp_channel {
        self.comp_channel as *mut ibv_comp_channel
    }
    pub fn sender_client(addr: String, port: u16, task: ChunkTransferTask, op: ChunkOp){
        crate::sc_rs::sender_client::sender_client(addr, port, task, op);
    }
    pub fn recver_server(addr: String, port: u16, op: ChunkOp) -> Result<i64, i32> {
        crate::sc_rs::receiver_server::recver_server(addr, port, op)
    }
    pub fn sender_server(addr: String, port: u16, task: ChunkTransferTask, op: ChunkOp){
        crate::rc_ss::sender_server::sender_server(addr, port, task, op);
    }
    pub fn recver_client(addr: String, port: u16, op: ChunkOp) -> Result<i64, i32> {
        crate::rc_ss::receiver_client::recver_client(addr, port, op)
    }

}