use std::{
    ffi::CStr,
    ptr::{null, null_mut},
};

use errno::errno;
use libc::{
    addrinfo, c_char, c_void, freeaddrinfo, getaddrinfo, malloc, memcpy, sockaddr, sockaddr_in,
};
use rdma::RDMA;
use rdma_sys::{
    ibv_alloc_pd, ibv_context, ibv_create_comp_channel, ibv_create_cq, ibv_qp_init_attr,
    ibv_qp_type::IBV_QPT_RC, ibv_req_notify_cq, rdma_ack_cm_event, rdma_cm_event,
    rdma_cm_event_type, rdma_cm_id, rdma_conn_param, rdma_create_qp, rdma_event_channel,
    rdma_event_str, rdma_get_cm_event,
};
pub mod chunk_operation;
mod rc_ss;
pub mod rdma;
mod sc_rs;
pub mod transfer;

pub static CQ_CAPACITY: i32 = 16;
pub static MAX_SGE: u32 = 2;
pub static MAX_WR: u32 = 8;

const CHUNK_SIZE: u64 = sfs_global::global::network::config::CHUNK_SIZE;
pub static RDMA_WRITE_PORT: u16 = 8084;
pub static RDMA_READ_PORT: u16 = 8085;

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

#[cfg(test)]
mod tests {

    use crate::{
        chunk_operation::ChunkOp,
        rdma::RDMA,
        transfer::{ChunkMetadata, ChunkTransferTask},
        CHUNK_SIZE, RDMA_READ_PORT, RDMA_WRITE_PORT,
    };

    #[test]
    fn it_works() {}
    #[test]
    fn test_port() {}
    pub fn show(
        file_path: &String,
        _chunk_id: u64,
        buf: *mut u8,
        size: u64,
        _offset: u64,
    ) -> Result<i64, i32> {
        let slice = unsafe { std::slice::from_raw_parts(buf, size as usize) };
        let str = String::from_utf8(slice.to_vec()).unwrap();
        println!("received '{}' of '{}' with length {}", str, file_path, size);
        return Ok(size as i64);
    }
    #[test]
    fn test_recv_server() {
        let op = ChunkOp { op: show };
        let _handles = RDMA::recver_server(&"192.168.230.142".to_string(), op, 4);
    }
    #[test]
    fn test_send_client() {
        let data = "hello, here is RDMA data transfer test!\0";
        let offset = 11;
        let size = 24;
        let chunk_start = offset / CHUNK_SIZE;
        let chunks = ( offset + size) / CHUNK_SIZE;
        let chunk_ids: Vec<u64> = (chunk_start..chunks + 1).into_iter().collect();
        let task = ChunkTransferTask {
            metadata: ChunkMetadata {
                path: "testfile".to_string(),
                chunk_start,
                offset: offset % CHUNK_SIZE,
                size
            },
            chunk_id: chunk_ids,
            addr: data.as_ptr() as u64,
        };
        let op = ChunkOp { op: show };
        println!(
            "send {} bytes",
            RDMA::sender_client(&"192.168.230.142".to_string(), RDMA_WRITE_PORT, task, op).unwrap()
        );
    }
    pub fn read(
        _file_path: &String,
        chunk_id: u64,
        buf: *mut u8,
        size: u64,
        offset: u64,
    ) -> Result<i64, i32> {
        let data = "hello, here is RDMA data transfer test!\0";
        let start = chunk_id * CHUNK_SIZE + offset;
        unsafe {
            let ptr = data.as_ptr().offset(start as isize);
            libc::memcpy(buf.cast(), ptr.cast(), size as usize);
        }
        return Ok(size as i64);
    }
    #[test]
    fn test_send_server() {
        let op = ChunkOp { op: read };
        RDMA::sender_server(&"192.168.230.142".to_string(), op, 4);
    }
    #[test]
    fn test_recv_client() {
        let data = "hello, here is RDMA data transfer test!\0";
        let offset = 8;
        let size = 24 as u64;
        let chunk_start = offset / CHUNK_SIZE;
        let chunks = (size + offset) / CHUNK_SIZE;
        let chunk_ids: Vec<u64> = (chunk_start..chunks + 1).into_iter().collect();
        let buf = unsafe { libc::calloc(1, data.len() + 2) } as *mut u8;
        let task = ChunkTransferTask {
            metadata: ChunkMetadata {
                path: "testfile".to_string(),
                chunk_start,
                offset: offset % CHUNK_SIZE,
                size
            },
            chunk_id: chunk_ids,
            addr: buf as u64,
        };
        let op = ChunkOp { op: show };
        if let Ok(len) =
            RDMA::recver_client(&"192.168.230.142".to_string(), RDMA_READ_PORT, task, op)
        {
            println!(
                "receiver result: '{}' with length {}",
                unsafe {
                    std::ffi::CStr::from_ptr(buf.cast())
                        .to_string_lossy()
                        .into_owned()
                },
                len
            );
        }
        unsafe { libc::free(buf.cast()) };
    }
}
