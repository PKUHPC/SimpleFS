use std::ptr::null_mut;

use errno::errno;
use libc::{c_void, calloc, in_addr, sockaddr, sockaddr_in, AF_INET, INADDR_LOOPBACK};
use rdma_sys::{
    ibv_access_flags, ibv_alloc_pd, 
    ibv_create_comp_channel, ibv_create_cq, ibv_dealloc_pd, ibv_dereg_mr, ibv_destroy_comp_channel,
    ibv_destroy_cq, ibv_mr, ibv_pd, ibv_post_recv, ibv_post_send,
    ibv_qp_init_attr,
    ibv_qp_type::IBV_QPT_RC,
    ibv_recv_wr, ibv_reg_mr, ibv_req_notify_cq, ibv_send_flags, ibv_send_wr, ibv_sge, ibv_wc,
    ibv_wc_opcode::IBV_WC_RECV,
    ibv_wr_opcode::IBV_WR_RDMA_WRITE_WITH_IMM,
    imm_data_invalidated_rkey_union_t,
    rdma_cm_id, rdma_create_id,
    rdma_create_qp, rdma_destroy_id, rdma_destroy_qp, rdma_disconnect,
    rdma_port_space::RDMA_PS_TCP,
    rdma_resolve_addr, rdma_event_channel, 
};
use sfs_global::global::network::config::CHUNK_SIZE;
use sfs_rdma::{transfer::{MessageType, TransferMetadata}, rdma::{CQPoller, RDMAContext}, RDMA_WRITE_PORT};

use sfs_rdma::{
    chunk_operation::ChunkOp,
    get_addr, 
    transfer::{ChunkTransferTask, Message},
    CQ_CAPACITY, MAX_SGE, MAX_WR,
};
use tokio::{task::JoinHandle, sync::{oneshot::{self}}};

use crate::client::{network::rdmacm::RDMACMContext, context::StaticContext};

#[derive(Debug)]
struct SenderClientContext {
    pub chunk_id: Vec<u64>,
    pub metadata: TransferMetadata,
    pub addr: u64,

    pub buffer: *mut u8,
    pub buffer_mr: *mut ibv_mr,

    pub msg: *mut Message,
    pub msg_mr: *mut ibv_mr,

    pub peer_addr: u64,
    pub peer_rkey: u32,
}
impl SenderClientContext {
    pub fn new() -> Self {
        SenderClientContext {
            chunk_id: Vec::new(),
            metadata: TransferMetadata::default(),
            addr: 0,
            buffer: null_mut(),
            buffer_mr: null_mut(),
            msg: null_mut(),
            msg_mr: null_mut(),
            peer_addr: 0,
            peer_rkey: 0
        }
    }
}

