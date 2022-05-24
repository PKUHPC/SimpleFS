use std::ptr::null_mut;

use libc::c_void;
use rdma_sys::{
    ibv_pd, ibv_post_recv, ibv_post_send, ibv_recv_wr, ibv_send_flags, ibv_send_wr, ibv_sge,
    ibv_wc, ibv_wc_opcode::IBV_WC_RECV_RDMA_WITH_IMM, ibv_wr_opcode::IBV_WR_SEND, rdma_cm_id,
};

use crate::{
    chunk_operation::{ChunkInfo, ChunkOp},
    transfer::{MessageType, ReceiverContext, TransferMetadata},
};

pub(crate) fn on_completion(wc: *mut ibv_wc, _pd: *mut ibv_pd, op: &ChunkOp) -> Result<i64, i32> {
    unsafe {
        let id: *mut rdma_cm_id = (*wc).wr_id as *mut rdma_cm_id;
        let ctx: *mut ReceiverContext = (*id).context.cast();

        if (*wc).opcode == IBV_WC_RECV_RDMA_WITH_IMM {
            let chunk_id = u32::from_be((*wc).imm_data_invalidated_rkey_union.imm_data);
            if chunk_id == u32::MAX {
                (*(*ctx).msg).mtype = MessageType::MSG_DONE;
                send_message(id);
                return Ok(-1);
            } else if (*ctx).metadata.size != 0 {
                post_receive(id);
                (*(*ctx).msg).mtype = MessageType::MSG_READY;
                let ret = op.submit(ChunkInfo {
                    chunk_id: chunk_id as u64,
                    metadata: (*ctx).metadata.clone(),
                    data: (*ctx).buffer,
                });
                send_message(id);
                return ret;
            } else {
                let len = chunk_id;
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

                post_receive(id);
                (*(*ctx).msg).mtype = MessageType::MSG_READY;
                send_message(id);
                return Ok(0);
            }
        }
        return Ok(0);
    }
}
pub(crate) fn send_message(id: *mut rdma_cm_id) {
    unsafe {
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
pub(crate) fn post_receive(id: *mut rdma_cm_id) {
    unsafe {
        let mut wr: ibv_recv_wr = std::mem::zeroed();
        let mut bad_wr: *mut ibv_recv_wr = null_mut();
        wr.wr_id = id as u64;
        wr.sg_list = null_mut();
        wr.num_sge = 0;

        assert_eq!(ibv_post_recv((*id).qp, &mut wr, &mut bad_wr), 0);
    }
}
