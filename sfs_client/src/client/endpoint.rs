#[derive(Debug)]
pub struct SFSEndpoint {
    pub addr: String,
}
impl Clone for SFSEndpoint {
    fn clone(&self) -> Self {
        Self {
            addr: self.addr.clone(),
        }
    }
}
impl SFSEndpoint {
    pub fn new() -> SFSEndpoint {
        SFSEndpoint {
            addr: "".to_string(),
        }
    }
}