#[allow(unused)]
// this method is not capatible with new system, should be discarded and replaced by sender_client_on_id
pub(crate) async fn sender_client(
    addr: &String,
    port: u16,
    task: ChunkTransferTask,
    op: ChunkOp,
) -> JoinHandle<Result<i64, i32>> {
    let mut server_sockaddr = sockaddr_in {
        sin_family: AF_INET as u16,
        sin_port: port,
        sin_addr: in_addr {
            s_addr: INADDR_LOOPBACK,
        },
        sin_zero: [0; 8],
    };
    assert_eq!(
        get_addr(
            addr,
            port,
            (&mut server_sockaddr) as *mut sockaddr_in as *mut sockaddr,
        ),
        0
    );
    unsafe {
        let mut ctx: *mut SenderClientContext =
            libc::calloc(1, std::mem::size_of::<SenderClientContext>()).cast();
        let init_ctx = SenderClientContext::new();
        libc::memcpy(
            ctx.cast(),
            (&init_ctx) as *const SenderClientContext as *const c_void,
            std::mem::size_of::<SenderClientContext>(),
        );
        (*ctx).addr = task.addr;
        (*ctx).chunk_id = task.chunk_id;
        let mut md = TransferMetadata::default();
        md.size = task.metadata.size;
        md.offset = task.metadata.offset;
        md.chunk_start = task.metadata.chunk_start;
        md.path_len = task.metadata.path.len();
        libc::memcpy(
            md.path.as_mut_ptr().cast(),
            task.metadata.path.as_ptr().cast(),
            task.metadata.path.len(),
        );
        (*ctx).metadata = md;

        let cm_ctx: *mut RDMACMContext = 
            libc::calloc(1, std::mem::size_of::<RDMACMContext>()).cast();
        let init_ctx = RDMACMContext{
            ctx: ctx as u64,
            on_route_resolved,
            on_established,
            on_disconnect,
            s_ctx: null_mut(),
            tx: None
        };
        libc::memcpy(
            cm_ctx.cast(),
            (&init_ctx) as *const RDMACMContext as *const c_void,
            std::mem::size_of::<RDMACMContext>(),
        );

        let mut cm_id: *mut rdma_cm_id = null_mut();
        assert_eq!(rdma_create_id(StaticContext::get_instance().get_event_channel(), &mut cm_id, null_mut(), RDMA_PS_TCP), 0);
        (*cm_id).context = cm_ctx.cast();

        let (tx, rx) = oneshot::channel();
        (*cm_ctx).tx = Some(tx);
        // resolve addr
        assert_eq!(
            rdma_resolve_addr(
                cm_id,
                null_mut(),
                (&mut server_sockaddr) as *mut sockaddr_in as *mut sockaddr,
                2000,
            ),
            0
        );
        let _msg = rx.await.unwrap();
        let cm_ctx_addr = cm_ctx as u64;
        let cm_id_addr = cm_id as u64;
        let handle = tokio::spawn(async move {
                let cm_ctx = cm_ctx_addr as *mut RDMACMContext;
                let s_ctx = (*cm_ctx).s_ctx;
                let pd = (*s_ctx).pd;
                let comp_channel = (*s_ctx).comp_channel;

                let poll_cq = CQPoller::new(comp_channel, pd, on_completion, op);
                let result = poll_cq.poll();
                
                rdma_disconnect(cm_id_addr as *mut rdma_cm_id);
                return result;
            }
            
        );
        return handle;
    }
}

