use lazy_static::*;
use serde::{Serialize, Deserialize};
use std::{sync::{Mutex, Arc, MutexGuard}, net::{TcpStream, TcpListener}, io::{Error, Write}};

use crate::{client::client_endpoint::SFSEndpoint, global::network::post::{PostOption, Post}};

use super::handle::ClientHandle;
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
    pub fn post<T: Serialize>(&self, endp: &SFSEndpoint, data: T, opt: PostOption) -> Result<ClientHandle, Error>{
        let mut stream = TcpStream::connect(&endp.addr)?;
        let serialized_data = serde_json::to_string(&data)?;
        let post = Post{
            option: opt.clone(),
            data: serialized_data
        };
        let buf = serde_json::to_string(&post)?;
        stream.write(buf.as_bytes()).expect("Failed to write to stream");
        Ok(ClientHandle{
            op: opt,
            err: 0,
            socket: stream,
            nreads: 0,
        })
    }
}