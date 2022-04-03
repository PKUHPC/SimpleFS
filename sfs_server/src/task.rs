#[derive(Debug)]
pub struct WriteChunkTask {
    pub path: String,
    pub buf: String,
    pub chunk_id: u64,
    pub size: u64,
    pub offset: u64,
}

#[derive(Debug)]
pub struct ReadChunkTask {
    pub path: String,
    pub chunk_id: u64,
    pub size: u64,
    pub offset: u64,
}
impl Clone for ReadChunkTask {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            chunk_id: self.chunk_id.clone(),
            size: self.size.clone(),
            offset: self.offset.clone(),
        }
    }
}
impl ReadChunkTask {
    pub fn new() -> ReadChunkTask {
        ReadChunkTask {
            path: "".to_string(),
            chunk_id: 0,
            size: 0,
            offset: 0,
        }
    }
}
