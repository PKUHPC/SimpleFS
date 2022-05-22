use std::{
    ffi::CStr,
    ptr::{null, null_mut},
};

use errno::errno;
use libc::{
    addrinfo, c_char, c_void, freeaddrinfo, getaddrinfo, malloc, memcpy, sockaddr,
    sockaddr_in,
};
use rdma::RDMA;
use rdma_sys::{
    ibv_alloc_pd, ibv_context,
    ibv_create_comp_channel, ibv_create_cq, 
    ibv_qp_init_attr, ibv_qp_type::IBV_QPT_RC, ibv_req_notify_cq, 
    rdma_ack_cm_event, rdma_cm_event,
    rdma_cm_event_type, rdma_cm_id, rdma_conn_param, rdma_create_qp, 
    rdma_event_channel, rdma_event_str, rdma_get_cm_event,
};
//pub mod function;
//pub mod memory_region;
pub mod chunk_operation;
pub mod rdma;
pub mod transfer;
mod sc_rs;
mod rc_ss;
mod send;
mod recv;
//pub mod work_request;

pub static CQ_CAPACITY: i32 = 16;
pub static MAX_SGE: u32 = 2;
pub static MAX_WR: u32 = 8;
#[allow(non_camel_case_types)]
pub enum stag {
    LocalStag { key: u32 },
    RemoteStag { key: u32 },
}
#[allow(non_camel_case_types)]
pub struct rdma_buffer_attr {
    pub address: u64,
    pub length: usize,
    pub stag: stag,
}
impl rdma_buffer_attr {
    pub fn as_mut_ptr(&mut self) -> *mut rdma_buffer_attr {
        self as *mut rdma_buffer_attr
    }
}
pub fn get_addr(addr: &String, port: u16, sockaddr: *mut sockaddr) -> i32 {
    unsafe {
        let fixed_addr = addr.clone() + "\0";
        let fixed_port = port.to_string() + "\0";
        let mut res: *mut addrinfo = null_mut();
        let ret = getaddrinfo(
            fixed_addr.as_ptr() as *const c_char,
            fixed_port.as_ptr() as *const c_char,
            null() as *const addrinfo,
            (&mut res) as *mut *mut addrinfo,
        );
        if ret != 0 {
            println!("getaddrinfo failed: invalid ip address");
            return ret;
        }
        memcpy(
            sockaddr as *mut c_void,
            (*res).ai_addr as *const c_void,
            std::mem::size_of::<sockaddr_in>(),
        );
        freeaddrinfo(res);
        return ret;
    }
}
pub fn process_rdma_cm_event(
    echannel: *mut rdma_event_channel,
    expected_event: rdma_cm_event_type::Type,
    cm_event: *mut *mut rdma_cm_event,
) -> i32 {
    unsafe {
        let mut ret;
        ret = rdma_get_cm_event(echannel, cm_event);
        if ret != 0 {
            println!("failed to get CM event: {}", errno().0);
            return -errno().0;
        }
        ret = (**cm_event).status;
        if ret != 0 {
            println!("CM event has non zero status: {}", ret);
            rdma_ack_cm_event(*cm_event);
            return ret;
        }
        if (**cm_event).event != expected_event {
            println!(
                "unexpected event received: {} [exected: {}]",
                CStr::from_ptr(rdma_event_str((**cm_event).event))
                    .to_string_lossy()
                    .into_owned(),
                CStr::from_ptr(rdma_event_str(expected_event))
                    .to_string_lossy()
                    .into_owned()
            );
            rdma_ack_cm_event(*cm_event);
            return -1;
        }
        return ret;
    }
}
pub fn build_params(params: &mut rdma_conn_param) {
    params.initiator_depth = 3;
    params.responder_resources = 3;
    params.rnr_retry_count = 3;
}
pub fn build_qp_attr(attr: &mut ibv_qp_init_attr, s_ctx: *mut RDMA) {
    unsafe {
        attr.send_cq = (*s_ctx).cq();
        attr.recv_cq = (*s_ctx).cq();
        attr.qp_type = IBV_QPT_RC;

        attr.cap.max_send_wr = MAX_WR;
        attr.cap.max_recv_wr = MAX_WR;
        attr.cap.max_send_sge = MAX_SGE;
        attr.cap.max_send_sge = MAX_SGE;
    }
}

