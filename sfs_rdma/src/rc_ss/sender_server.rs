use std::{ptr::null_mut, sync::Arc};

use crate::{
    chunk_operation::ChunkInfo,
    rdma::RDMAContext,
    transfer::{ChunkMetadata, MessageType, TransferMetadata},
    CHUNK_SIZE, RDMA_READ_PORT,
};
use libc::{c_void, calloc, in_addr, sockaddr, sockaddr_in, AF_INET, INADDR_LOOPBACK};
use rdma_sys::{
    ibv_access_flags, ibv_alloc_pd, ibv_create_comp_channel, ibv_create_cq, ibv_dealloc_pd,
    ibv_dereg_mr, ibv_destroy_comp_channel, ibv_destroy_cq, ibv_mr, ibv_pd, ibv_post_recv,
    ibv_post_send, ibv_qp_init_attr,
    ibv_qp_type::IBV_QPT_RC,
    ibv_recv_wr, ibv_reg_mr, ibv_req_notify_cq, ibv_send_flags, ibv_send_wr, ibv_sge, ibv_wc,
    ibv_wc_opcode::{IBV_WC_RECV, IBV_WC_RECV_RDMA_WITH_IMM},
    ibv_wr_opcode::{IBV_WR_RDMA_WRITE_WITH_IMM, IBV_WR_SEND},
    imm_data_invalidated_rkey_union_t, rdma_accept, rdma_ack_cm_event, rdma_bind_addr,
    rdma_cm_event,
    rdma_cm_event_type::{
        RDMA_CM_EVENT_CONNECT_REQUEST, RDMA_CM_EVENT_DISCONNECTED, RDMA_CM_EVENT_ESTABLISHED,
    },
    rdma_cm_id, rdma_conn_param, rdma_create_event_channel, rdma_create_id, rdma_create_qp,
    rdma_destroy_event_channel, rdma_destroy_id, rdma_destroy_qp, rdma_event_str,
    rdma_get_cm_event, rdma_listen,
    rdma_port_space::RDMA_PS_TCP, rdma_disconnect,
};

use crate::{
    build_params, chunk_operation::ChunkOp, get_addr, rdma::CQPoller, transfer::Message,
    CQ_CAPACITY, MAX_SGE, MAX_WR,
};

struct SenderServerContext {
    pub buffer: *mut u8,
    pub buffer_mr: *mut ibv_mr,

    pub msg: *mut Message,
    pub msg_mr: *mut ibv_mr,

