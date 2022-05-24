use std::ptr::null_mut;

use crate::{transfer::TransferMetadata, CHUNK_SIZE};
use errno::errno;
use rdma_sys::{
    ibv_dereg_mr, ibv_pd, ibv_post_recv, ibv_post_send, ibv_recv_wr, ibv_reg_mr, ibv_send_flags,
    ibv_send_wr, ibv_sge, ibv_wc, ibv_wc_opcode::IBV_WC_RECV,
    ibv_wr_opcode::IBV_WR_RDMA_WRITE_WITH_IMM, imm_data_invalidated_rkey_union_t, rdma_cm_id,
    rdma_disconnect,
};

use crate::{
    chunk_operation::ChunkOp,
    transfer::{MessageType, SenderContext},
};

pub(crate) fn on_completion(wc: *mut ibv_wc, pd: *mut ibv_pd, _op: &ChunkOp) -> Result<i64, i32> {
    unsafe {
        let id: *mut rdma_cm_id = (*wc).wr_id as *mut rdma_cm_id;
        let ctx: *mut SenderContext = (*id).context.cast();

        if (*wc).opcode & IBV_WC_RECV != 0 {
            if matches!((*(*ctx).msg).mtype, MessageType::MSG_MR) {
                post_receive(id);
                (*ctx).peer_addr = (*(*ctx).msg).addr;
                (*ctx).peer_rkey = (*(*ctx).msg).rkey;
                send_metadata(id, pd);
            } else if matches!((*(*ctx).msg).mtype, MessageType::MSG_READY) {
                post_receive(id);
                return send_next_chunk(id, pd);
            } else if matches!((*(*ctx).msg).mtype, MessageType::MSG_DONE) {
                sender_disconnect(id);
                return Ok(-1);
            }
        }
        return Ok(0);
    }
}
pub(crate) enum WriteOp {
    META,
    DATA,
}
pub(crate) fn write_remote(id: *mut rdma_cm_id, len: u32, op: WriteOp) -> Result<i64, i32> {
    unsafe {
        let ctx: *mut SenderContext = (*id).context.cast();
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
            return Err(errno().0);
        }
        return Ok(len as i64);
    }
}
pub(crate) fn send_next_chunk(id: *mut rdma_cm_id, pd: *mut ibv_pd) -> Result<i64, i32> {
    unsafe {
        let mut ctx: *mut SenderContext = (*id).context.cast();

        let transfer_size = if (*ctx).chunk_id.len() > 0 {
            assert_eq!(ibv_dereg_mr((*ctx).buffer_mr), 0);

            let offset = if (*ctx).chunk_id[0] == (*ctx).metadata.chunk_start {
                0
            } else {
                CHUNK_SIZE * ((*ctx).chunk_id[0] - (*ctx).metadata.chunk_start)
                    - (*ctx).metadata.offset
            };
            (*ctx).buffer = ((*ctx).addr as *mut u8).offset(offset as isize);
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
pub(crate) fn post_receive(id: *mut rdma_cm_id) {
    unsafe {
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
pub(crate) fn sender_disconnect(id: *mut rdma_cm_id) {
    unsafe {
        rdma_disconnect(id);
    }
}
pub(crate) fn send_metadata(id: *mut rdma_cm_id, pd: *mut ibv_pd) {
    unsafe {
        let mut ctx: *mut SenderContext = (*id).context.cast();

        assert_eq!(ibv_dereg_mr((*ctx).buffer_mr), 0);
        let len = std::mem::size_of_val(&(*ctx).metadata);
        (*ctx).buffer = (&mut (*ctx).metadata) as *mut TransferMetadata as *mut u8;
        (*ctx).buffer_mr = ibv_reg_mr(pd, (*ctx).buffer.cast(), len as usize, 0);

        write_remote(id, len as u32, WriteOp::META).unwrap();
    }
}