fn on_completion(wc: *mut ibv_wc, pd: *mut ibv_pd, _op: &ChunkOp) -> Result<i64, i32> {
    unsafe {
        let id: *mut rdma_cm_id = (*wc).wr_id as *mut rdma_cm_id;
        let cm_ctx: *mut RDMACMContext = (*id).context.cast();
        let ctx = (*cm_ctx).ctx as *mut SenderClientContext;

        if (*wc).opcode & IBV_WC_RECV != 0 {
            if matches!((*(*ctx).msg).mtype, MessageType::MSG_MR) {
                post_receive(id);
                (*ctx).peer_addr = (*(*ctx).msg).addr;
                (*ctx).peer_rkey = (*(*ctx).msg).rkey;
                send_metadata(id, pd);
                return Ok(0);
            } else if matches!((*(*ctx).msg).mtype, MessageType::MSG_READY) {
                post_receive(id);
                if let Err(e) = send_next_chunk(id, pd) {
                    return Err(e);
                }
                return Ok(0);
            } else if matches!((*(*ctx).msg).mtype, MessageType::MSG_DONE) {
                let result = (*(*ctx).msg).data as i64 as i32;
                post_receive(id);
                return Err(result);
            }
        }
        return Ok(0);
    }
}
pub(crate) enum WriteOp {
    META,
    DATA,
}
fn write_remote(id: *mut rdma_cm_id, len: u32, op: WriteOp) -> Result<i64, i32> {
    unsafe {
        let cm_ctx: *mut RDMACMContext = (*id).context.cast();
        let ctx = (*cm_ctx).ctx as *mut SenderClientContext;
        let mut wr: ibv_send_wr = std::mem::zeroed();

        wr.wr_id = id as u64;
        wr.opcode = IBV_WR_RDMA_WRITE_WITH_IMM;
        wr.send_flags = ibv_send_flags::IBV_SEND_SIGNALED.0;
        wr.imm_data_invalidated_rkey_union = imm_data_invalidated_rkey_union_t {
            imm_data: match op {
                WriteOp::META => len.to_be(),
                WriteOp::DATA => {
                    if len > 0 {
                        let chunk_id = (*ctx).chunk_id.remove(0) as u32;
                        chunk_id.to_be()
                    } else {
                        u32::MAX.to_be()
                    }
                }
            },
        };
        wr.wr.rdma.remote_addr = (*ctx).peer_addr;
        wr.wr.rdma.rkey = (*ctx).peer_rkey;

        let mut sge = ibv_sge {
            addr: (*ctx).buffer as u64,
            length: len,
            lkey: (*(*ctx).buffer_mr).lkey,
        };
        if len > 0 {
            wr.sg_list = (&mut sge) as *mut ibv_sge;
            wr.num_sge = 1;
        }
        let mut bad_wr: *mut ibv_send_wr = null_mut();

        let ret = ibv_post_send(
            (*id).qp,
            (&mut wr) as *mut ibv_send_wr,
            (&mut bad_wr) as *mut *mut ibv_send_wr,
        );
        if ret != 0 {
            return Err(-errno().0);
        }
        return Ok(len as i64);
    }
}
fn send_next_chunk(id: *mut rdma_cm_id, pd: *mut ibv_pd) -> Result<i64, i32> {
    unsafe {
        let cm_ctx: *mut RDMACMContext = (*id).context.cast();
        let ctx = (*cm_ctx).ctx as *mut SenderClientContext;

        let transfer_size = if (*ctx).chunk_id.len() > 0 {
            assert_eq!(ibv_dereg_mr((*ctx).buffer_mr), 0);

            let offset = if (*ctx).chunk_id[0] == (*ctx).metadata.chunk_start {
                0
            } else {
                CHUNK_SIZE * ((*ctx).chunk_id[0] - (*ctx).metadata.chunk_start)
                    - (*ctx).metadata.offset
            };
            (*ctx).buffer = ((*ctx).addr as *mut u8).offset(offset as isize).cast();
            let len = if (*ctx).chunk_id[0] == (*ctx).metadata.chunk_start {
                u64::min(CHUNK_SIZE - (*ctx).metadata.offset, (*ctx).metadata.size)
            } else {
                u64::min(CHUNK_SIZE, (*ctx).metadata.size - offset)
            };
            (*ctx).buffer_mr = ibv_reg_mr(pd, (*ctx).buffer.cast(), len as usize, 0);

            //println!("{} - {}: {} {} | {} {}", (*ctx).chunk_start, (*ctx).chunk_id[0], (*ctx).offset, (*ctx).size, offset, len);
            len
        } else {
            0
        };
        return write_remote(id, transfer_size as u32, WriteOp::DATA);
    }
}
fn post_receive(id: *mut rdma_cm_id) {
    unsafe {
        let cm_ctx: *mut RDMACMContext = (*id).context.cast();
        let ctx = (*cm_ctx).ctx as *mut SenderClientContext;

        let mut wr: ibv_recv_wr = std::mem::zeroed();
        let mut bad_wr: *mut ibv_recv_wr = null_mut();
        let mut sge = ibv_sge {
            addr: (*(*ctx).msg_mr).addr as u64,
            length: (*(*ctx).msg_mr).length as u32,
            lkey: (*(*ctx).msg_mr).lkey,
        };
        wr.wr_id = id as u64;
        wr.sg_list = &mut sge;
        wr.num_sge = 1;

        assert_eq!(ibv_post_recv((*id).qp, &mut wr, &mut bad_wr), 0);
    }
}
#[allow(unused)]
fn sender_disconnect(id: *mut rdma_cm_id) {
    unsafe {
        rdma_disconnect(id);
    }
}
fn send_metadata(id: *mut rdma_cm_id, pd: *mut ibv_pd) {
    unsafe {
        let cm_ctx: *mut RDMACMContext = (*id).context.cast();
        let ctx = (*cm_ctx).ctx as *mut SenderClientContext;

        assert_eq!(ibv_dereg_mr((*ctx).buffer_mr), 0);
        let len = std::mem::size_of_val(&(*ctx).metadata);
        (*ctx).buffer = (&mut (*ctx).metadata) as *mut TransferMetadata as *mut u8;
        (*ctx).buffer_mr = ibv_reg_mr(pd, (*ctx).buffer.cast(), len as usize, 0);

        write_remote(id, len as u32, WriteOp::META).unwrap();
    }
}
pub fn on_route_resolved(cm_id: *mut rdma_cm_id){
    unsafe{
        let cm_ctx = (*cm_id).context as *mut RDMACMContext;
        let ctx = (*cm_ctx).ctx as *mut SenderClientContext;
        // prepare RDMA resource
        let pd = ibv_alloc_pd((*cm_id).verbs);
        assert!(!pd.is_null());
        let comp_channel = ibv_create_comp_channel((*cm_id).verbs);
        assert!(!comp_channel.is_null());
        let cq = ibv_create_cq((*cm_id).verbs, CQ_CAPACITY, null_mut(), comp_channel, 0);
        assert!(!cq.is_null());
        assert_eq!(ibv_req_notify_cq(cq, 0), 0);
    
        let mut attr: ibv_qp_init_attr = std::mem::zeroed();
        attr.recv_cq = cq;
        attr.send_cq = cq;
        attr.qp_type = IBV_QPT_RC;
    
        attr.cap.max_send_wr = MAX_WR;
        attr.cap.max_recv_wr = MAX_WR;
        attr.cap.max_send_sge = MAX_SGE;
        attr.cap.max_recv_sge = MAX_SGE;
    
        assert_eq!(rdma_create_qp(cm_id, pd, &mut attr), 0);
        let _qp = (*cm_id).qp;
    
        // build client context buffer
        /* 
        (*ctx).buffer = (*ctx).addr as *mut u8;
    
        let offset = if (*ctx).chunk_id[0] == (*ctx).metadata.chunk_start {
            0
        } else {
            CHUNK_SIZE * ((*ctx).chunk_id[0] - (*ctx).metadata.chunk_start) - (*ctx).metadata.offset
        };
        (*ctx).buffer = ((*ctx).addr as *mut u8).offset(offset as isize).cast();
        let len = if (*ctx).chunk_id[0] == (*ctx).metadata.chunk_start {
            u64::min(CHUNK_SIZE - (*ctx).metadata.offset, (*ctx).metadata.size)
        } else {
            u64::min(CHUNK_SIZE, (*ctx).metadata.size - offset)
        };
    
        (*ctx).buffer_mr = ibv_reg_mr(pd, (*ctx).buffer.cast(), len as usize, 0);
        */

        (*ctx).msg = calloc(1, std::mem::size_of::<Message>()) as *mut Message;
        (*ctx).msg_mr = ibv_reg_mr(
            pd,
            (*ctx).msg.cast(),
            std::mem::size_of::<Message>(),
            ibv_access_flags::IBV_ACCESS_LOCAL_WRITE.0 as i32,
        );
        // this mr register is temporary, without this send_metadata will fail when dereg mr!
        (*ctx).buffer_mr = ibv_reg_mr(
            pd,
            (*ctx).msg.cast(),
            std::mem::size_of::<Message>(),
            ibv_access_flags::IBV_ACCESS_LOCAL_WRITE.0 as i32,
        );

        let s_ctx:*mut RDMAContext = libc::calloc(1, std::mem::size_of::<RDMAContext>()).cast();
        let init_ctx = RDMAContext{
            ctx: (*cm_id).verbs,
            pd,
            cq,
            comp_channel,
            cq_poller: None,
        };
        libc::memcpy(s_ctx.cast(), (&init_ctx) as *const RDMAContext as *const c_void, std::mem::size_of::<RDMAContext>());
        (*cm_ctx).s_ctx = s_ctx;
        // pre-post receive buffer
        post_receive(cm_id);   

    }
}
pub fn on_established(cm_id: *mut rdma_cm_id){
    unsafe{
        let cm_ctx = (*cm_id).context as *mut RDMACMContext;
        (*cm_ctx).tx.take().unwrap().send(1).unwrap(); 
    }
}
pub fn on_disconnect(cm_id: *mut rdma_cm_id){
    unsafe{
        let cm_ctx = (*cm_id).context as *mut RDMACMContext;
        let ctx = (*cm_ctx).ctx as *mut SenderClientContext;
        let s_ctx = (*cm_ctx).s_ctx;
        let pd = (*s_ctx).pd;
        let comp_channel = (*s_ctx).comp_channel;
        let cq = (*s_ctx).cq;

        ibv_dereg_mr((*ctx).buffer_mr);
        ibv_dereg_mr((*ctx).msg_mr);

        rdma_destroy_qp(cm_id);
        rdma_destroy_id(cm_id);
        libc::free((*ctx).msg.cast());

        ibv_dealloc_pd(pd);
        ibv_destroy_cq(cq);
        ibv_destroy_comp_channel(comp_channel);

        libc::free(s_ctx.cast());
        libc::free(ctx.cast());
        libc::free(cm_ctx.cast());
    }
}

