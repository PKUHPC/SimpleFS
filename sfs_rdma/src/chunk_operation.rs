use crate::CHUNK_SIZE;
pub fn offset_to_chunk_id(offset: i64, chunk_size: u64) -> u64 {
    //(chunk_align_down(offset, chunk_size) >> ((chunk_size as f64).log2() as i64)) as u64
    offset as u64 / chunk_size
}

pub struct ChunkInfo{
    pub chunk_id: u64,
    pub data: *mut u8,
}
pub struct ChunkOp{
    pub path: String,
    pub offset: u64,
    pub size: u64,
    pub op: fn(&String, u64, &[u8], u64, u64) -> Result<i64, i32>
}
impl ChunkOp{
    pub fn submit(&self, chunk: ChunkInfo) -> Result<i64, i32> {
        let size = u64::min(CHUNK_SIZE, self.size - chunk.chunk_id * CHUNK_SIZE);
        let offset = if chunk.chunk_id == offset_to_chunk_id(self.offset.try_into().unwrap(), CHUNK_SIZE) {self.offset % CHUNK_SIZE} else {0};
        let data = unsafe{std::slice::from_raw_parts(chunk.data as *const u8, size as usize)};
        (self.op)(&self.path, chunk.chunk_id, data, size, offset)
    }
}