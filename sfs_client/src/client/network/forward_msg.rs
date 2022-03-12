use std::collections::HashMap;
use std::ffi::CStr;
use std::io::Error;
use std::sync::{Arc, Mutex};

use libc::{c_char, strncpy};

use crate::client::client_openfile::OpenFile;
use crate::client::client_util::{offset_to_chunk_id, chunk_lpad, chunk_rpad};
use crate::client::{client_context::ClientContext, network::network_service::NetworkService};
use crate::client::network::network_service::*;
use crate::global::distributor::Distributor;
use crate::global::error_msg::error_msg;
use crate::global::fsconfig::SFSConfig;
use crate::global::network::config::CHUNK_SIZE;
use crate::global::network::forward_data::{WriteData, ReadData, ReadResult};
use crate::global::network::post::PostOption;


pub struct ChunkStat{
    pub chunk_size: u64,
    pub chunk_total: u64,
    pub chunk_free: u64
}

pub fn forward_stat(path: &String) -> Result<String, Error>{

    let endp_id = ClientContext::get_instance().get_distributor().lock().unwrap().locate_file_metadata(path);
    let post_res = NetworkService::get_instance().post(ClientContext::get_instance().get_hosts().get(endp_id as usize).unwrap(), path, PostOption::Stat);
    if let Err(e) = post_res{
        error_msg("client::network::forward_stat".to_string(), format!("error {} occurs while fetching file stat", e));
        return Err(e);
    }

    todo!()
}
pub fn forward_create(path: &String, mode: u32) -> Result<String, Error>{

    let endp_id = ClientContext::get_instance().get_distributor().lock().unwrap().locate_file_metadata(path);
    let post_res = NetworkService::get_instance().post(ClientContext::get_instance().get_hosts().get(endp_id as usize).unwrap(), path, PostOption::Create);
    if let Err(e) = post_res{
        error_msg("client::network::forward_create".to_string(), format!("error {} occurs while fetching file stat", e));
        return Err(e);
    }

    todo!()
}
pub fn forward_remove(path: &String, remove_metadentry_only: bool, size: i64) -> Result<String, Error>{
    if remove_metadentry_only{
        let endp_id = ClientContext::get_instance().get_distributor().lock().unwrap().locate_file_metadata(path);
        let post_res = NetworkService::get_instance().post(ClientContext::get_instance().get_hosts().get(endp_id as usize).unwrap(), path, PostOption::Remove);
        todo!()
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
pub fn forward_write(path: &String, buf: * const c_char, append_flag: bool, in_offset: i64, write_size: i64, updated_metadentry_size: i64) -> (i32, i64){
    if write_size < 0 {
        return (-1, 0);
    }
    let offset = if append_flag { in_offset } else {updated_metadentry_size - write_size};
    let chunk_start = offset_to_chunk_id(offset.clone(), CHUNK_SIZE);
    let chunk_end = offset_to_chunk_id(offset + write_size - 1, CHUNK_SIZE);
    let mut target_chunks: HashMap<u64,Vec<u64>> = HashMap::new();
    let mut targets: Vec<u64> = Vec::new();

    let mut chunk_start_target = 0;
    let mut chunk_end_target = 0;
    for chunk_id in chunk_start..(chunk_end + 1){
        let target = ClientContext::get_instance().get_distributor().lock().unwrap().locate_data(path, chunk_id);
        if !target_chunks.contains_key(&target){
            target_chunks.insert(target, Vec::new());
            target_chunks.get_mut(&target).unwrap().push(chunk_id);
            targets.push(target);
        }
        else{
            target_chunks.get_mut(&target).unwrap().push(chunk_id);
        }
        if chunk_id == chunk_start{
            chunk_start_target = target;
        }
        if chunk_id == chunk_end{
            chunk_end_target = target;
        }
    }
    let mut tot_write = 0;
    for target in targets{
        let mut tot_chunk_size = target_chunks.get(&target).unwrap().len() as u64 * CHUNK_SIZE;
        if target == chunk_start_target{
            tot_chunk_size -= chunk_lpad(offset, CHUNK_SIZE);
        }
        if target == chunk_end_target{
            tot_chunk_size -= chunk_rpad(offset + write_size, CHUNK_SIZE);
        }
        
        let input = WriteData{
            path: path.clone(),
            offset: chunk_lpad(offset, CHUNK_SIZE) as i64,
            host_id: target,
            host_size: ClientContext::get_instance().get_hosts().len() as u64,
            chunk_n: target_chunks.get(&target).unwrap().len() as u64,
            chunk_start: chunk_start,
            chunk_end: chunk_end,
            total_chunk_size: tot_chunk_size,
            buffers: unsafe { CStr::from_ptr(buf).to_string_lossy().into_owned() }
        };
        if let Ok(p) = NetworkService::get_instance().post::<WriteData>(ClientContext::get_instance().get_hosts().get(target as usize).unwrap(), input, PostOption::Write){
            if p.err{
                return (-1, 0);
            }
            tot_write += p.data.as_str().parse::<i64>().expect("response should be 'i64'");
        }
        else{
            return (-1, 0);
        }
    }
    return (0, tot_write);
}
pub fn forward_read(path: &String, buf: * mut c_char, offset: i64, read_size: i64) -> (i32, u64){
    let chunk_start = offset_to_chunk_id(offset, CHUNK_SIZE);
    let chunk_end = offset_to_chunk_id(offset + read_size - 1, CHUNK_SIZE);
    let mut target_chunks: HashMap<u64,Vec<u64>> = HashMap::new();
    let mut targets: Vec<u64> = Vec::new();

    let mut chunk_start_target = 0;
    let mut chunk_end_target = 0;
    for chunk_id in chunk_start..(chunk_end + 1){
        let target = ClientContext::get_instance().get_distributor().lock().unwrap().locate_data(path, chunk_id);
        if !target_chunks.contains_key(&target){
            target_chunks.insert(target, Vec::new());
            target_chunks.get_mut(&target).unwrap().push(chunk_id);
            targets.push(target);
        }
        else{
            target_chunks.get_mut(&target).unwrap().push(chunk_id);
        }
        if chunk_id == chunk_start{
            chunk_start_target = target;
        }
        if chunk_id == chunk_end{
            chunk_end_target = target;
        }
    }
    
    let mut tot_read = 0;
    for target in targets{
        let mut tot_chunk_size = target_chunks.get(&target).unwrap().len() as u64 * CHUNK_SIZE;
        if target == chunk_start_target{
            tot_chunk_size -= chunk_lpad(offset, CHUNK_SIZE);
        }
        if target == chunk_end_target{
            tot_chunk_size -= chunk_rpad(offset + read_size, CHUNK_SIZE);
        }
        
        let input = ReadData{
            path: path.clone(),
            offset: chunk_lpad(offset, CHUNK_SIZE) as i64,
            host_id: target,
            host_size: ClientContext::get_instance().get_hosts().len() as u64,
            chunk_n: target_chunks.get(&target).unwrap().len() as u64,
            chunk_start: chunk_start,
            chunk_end: chunk_end,
            total_chunk_size: tot_chunk_size
        };
        println!("{:?}", input);
        if let Ok(p) = NetworkService::get_instance().post::<ReadData>(ClientContext::get_instance().get_hosts().get(target as usize).unwrap(), input, PostOption::Read){
            if p.err{
                return (-1, 0);
            }
            let read_res: ReadResult = serde_json::from_str(p.data.as_str()).unwrap();
            tot_read += read_res.nreads;
            let data = read_res.data;
            for chnk in data{     
                let local_offset = if chnk.0 == chunk_start {0} else {chnk.0 * CHUNK_SIZE - (offset as u64 % CHUNK_SIZE)};
                unsafe{
                    strncpy(buf.offset(local_offset as isize), chnk.1.as_ptr() as *const i8, chnk.1.len());
                }
            }
        }
        else{
            return (-1, 0);
        }
    }
    return (0, tot_read);
}
pub fn forward_get_dirents(path: &String) -> (i32, Arc<Mutex<OpenFile>>){
    todo!();
}
pub fn forward_get_fs_config() -> bool{
    if let Ok(handle) = NetworkService::get_instance().post::<>(ClientContext::get_instance().get_hosts().get(ClientContext::get_instance().get_local_host_id() as usize).unwrap(), (), PostOption::FsConfig){
        let out: String = "".to_string();
        let config: SFSConfig = serde_json::from_str(out.as_str()).unwrap();
        ClientContext::get_instance().set_mountdir(config.mountdir.clone());
        ClientContext::get_instance().set_fsconfig(config);
        return true;
    }
    else{
        return false;
    }
}