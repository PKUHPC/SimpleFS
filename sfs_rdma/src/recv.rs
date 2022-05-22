use std::ptr::null_mut;

use rdma_sys::{ibv_wc, ibv_pd, rdma_cm_id, ibv_wc_opcode::IBV_WC_RECV_RDMA_WITH_IMM, ibv_send_wr, ibv_sge, ibv_wr_opcode::IBV_WR_SEND, ibv_send_flags, ibv_post_send, ibv_recv_wr, ibv_post_recv};

use crate::{transfer::{MessageType, ReceiverContext}, chunk_operation::{ChunkOp, ChunkInfo}};


pub (crate) fn on_completion(wc: *mut ibv_wc, _pd: *mut ibv_pd, op: &ChunkOp) -> Result<i64, i32>{
    unsafe{
        let id: *mut rdma_cm_id = (*wc).wr_id as *mut rdma_cm_id;
        let ctx: *mut ReceiverContext = (*id).context.cast();

        if (*wc).opcode == IBV_WC_RECV_RDMA_WITH_IMM{
            let chunk_id = u32::from_be((*wc).imm_data_invalidated_rkey_union.imm_data);
            if chunk_id == u32::MAX{
                (*(*ctx).msg).mtype = MessageType::MSG_DONE;
                send_message(id);
                return Ok(-1);
            }
            else{
                post_receive(id);
                (*(*ctx).msg).mtype = MessageType::MSG_READY;
                send_message(id);
                return op.submit(ChunkInfo{ chunk_id: chunk_id as u64, data: (*ctx).buffer });
            }
        }
        return Ok(0);
    }
}
pub (crate) fn send_message(id: *mut rdma_cm_id){
    unsafe{
        let ctx: *mut ReceiverContext = (*id).context.cast();
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
pub (crate) fn post_receive(id: *mut rdma_cm_id){
    unsafe{
        let mut wr: ibv_recv_wr = std::mem::zeroed();
        let mut bad_wr: *mut ibv_recv_wr = null_mut();
        wr.wr_id = id as u64;
        wr.sg_list = null_mut();
        wr.num_sge = 0;

        assert_eq!(ibv_post_recv((*id).qp, &mut wr, &mut bad_wr), 0);
    }
}