pub fn build_context(verbs: *mut ibv_context, s_ctx: *mut *mut RDMA) {
    unsafe {
        (*s_ctx) = malloc(std::mem::size_of::<RDMA>()).cast();
        (*(*s_ctx)).ctx = verbs as u64;
        (*(*s_ctx)).pd = ibv_alloc_pd(verbs) as u64;
        (*(*s_ctx)).comp_channel = ibv_create_comp_channel(verbs) as u64;
        (*(*s_ctx)).cq = ibv_create_cq(
            verbs,
            CQ_CAPACITY,
            null_mut(),
            (*(*s_ctx)).comp_channel(),
            0,
        ) as u64;
        ibv_req_notify_cq((*(*s_ctx)).cq(), 0);
    }
}

pub fn build_connection(id: *mut rdma_cm_id, s_ctx: *mut *mut RDMA) {
    unsafe {
        let mut attr: ibv_qp_init_attr = std::mem::zeroed();
        build_context((*id).verbs, s_ctx);
        build_qp_attr(&mut attr, *s_ctx);

        rdma_create_qp(id, (*(*s_ctx)).pd(), &mut attr);
    }
}
static CHUNK_SIZE: u64 = sfs_global::global::network::config::CHUNK_SIZE;

#[cfg(test)]
mod tests {

    use crate::{rdma::RDMA, transfer::ChunkTransferTask, chunk_operation::ChunkOp, CHUNK_SIZE};

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
    pub fn show(
        _file_path: &String,
        _chunk_id: u64,
        buf: *mut u8,
        size: u64,
        _offset: u64,
    ) -> Result<i64, i32>{
        let str = unsafe{std::ffi::CStr::from_ptr(buf.cast()).to_string_lossy().into_owned()};
        println!("received {}", str);
        return Ok(size as i64);
    }
    #[test] 
    fn test_recv_server() {
        let op = ChunkOp{
            path: "".to_string(),
            offset: 0,
            chunk_start: 0,
            size: "hello, here is RDMA data transfer test!\0".len() as u64,
            op: show,
        };
        if let Ok(data) = RDMA::recver_server(&"192.168.230.142".to_string(), 20432, op){
            println!("receiver result: {}", data);
        }
    }
    #[test] 
    fn test_send_client() {
        let data = "hello, here is RDMA data transfer test!\0";
        let chunks = (data.len() as u64 + CHUNK_SIZE - 1) / CHUNK_SIZE;
        let chunk_ids: Vec<u64> = (0..chunks).into_iter().collect();
        let task = ChunkTransferTask {
            chunk_id: chunk_ids,
            chunk_start: 0,
            offset: 0,
            addr: data.as_ptr() as u64,
            size: data.len() as u64,
        };
        let op = ChunkOp{
            path: "".to_string(),
            offset: 0,
            chunk_start: 0,
            size: "hello, here is RDMA data transfer test!\0".len() as u64,
            op: show,
        };
        RDMA::sender_client(&"192.168.230.142".to_string(), 20432, task, op);
    }
    
    #[test] 
    fn test_send_server() {
        let data = "hello, here is RDMA data transfer test!\0";
        let chunks = (data.len() as u64 + CHUNK_SIZE - 1) / CHUNK_SIZE;
        let chunk_ids: Vec<u64> = (0..chunks).into_iter().collect();
        let task = ChunkTransferTask {
            chunk_id: chunk_ids,
            offset: 0,
            addr: data.as_ptr() as u64,
            size: data.len() as u64,
            chunk_start: 0,
        };
        let op = ChunkOp{
            path: "".to_string(),
            offset: 0,
            chunk_start: 0,
            size: "hello, here is RDMA data transfer test!\0".len() as u64,
            op: show,
        };
        RDMA::sender_server(&"192.168.230.142".to_string(), 20532, task, op).join().unwrap();
    }
    #[test] 
    fn test_recv_client() {
        let op = ChunkOp{
            path: "".to_string(),
            offset: 0,
            chunk_start: 0,
            size: "hello, here is RDMA data transfer test!\0".len() as u64,
            op: show,
        };
        if let Ok(data) = RDMA::recver_client(&"192.168.230.142".to_string(), 20532, op){
            println!("receiver result: {}", data);
        }
    }
}
