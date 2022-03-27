use std::hash::Hash;
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
impl PartialEq for SFSEndpoint {
    fn eq(&self, other: &Self) -> bool {
        self.addr.eq(&other.addr)
    }
}
impl Eq for SFSEndpoint {}
impl Hash for SFSEndpoint {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.addr.hash(state);
    }
}
