#[derive(Debug)]
pub struct WriteChunkTask{
    pub path: String,
    pub buf: String,
    pub chunk_id: u64,
    pub size: u64,
    pub offset: u64
}
impl Clone for WriteChunkTask{
    fn clone(&self) -> Self {
        Self { path: self.path.clone(), buf: self.buf.clone(), chunk_id: self.chunk_id.clone(), size: self.size.clone(), offset: self.offset.clone() }
    }
}
impl WriteChunkTask{
    pub fn new() -> WriteChunkTask{
        WriteChunkTask{
            path: "".to_string(),
            buf: "".to_string(),
            chunk_id: 0,
            size: 0,
            offset: 0,
        }
    }
}