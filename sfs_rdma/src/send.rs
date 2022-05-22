use std::ptr::null_mut;

use rdma_sys::{ibv_wc, ibv_pd, rdma_cm_id, ibv_wc_opcode::IBV_WC_RECV, ibv_send_wr, ibv_wr_opcode::IBV_WR_RDMA_WRITE_WITH_IMM, ibv_send_flags, imm_data_invalidated_rkey_union_t, ibv_sge, ibv_post_send, ibv_dereg_mr, ibv_reg_mr, ibv_recv_wr, ibv_post_recv, rdma_disconnect};
use crate::CHUNK_SIZE;

use crate::{transfer::{SenderContext, MessageType}, chunk_operation::ChunkOp};


pub(crate) fn on_completion(wc: *mut ibv_wc, pd: *mut ibv_pd, _op: &ChunkOp) -> Result<i64, i32>{
    unsafe{
        let id: *mut rdma_cm_id = (*wc).wr_id as *mut rdma_cm_id;
        let ctx: *mut SenderContext = (*id).context.cast();

        if (*wc).opcode & IBV_WC_RECV != 0{
            if matches!((*(*ctx).msg).mtype, MessageType::MSG_MR){
                post_receive(id);
                (*ctx).peer_addr = (*(*ctx).msg).addr;
                (*ctx).peer_rkey = (*(*ctx).msg).rkey;
                send_next_chunk(id, pd);
            }
            else if matches!((*(*ctx).msg).mtype, MessageType::MSG_READY){
                post_receive(id);
                send_next_chunk(id, pd);
            }
            else if matches!((*(*ctx).msg).mtype, MessageType::MSG_DONE){
                sender_disconnect(id);
                return Ok(-1);
            }
        }
        return Ok(0);
    }
}
pub(crate) fn write_remote(id: *mut rdma_cm_id, len: u32) {
    unsafe {
        let ctx: *mut SenderContext = (*id).context.cast();
        let mut wr: ibv_send_wr = std::mem::zeroed();

        wr.wr_id = id as u64;
        wr.opcode = IBV_WR_RDMA_WRITE_WITH_IMM;
        wr.send_flags = ibv_send_flags::IBV_SEND_SIGNALED.0;
        wr.imm_data_invalidated_rkey_union = imm_data_invalidated_rkey_union_t {
            imm_data: if len > 0 {
                (*(*ctx).chunk_id).to_be() as u32
            } else {
                u32::MAX
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
        assert_eq!(
            ibv_post_send(
                (*id).qp,
                (&mut wr) as *mut ibv_send_wr,
                (&mut bad_wr) as *mut *mut ibv_send_wr
            ),
            0
        );
    }
}
pub(crate) fn send_next_chunk(id: *mut rdma_cm_id, pd: *mut ibv_pd){
    unsafe {
        let mut ctx: *mut SenderContext = (*id).context.cast();

        let transfer_size = if (*ctx).chunks > 0 {
            assert_eq!(ibv_dereg_mr((*ctx).buffer_mr), 0);

            let offset = CHUNK_SIZE * (*(*ctx).chunk_id);
            (*ctx).buffer = ((*ctx).addr as *mut u8).offset(offset as isize);
            let len = u64::min(CHUNK_SIZE, (*ctx).size - offset);
            (*ctx).buffer_mr = ibv_reg_mr(pd, (*ctx).buffer.cast(), len as usize, 0);

            (*ctx).chunks -= 1;
            (*ctx).chunk_id = (*ctx).chunk_id.offset(1);
            len
        } else {
            0
        };
        write_remote(id, transfer_size as u32);
    }
}
pub(crate) fn post_receive(id: *mut rdma_cm_id){
    unsafe{
        let ctx: *mut SenderContext = (*id).context.cast();

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
pub(crate) fn sender_disconnect(id: *mut rdma_cm_id){
    unsafe{
        rdma_disconnect(id);
    }
}