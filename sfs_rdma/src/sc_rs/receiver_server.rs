use std::{ptr::null_mut, sync::Arc};

use crate::{rdma::RDMAContext, CHUNK_SIZE, RDMA_WRITE_PORT, transfer::TransferMetadata};
use libc::{calloc, in_addr, sockaddr, sockaddr_in, AF_INET, INADDR_LOOPBACK};
use rdma_sys::{
    ibv_access_flags, ibv_alloc_pd, ibv_create_comp_channel, ibv_create_cq, ibv_dealloc_pd,
    ibv_dereg_mr, ibv_destroy_comp_channel, ibv_destroy_cq, ibv_qp_init_attr,
    ibv_qp_type::IBV_QPT_RC,
    ibv_reg_mr, ibv_req_notify_cq, rdma_accept, rdma_ack_cm_event, rdma_bind_addr, rdma_cm_event,
    rdma_cm_event_type::{
        RDMA_CM_EVENT_CONNECT_REQUEST, RDMA_CM_EVENT_DISCONNECTED, RDMA_CM_EVENT_ESTABLISHED,
    },
    rdma_cm_id, rdma_conn_param, rdma_create_event_channel, rdma_create_id, rdma_create_qp,
    rdma_destroy_event_channel, rdma_destroy_id, rdma_destroy_qp, rdma_event_str,
    rdma_get_cm_event, rdma_listen,
    rdma_port_space::RDMA_PS_TCP,
};

use crate::{
    build_params,
    chunk_operation::ChunkOp,
    get_addr,
    rdma::CQPoller,
    transfer::{Message, MessageType, ReceiverContext},
    CQ_CAPACITY, MAX_SGE, MAX_WR,
};
pub(crate) fn recver_server(addr: &String, op: ChunkOp, nthreads: u32) {
    unsafe {
        let port = RDMA_WRITE_PORT; //portpicker::pick_unused_port().expect("no port available");
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

                    let poll_cq =
                        CQPoller::new(comp_channel, pd, crate::recv::on_completion, op.clone());
                    workers.execute(move || {poll_cq.poll().unwrap();});

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
                    let mut ctx: *mut ReceiverContext =
                        calloc(1, std::mem::size_of::<ReceiverContext>()).cast();
                    (*cm_id).context = ctx.cast();
                    (*ctx).s_ctx = s_ctx;

                    let rdma_buffer_size = usize::max(CHUNK_SIZE as usize, std::mem::size_of::<TransferMetadata>());
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
                    crate::recv::post_receive(cm_id);

                    // accept connection
                    let mut cm_params: rdma_conn_param = std::mem::zeroed();
                    build_params(&mut cm_params);

                    rdma_accept(cm_id, &mut cm_params);
                }
                RDMA_CM_EVENT_ESTABLISHED => {
                    let cm_id = (*cm_event).id;
                    rdma_ack_cm_event(cm_event);

                    let ctx: *mut ReceiverContext = (*cm_id).context.cast();
                    // send message to notify client local buffer
                    (*(*ctx).msg).mtype = MessageType::MSG_MR;
                    (*(*ctx).msg).addr = (*(*ctx).buffer_mr).addr as u64;
                    (*(*ctx).msg).rkey = (*(*ctx).buffer_mr).rkey;

                    crate::recv::send_message(cm_id);
                }
                RDMA_CM_EVENT_DISCONNECTED => {
                    let cm_id = (*cm_event).id;
                    rdma_ack_cm_event(cm_event);
                    let ctx: *mut ReceiverContext = (*cm_id).context.cast();
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