pub async fn new_write_cm_id(
    ec: *mut rdma_event_channel,
    addr: &String
) -> u64
{
    let mut server_sockaddr = sockaddr_in {
        sin_family: AF_INET as u16,
        sin_port: RDMA_WRITE_PORT,
        sin_addr: in_addr {
            s_addr: INADDR_LOOPBACK,
        },
        sin_zero: [0; 8],
    };
    assert_eq!(
        get_addr(
            addr,
            RDMA_WRITE_PORT,
            (&mut server_sockaddr) as *mut sockaddr_in as *mut sockaddr,
        ),
        0
    );
    unsafe{
        let ctx: *mut SenderClientContext =
            libc::calloc(1, std::mem::size_of::<SenderClientContext>()).cast();
        let init_ctx = SenderClientContext::new();
        libc::memcpy(
            ctx.cast(),
            (&init_ctx) as *const SenderClientContext as *const c_void,
            std::mem::size_of::<SenderClientContext>(),
        );

        let cm_ctx: *mut RDMACMContext = 
            libc::calloc(1, std::mem::size_of::<RDMACMContext>()).cast();
        let init_ctx = RDMACMContext{
            ctx: ctx as u64,
            on_route_resolved,
            on_established,
            on_disconnect,
            s_ctx: null_mut(),
            tx: None
        };
        libc::memcpy(
            cm_ctx.cast(),
            (&init_ctx) as *const RDMACMContext as *const c_void,
            std::mem::size_of::<RDMACMContext>(),
        );

        let mut cm_id: *mut rdma_cm_id = null_mut();
        assert_eq!(rdma_create_id(ec, &mut cm_id, null_mut(), RDMA_PS_TCP), 0);
        (*cm_id).context = cm_ctx.cast();

        let (tx, rx) = oneshot::channel();
        (*cm_ctx).tx = Some(tx);
        // resolve addr
        assert_eq!(
            rdma_resolve_addr(
                cm_id,
                null_mut(),
                (&mut server_sockaddr) as *mut sockaddr_in as *mut sockaddr,
                2000,
            ),
            0
        );

        let _msg = rx.await.unwrap();
        return cm_id as u64;
    }   
}

