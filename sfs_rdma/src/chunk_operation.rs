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
    pub chunk_start: u64,
    pub size: u64,
    pub op: fn(&String, u64, *mut u8, u64, u64) -> Result<i64, i32>
}
impl ChunkOp{
    pub fn submit(&self, chunk: ChunkInfo) -> Result<i64, i32> {
        let buffer_offset = if chunk.chunk_id == self.chunk_start {0} else {CHUNK_SIZE * (chunk.chunk_id  - self.chunk_start) - self.offset};
        let size = if chunk.chunk_id == self.chunk_start {u64::min(CHUNK_SIZE - self.offset, self.size)} else {u64::min(CHUNK_SIZE, self.size - buffer_offset)};
        //println!("{} - {}: {} {} | {} {}", self.chunk_start, chunk.chunk_id, self.offset, self.size, buffer_offset, size);
        let offset = if chunk.chunk_id == self.chunk_start {self.offset} else {0};
        (self.op)(&self.path, chunk.chunk_id, chunk.data, size, offset)
    }
    pub fn none() -> Self{
        #[allow(unused)]
        fn null_op(_path: &String, chunk_id: u64, data: *mut u8, size: u64, offset: u64) -> Result<i64, i32>{
            return Ok(0);
        }
        ChunkOp { path: "".to_string(), offset: 0, chunk_start: 0, size: 0, op: null_op }
    }
}