    pub metadata: ChunkMetadata,
    pub s_ctx: *mut RDMAContext,
}
pub(crate) fn sender_server(addr: &String, op: ChunkOp, nthreads: u32) {
    unsafe {
        let port = RDMA_READ_PORT; //portpicker::pick_unused_port().expect("no port available");
        let mut listener: *mut rdma_cm_id = null_mut();
        let ec = rdma_create_event_channel();
        rdma_create_id(ec, &mut listener, null_mut(), RDMA_PS_TCP);
        let workers = Arc::new(threadpool::ThreadPool::new(nthreads as usize));

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
        rdma_bind_addr(
            listener,
            (&mut server_sockaddr) as *mut sockaddr_in as *mut sockaddr,
        );
        // listen on socket
        assert_eq!(rdma_listen(listener, 10), 0);

        let mut cm_event: *mut rdma_cm_event = null_mut();
        while rdma_get_cm_event(ec, &mut cm_event) == 0 {
            let ret = (*cm_event).status;
            if ret != 0 {
                println!("CM event has non zero status: {}", ret);
                rdma_ack_cm_event(cm_event);
                break;
            }
            match (*cm_event).event {
                RDMA_CM_EVENT_CONNECT_REQUEST => {
                    let cm_id = (*cm_event).id;
                    rdma_ack_cm_event(cm_event);

                    let mut s_ctx: *mut RDMAContext = RDMAContext::new_ptr();
                    // prepare RDMA resource
                    let pd = ibv_alloc_pd((*cm_id).verbs);
                    assert!(!pd.is_null());
                    let comp_channel = ibv_create_comp_channel((*cm_id).verbs);
                    assert!(!comp_channel.is_null());
                    let cq =
                        ibv_create_cq((*cm_id).verbs, CQ_CAPACITY, null_mut(), comp_channel, 0);
                    assert!(!cq.is_null());
                    assert_eq!(ibv_req_notify_cq(cq, 0), 0);

                    let poll_cq = CQPoller::new(comp_channel, pd, on_completion, op.clone());
                    workers.execute(move || {
                        poll_cq.poll().unwrap();
                    });

                    let mut attr: ibv_qp_init_attr = std::mem::zeroed();
                    attr.send_cq = cq;
                    attr.recv_cq = cq;
                    attr.qp_type = IBV_QPT_RC;

                    attr.cap.max_send_wr = MAX_WR;
                    attr.cap.max_recv_wr = MAX_WR;
                    attr.cap.max_send_sge = MAX_SGE;
                    attr.cap.max_recv_sge = MAX_SGE;

                    assert_eq!(rdma_create_qp(cm_id, pd, &mut attr), 0);

                    (*s_ctx).ctx = (*cm_id).verbs;
                    (*s_ctx).cq = cq;
                    (*s_ctx).comp_channel = comp_channel;
                    (*s_ctx).pd = pd;
                    //(*s_ctx).cq_poller = Some(handle);

                    // prepare receiver context
                    let mut ctx: *mut SenderServerContext =
                        calloc(1, std::mem::size_of::<SenderServerContext>()).cast();
                    (*cm_id).context = ctx.cast();
                    (*ctx).s_ctx = s_ctx;

                    let rdma_buffer_size =
                        usize::max(CHUNK_SIZE as usize, std::mem::size_of::<TransferMetadata>());
                    (*ctx).buffer = calloc(1, rdma_buffer_size).cast();
                    (*ctx).buffer_mr = ibv_reg_mr(
                        pd,
                        (*ctx).buffer.cast(),
                        rdma_buffer_size,
                        (ibv_access_flags::IBV_ACCESS_REMOTE_WRITE
                            | ibv_access_flags::IBV_ACCESS_REMOTE_READ
                            | ibv_access_flags::IBV_ACCESS_LOCAL_WRITE)
                            .0 as i32,
                    );

                    (*ctx).msg = calloc(1, std::mem::size_of::<Message>()).cast();
                    (*ctx).msg_mr = ibv_reg_mr(
                        pd,
                        (*ctx).msg.cast(),
                        std::mem::size_of::<Message>(),
                        ibv_access_flags::IBV_ACCESS_LOCAL_WRITE.0 as i32,
                    );
                    // pre-post receive buffer
                    post_receive_meta(cm_id);

                    // accept connection
                    let mut cm_params: rdma_conn_param = std::mem::zeroed();
                    build_params(&mut cm_params);

                    rdma_accept(cm_id, &mut cm_params);
                }
                RDMA_CM_EVENT_ESTABLISHED => {
                    let cm_id = (*cm_event).id;
                    rdma_ack_cm_event(cm_event);

                    let ctx: *mut SenderServerContext = (*cm_id).context.cast();
                    // send message to notify client local buffer
                    (*(*ctx).msg).mtype = MessageType::MSG_MR;
                    (*(*ctx).msg).addr = (*(*ctx).buffer_mr).addr as u64;
                    (*(*ctx).msg).rkey = (*(*ctx).buffer_mr).rkey;

                    send_message(cm_id);
                }
                RDMA_CM_EVENT_DISCONNECTED => {
                    let cm_id = (*cm_event).id;
                    rdma_ack_cm_event(cm_event);
                    let ctx: *mut SenderServerContext = (*cm_id).context.cast();
                    let s_ctx: *mut RDMAContext = (*ctx).s_ctx;

                    ibv_dereg_mr((*ctx).buffer_mr);
                    ibv_dereg_mr((*ctx).msg_mr);

                    rdma_destroy_qp(cm_id);
                    rdma_destroy_id(cm_id);

                    libc::free((*ctx).buffer.cast());
                    libc::free((*ctx).msg.cast());

                    //(*s_ctx).cq_poller.take().unwrap().join().unwrap().unwrap();

                    ibv_dealloc_pd((*s_ctx).pd);
                    ibv_destroy_cq((*s_ctx).cq);
                    ibv_destroy_comp_channel((*s_ctx).comp_channel);

                    //std::alloc::dealloc(s_ctx.cast(), std::alloc::Layout::new::<RDMAContext>());
                    //std::alloc::dealloc(ctx.cast(), std::alloc::Layout::new::<ReceiveContext>());
                    libc::free(s_ctx.cast());
                    libc::free(ctx.cast());
                }
                _ => {
                    println!(
                        "unexpected event: {}",
                        std::ffi::CStr::from_ptr(rdma_event_str((*cm_event).event))
                            .to_string_lossy()
                            .into_owned()
                    );
                    break;
                }
            }
        }
        rdma_destroy_id(listener);
        rdma_destroy_event_channel(ec);
    }
}
fn on_completion(wc: *mut ibv_wc, _pd: *mut ibv_pd, op: &ChunkOp) -> Result<i64, i32> {
    unsafe {
        let id: *mut rdma_cm_id = (*wc).wr_id as *mut rdma_cm_id;
        let ctx: *mut SenderServerContext = (*id).context.cast();

        if (*wc).opcode == IBV_WC_RECV_RDMA_WITH_IMM {
            let len = u32::from_be((*wc).imm_data_invalidated_rkey_union.imm_data);
            let mut transfer_md = TransferMetadata::default();
            libc::memcpy(
                (&mut transfer_md) as *mut TransferMetadata as *mut c_void,
                (*ctx).buffer.cast(),
                len as usize,
            );
            (*ctx).metadata.path =
                String::from_utf8(transfer_md.path[0..transfer_md.path_len as usize].to_vec())
                    .unwrap();
            (*ctx).metadata.chunk_start = transfer_md.chunk_start;
            (*ctx).metadata.offset = transfer_md.offset;
            (*ctx).metadata.size = transfer_md.size;

            post_receive_msg(id);
            (*(*ctx).msg).mtype = MessageType::MSG_READY;
            send_message(id);
            return Ok(0);
        } else if (*wc).opcode & IBV_WC_RECV != 0 {
            if matches!((*(*ctx).msg).mtype, MessageType::MSG_DONE){
                rdma_disconnect(id);
                //post_receive_meta(id);
                return Err(0);
            }
            let info = ChunkInfo {
                chunk_id: (*(*ctx).msg).data,
                metadata: (*ctx).metadata.clone(),
                data: (*ctx).buffer,
            };
            // read chunk
            let len = op.submit(info).unwrap();

            // send to receiver client
            let mut wr: ibv_send_wr = std::mem::zeroed();
            wr.wr_id = id as u64;
            wr.opcode = IBV_WR_RDMA_WRITE_WITH_IMM;
            wr.send_flags = ibv_send_flags::IBV_SEND_SIGNALED.0;
            wr.imm_data_invalidated_rkey_union = imm_data_invalidated_rkey_union_t {
                imm_data: (len as u32).to_be(),
            };
            wr.wr.rdma.remote_addr = (*(*ctx).msg).addr;
            wr.wr.rdma.rkey = (*(*ctx).msg).rkey;

            let mut sge = ibv_sge {
                addr: (*ctx).buffer as u64,
                length: len as u32,
                lkey: (*(*ctx).buffer_mr).lkey,
            };
            if len > 0 {
                wr.sg_list = (&mut sge) as *mut ibv_sge;
                wr.num_sge = 1;
            }
            let mut bad_wr: *mut ibv_send_wr = null_mut();
            post_receive_msg(id);
            ibv_post_send(
                (*id).qp,
                (&mut wr) as *mut ibv_send_wr,
                (&mut bad_wr) as *mut *mut ibv_send_wr,
            );
        }
        return Ok(0);
    }
}
fn send_message(id: *mut rdma_cm_id) {
    unsafe {
        let ctx: *mut SenderServerContext = (*id).context.cast();
        let mut wr: ibv_send_wr = std::mem::zeroed();
        let mut bad_wr: *mut ibv_send_wr = null_mut();
        let mut sge = ibv_sge {
            addr: (*ctx).msg as u64,
            length: std::mem::size_of_val(&(*(*ctx).msg)) as u32,
            lkey: (*(*ctx).msg_mr).lkey,
        };

        wr.wr_id = id as u64;
        wr.opcode = IBV_WR_SEND;
        wr.sg_list = &mut sge;
        wr.num_sge = 1;
        wr.send_flags = ibv_send_flags::IBV_SEND_SIGNALED.0;

        assert_eq!(ibv_post_send((*id).qp, &mut wr, &mut bad_wr), 0);
    }
}
fn post_receive_meta(id: *mut rdma_cm_id) {
    unsafe {
        let mut wr: ibv_recv_wr = std::mem::zeroed();
        let mut bad_wr: *mut ibv_recv_wr = null_mut();
        wr.wr_id = id as u64;
        wr.sg_list = null_mut();
        wr.num_sge = 0;

        assert_eq!(ibv_post_recv((*id).qp, &mut wr, &mut bad_wr), 0);
    }
}
fn post_receive_msg(id: *mut rdma_cm_id) {
    unsafe {
        let ctx: *mut SenderServerContext = (*id).context.cast();

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