pub(crate) async fn sender_client_on_id(
    cm_id: *mut rdma_cm_id,
    task: ChunkTransferTask,
    op: ChunkOp,
) -> JoinHandle<Result<i64, i32>> {
    unsafe {
        let cm_ctx = (*cm_id).context as *mut RDMACMContext;
        let mut ctx = (*cm_ctx).ctx as *mut SenderClientContext;
        (*ctx).addr = task.addr;
        (*ctx).chunk_id = task.chunk_id;
        let mut md = TransferMetadata::default();
        md.size = task.metadata.size;
        md.offset = task.metadata.offset;
        md.chunk_start = task.metadata.chunk_start;
        md.path_len = task.metadata.path.len();
        libc::memcpy(
            md.path.as_mut_ptr().cast(),
            task.metadata.path.as_ptr().cast(),
            task.metadata.path.len(),
        );
        (*ctx).metadata = md;

        let cm_ctx_addr = cm_ctx as u64;
        let cm_id_addr = cm_id as u64;
        let handle = tokio::spawn(async move {
                let cm_ctx = cm_ctx_addr as *mut RDMACMContext;
                let ctx = (*cm_ctx).ctx as *mut SenderClientContext;
                let s_ctx = (*cm_ctx).s_ctx;
                let pd = (*s_ctx).pd;
                let comp_channel = (*s_ctx).comp_channel;
                
                let cm_id = cm_id_addr as *mut rdma_cm_id;

                if !matches!((*(*ctx).msg).mtype, MessageType::MSG_MR){
                    send_metadata(cm_id, pd);
                }

                let poll_cq = CQPoller::new(comp_channel, pd, on_completion, op);
                let result = poll_cq.poll();
                
                return result;
            }
            
        );
        return handle;
    }
}