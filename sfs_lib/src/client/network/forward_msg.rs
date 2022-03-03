use std::sync::{Arc, Mutex};

use crate::client::client_openfile::OpenFile;
use crate::client::{client_context::ClientContext, client_distributor::Distributor, network::network_service::NetworkService};
use crate::client::network::network_service::*;
use crate::global::error_msg::error_msg;
use crate::server::storage::data::chunk_storage::ChunkStat;

pub fn forward_stat(path: &String) -> Result<String, i32>{
    let hosts = ClientContext::get_instance().get_hosts();

    let endp_id = ClientContext::get_instance().get_distributor().locate_file_metadata(path);
    let endp = hosts.get(endp_id as usize).unwrap();
    let post_res = NetworkService::get_instance().post(endp, path, PostOption::Stat);
    if let Err(e) = post_res{
        error_msg("client::network::forward_stat".to_string(), format!("error {} occurs while fetching file stat", e));
        return Err(e);
    }

    Ok(post_res.unwrap())
}
pub fn forward_create(path: &String, mode: u32) -> Result<String, i32>{
    let hosts = ClientContext::get_instance().get_hosts();

    let endp_id = ClientContext::get_instance().get_distributor().locate_file_metadata(path);
    let endp = hosts.get(endp_id as usize).unwrap();
    let post_res = NetworkService::get_instance().post(endp, path, PostOption::Create);
    if let Err(e) = post_res{
        error_msg("client::network::forward_create".to_string(), format!("error {} occurs while fetching file stat", e));
        return Err(e);
    }

    Ok(post_res.unwrap())
}
pub fn forward_remove(path: &String, remove_metadentry_only: bool, size: i64) -> Result<String, i32>{
    let hosts = ClientContext::get_instance().get_hosts();
    if remove_metadentry_only{
        let endp_id = ClientContext::get_instance().get_distributor().locate_file_metadata(path);
        let endp = hosts.get(endp_id as usize).unwrap();
        let post_res = NetworkService::get_instance().post(endp, path, PostOption::Remove);
        return post_res;
    }
    todo!()
}
pub fn forward_get_chunk_stat() -> (i32, ChunkStat){
    todo!();
}
pub fn forward_get_metadentry_size(path: &String) -> (i32, i64){
    todo!();
}
pub fn forward_get_decr_size(path: &String, new_size: i64) -> i32{
    todo!()
}
pub fn forward_truncate(path: &String, old_size: i64, new_size: i64) -> i32{
    todo!()
}
pub fn forward_update_metadentry_size(path: &String, size: i64, offset: i64, append_flag: bool) -> (i32, i64){
    todo!();
}
pub fn forward_write(path: &String, buf: * const char, append_flag: bool, in_offset: i64, write_size: i64, updated_metadentry_size: i64) -> (i32, i64){
    todo!();
}
pub fn forward_read(path: &String, buf: * mut char, offset: i64, read_size: i64) -> (i32, i64){
    todo!();
}
pub fn forward_get_dirents(path: &String) -> (i32, Arc<Mutex<OpenFile>>){
    todo!();
}