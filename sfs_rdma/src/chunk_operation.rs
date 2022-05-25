use crate::{transfer::ChunkMetadata, CHUNK_SIZE};
pub fn offset_to_chunk_id(offset: i64, chunk_size: u64) -> u64 {
    //(chunk_align_down(offset, chunk_size) >> ((chunk_size as f64).log2() as i64)) as u64
    offset as u64 / chunk_size
}

pub struct ChunkInfo {
    pub chunk_id: u64,
    pub metadata: ChunkMetadata,
    pub data: *mut u8,
}
#[derive(Clone)]
pub struct ChunkOp {
    pub op: fn(&String, u64, *mut u8, u64, u64) -> Result<i64, i32>,
}
impl ChunkOp {
    pub fn submit(&self, chunk: ChunkInfo) -> Result<i64, i32> {
        let md = chunk.metadata;
        let buffer_offset = if chunk.chunk_id == md.chunk_start {
            0
        } else {
            CHUNK_SIZE * (chunk.chunk_id - md.chunk_start) - md.offset
        };
        let size = if chunk.chunk_id == md.chunk_start {
            u64::min(CHUNK_SIZE - md.offset, md.size)
        } else {
            u64::min(CHUNK_SIZE, md.size - buffer_offset)
        };
        //println!("{} - {}: {} {} | {} {}", self.chunk_start, chunk.chunk_id, self.offset, self.size, buffer_offset, size);
        let offset = if chunk.chunk_id == md.chunk_start {
            md.offset
        } else {
            0
        };
        (self.op)(&md.path, chunk.chunk_id, chunk.data, size, offset)
    }
    pub fn none() -> Self {
        #[allow(unused)]
        fn null_op(
            _path: &String,
            chunk_id: u64,
            data: *mut u8,
            size: u64,
            offset: u64,
        ) -> Result<i64, i32> {
            return Ok(0);
        }
        ChunkOp { op: null_op }
    }
}
