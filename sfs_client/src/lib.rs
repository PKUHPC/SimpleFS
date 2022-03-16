#![allow(dead_code)]
pub mod global;
pub mod client;

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use libc::{c_char, O_RDWR, O_CREAT, O_RDONLY, S_IFREG};

    use crate::{global::{network::{forward_data::WriteData, post::{PostOption, PostResult}}, distributor::SimpleHashDistributor}, client::{client_endpoint::SFSEndpoint, network::{network_service::NetworkService, forward_msg::{forward_write, forward_read}}, client_context::ClientContext, client_syscall::sfs_open, client_openfile::{OpenFile, FileType}}};

    #[test]
    pub fn test(){
        let s = "bybchuicbahbcashbadhasuhdadioada".to_string();
        let path = "/sfs/test/async_write/a".to_string();

        let distributor = SimpleHashDistributor::new(1, 1);
        ClientContext::get_instance().set_distributor(distributor);
        
        let endp = SFSEndpoint{
            addr: "127.0.0.1".to_string(),
        };
        ClientContext::get_instance().set_hosts(vec![endp; 1]);


        let res = sfs_open(path.as_str().as_ptr() as * const c_char, S_IFREG, O_CREAT | O_RDONLY);
        println!("open result: {}", res);
        println!("ofm length: {}", ClientContext::get_instance().get_ofm().lock().unwrap().get_length());
        /*
        let res = forward_write(&path, s.as_bytes().as_ptr() as * const c_char, true, 6, s.len() as i64, s.len() as i64);
        if res.0 != 0{
            println!("error ...");
        }
        else{
            println!("{} bytes written ...", res.1);
        }


        let mut buf = [0 as u8; 50];
        let res = forward_read(&path, buf.as_mut_ptr() as * mut i8, 0, 32);
        if res.0 != 0{
            println!("error ...");
        }
        else{
            println!("read: {}", String::from_utf8(buf.to_vec()).unwrap());
        }
         */
    }
}
