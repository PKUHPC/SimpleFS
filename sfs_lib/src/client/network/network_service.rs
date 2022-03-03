use lazy_static::*;
use std::sync::{Mutex, Arc, MutexGuard};

use crate::client::client_endpoint::SFSEndpoint;
pub enum PostOption {
    Stat,
    Create,
    Remove
}
pub struct PostResult{
    data: String,
    err: i32
}
pub struct NetworkService{

}
lazy_static!{
    static ref NTS: Mutex<NetworkService> = Mutex::new(
        NetworkService{}
    );
}
impl NetworkService{
    pub fn get_instance() -> MutexGuard<'static, NetworkService>{
        NTS.lock().unwrap()
    }
    pub fn post(&self, endp: &SFSEndpoint, path: &String, opt: PostOption) -> Result<String, i32>{
        todo!()
    }
}