#![allow(dead_code)]
pub mod global;
pub mod client;

#[cfg(test)]
mod tests {
    use libc::c_char;

    use crate::{global::{network::{forward_data::WriteData, post::{PostOption, PostResult}}, distributor::SimpleHashDistributor}, client::{client_endpoint::SFSEndpoint, network::{network_service::NetworkService, forward_msg::{forward_write, forward_read}}, client_context::ClientContext}};

    #[test]
    pub fn test(){
        let s = "bybchuicbahbcashbadhasuhdadioada".to_string();
        let path = "/sfs/test/async_write/a".to_string();
        /*
        let input = WriteData{
            path: path,
            offset: 0,
            host_id: 0,
            host_size: 1,
            chunk_n: 1,
            chunk_start: 0,
            chunk_end: 0,
            total_chunk_size: s.len() as u64,
            buffers: s
        };
        match NetworkService::get_instance().post::<WriteData>(&endp, input, PostOption::Write){
            Err(e) => {println!("error: {}", e.to_string());},
            Ok(res) => {println!("data: {}", res.data);}
        };
        */
        let distributor = SimpleHashDistributor::new(1, 1);
        ClientContext::get_instance().set_distributor(distributor);
        
        let endp = SFSEndpoint{
            addr: "127.0.0.1".to_string(),
        };
        ClientContext::get_instance().set_hosts(vec![endp; 1]);


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
    }
}
