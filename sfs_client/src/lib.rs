#![allow(dead_code)]
pub mod global;
pub mod client;

#[cfg(test)]
mod tests {
    use crate::{global::network::{forward_data::WriteData, post::{PostOption, PostResult}}, client::{client_endpoint::SFSEndpoint, network::network_service::NetworkService}};

    #[test]
    pub fn test(){
        let s = "bybchuicbahbcashbadhasuhdadioada".to_string();
        let path = "/sfs/test/async_write/a".to_string();
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
        let endp = SFSEndpoint{
            addr: "127.0.0.1".to_string(),
        };
        match NetworkService::get_instance().post::<WriteData>(&endp, input, PostOption::Write){
            Err(e) => {println!("error: {}", e.to_string());},
            Ok(res) => {println!("data: {}", res.data);}
        };
    }
}
