use std::ptr::null_mut;

use libc::{sockaddr_in, AF_INET, in_addr, INADDR_LOOPBACK, sockaddr, calloc};
use rdma_sys::{rdma_create_event_channel, rdma_cm_id, rdma_cm_event, rdma_create_id, rdma_bind_addr, rdma_listen, rdma_cm_event_type::{RDMA_CM_EVENT_CONNECT_REQUEST, RDMA_CM_EVENT_ESTABLISHED, RDMA_CM_EVENT_DISCONNECTED}, rdma_ack_cm_event, ibv_alloc_pd, ibv_create_comp_channel, ibv_create_cq, rdma_port_space::RDMA_PS_TCP, ibv_req_notify_cq, ibv_qp_init_attr, ibv_qp_type::IBV_QPT_RC, rdma_create_qp, ibv_reg_mr, ibv_access_flags, rdma_conn_param, rdma_accept, rdma_destroy_qp, rdma_destroy_id, rdma_destroy_event_channel};
use crate::CHUNK_SIZE;

use crate::{get_addr, process_rdma_cm_event, CQ_CAPACITY, MAX_WR, MAX_SGE, transfer::{ReceiverContext, Message, MessageType}, build_params, rdma::CQPoller, chunk_operation::ChunkOp};

pub(crate) fn recver_server(addr: String, port: u16, op: ChunkOp) -> Result<i64, i32>{
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
        let mut listener: *mut rdma_cm_id = null_mut();
        let mut cm_event: *mut rdma_cm_event = null_mut();
        let ec = rdma_create_event_channel();
        rdma_create_id(ec, &mut listener, null_mut(), RDMA_PS_TCP);
        rdma_bind_addr(
            listener,
            (&mut server_sockaddr) as *mut sockaddr_in as *mut sockaddr,
        );
        // listen on socket
        assert_eq!(rdma_listen(listener, 10), 0);

        process_rdma_cm_event(ec, RDMA_CM_EVENT_CONNECT_REQUEST, &mut cm_event);
        let cm_id = (*cm_event).id;
        rdma_ack_cm_event(cm_event);

        // prepare RDMA resource
        let pd = ibv_alloc_pd((*cm_id).verbs);
        assert!(!pd.is_null());
        let comp_channel = ibv_create_comp_channel((*cm_id).verbs);
        assert!(!comp_channel.is_null());
        let cq = ibv_create_cq((*cm_id).verbs, CQ_CAPACITY, null_mut(), comp_channel, 0);
        assert!(!cq.is_null());
        assert_eq!(ibv_req_notify_cq(cq, 0), 0);
        
        let poll_cq = CQPoller::new(comp_channel, pd, crate::recv::on_completion, op);
        let handle = std::thread::spawn(move || {poll_cq.poll()});

        let mut attr: ibv_qp_init_attr = std::mem::zeroed();
        attr.send_cq = cq;
        attr.recv_cq = cq;
        attr.qp_type = IBV_QPT_RC;

        attr.cap.max_send_wr = MAX_WR;
        attr.cap.max_recv_wr = MAX_WR;
        attr.cap.max_send_sge = MAX_SGE;
        attr.cap.max_recv_sge = MAX_SGE;
        
        assert_eq!(rdma_create_qp(cm_id, pd, &mut attr), 0);
        let _qp = (*cm_id).qp;

        // prepare receiver context
        let mut ctx: ReceiverContext = std::mem::zeroed();
        (*cm_id).context = ((&mut ctx) as *mut ReceiverContext).cast();

        ctx.buffer = calloc(1, CHUNK_SIZE as usize).cast();
        ctx.buffer_mr =
            ibv_reg_mr(pd, ctx.buffer.cast(), CHUNK_SIZE as usize, (ibv_access_flags::IBV_ACCESS_REMOTE_WRITE | ibv_access_flags::IBV_ACCESS_REMOTE_READ | ibv_access_flags::IBV_ACCESS_LOCAL_WRITE).0 as i32);

        ctx.msg = calloc(1, std::mem::size_of::<Message>()).cast();
        ctx.msg_mr = ibv_reg_mr(
            pd,
            ctx.msg.cast(),
            std::mem::size_of::<Message>(),
            ibv_access_flags::IBV_ACCESS_LOCAL_WRITE.0 as i32,
        );
        // pre-post receive buffer
        crate::recv::post_receive(cm_id);

        // accept connection
        let mut cm_params: rdma_conn_param = std::mem::zeroed();
        build_params(&mut cm_params);

        rdma_accept(cm_id, &mut cm_params);
        process_rdma_cm_event(ec, RDMA_CM_EVENT_ESTABLISHED, &mut cm_event);
        rdma_ack_cm_event(cm_event);

        // send message to notify client local buffer
        (*ctx.msg).mtype = MessageType::MSG_MR;
        (*ctx.msg).addr = (*ctx.buffer_mr).addr as u64;
        (*ctx.msg).rkey = (*ctx.buffer_mr).rkey;

        crate::recv::send_message(cm_id);

        process_rdma_cm_event(ec, RDMA_CM_EVENT_DISCONNECTED, &mut cm_event);
        rdma_ack_cm_event(cm_event);

        rdma_destroy_qp(cm_id);
        rdma_destroy_id(cm_id);
        //ibv_dereg_mr(ctx.buffer_mr);
        //ibv_dereg_mr(ctx.msg_mr);

        libc::free(ctx.buffer.cast());
        libc::free(ctx.msg.cast());

        rdma_destroy_id(listener);
        rdma_destroy_event_channel(ec);
        
        return handle.join().unwrap();
    }
}