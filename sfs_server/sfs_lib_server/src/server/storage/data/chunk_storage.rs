use std::os::unix::prelude::FileExt;
use std::path::Path;
use tokio::fs;
use std::os::unix::fs::PermissionsExt;

use libc::{S_IRUSR, S_IWUSR};
use nix::sys::statfs::statfs;
use std::sync::{MutexGuard, Mutex};

use crate::global::network::config::CHUNK_SIZE;
use crate::global::network::forward_data::ChunkStat;
use crate::global::{util::path_util::is_absolute, error_msg::error_msg};

use lazy_static::*;

pub struct ChunkStorage{
    pub root_path_: String,
    pub chunk_size_: u64,
}
lazy_static!{
    static ref CNK: Mutex<ChunkStorage> = Mutex::new(ChunkStorage{
        root_path_: "".to_string(),
        chunk_size_: CHUNK_SIZE
    });
}
impl ChunkStorage{
    pub fn get_instance() -> MutexGuard<'static, ChunkStorage>{
        CNK.lock().unwrap()
    }
    pub fn set_storage(storage_: ChunkStorage){
        CNK.lock().unwrap().root_path_ = storage_.root_path_;
        CNK.lock().unwrap().chunk_size_ = storage_.chunk_size_;
    }
    pub fn absolute(internel_path: &String) -> String{
        if is_absolute(&internel_path) {
            error_msg("server::storage::chunk_storage::absolute".to_string(), "path should be relative".to_string());
            return internel_path.clone();
        }
        format!("{}/{}", CNK.lock().unwrap().get_root_path(), internel_path)
    }
    pub fn get_chunks_dir(file_path: &String) -> String{
        if !is_absolute(file_path) {
            error_msg("server::storage::chunk_storage::get_chunks_dir".to_string(), "path should be absolute".to_string());
            return file_path.replace("/", ":");
        }
        let chunk_dir = file_path[1..].to_string();
        chunk_dir.replace("/", ":")
    }
    pub fn get_chunks_path(file_path: &String, chunk_id: u64) -> String{
        format!("{}/{}", ChunkStorage::get_chunks_dir(file_path), chunk_id)
    }
    pub async fn init_chunk_space(file_path: &String){
        let chunk_dir = ChunkStorage::absolute(&ChunkStorage::get_chunks_dir(file_path));
        let path = Path::new(&chunk_dir);
        if path.exists(){
            return;
        }
        if let Err(_e) = fs::create_dir_all(path).await{
            error_msg("server::storage::chunk_storage::init_chunk_space".to_string(), "fail to create chunk directory".to_string());
        }
    }

    pub fn new(path: &String, chunk_size: u64) -> Option<ChunkStorage>{
        if !is_absolute(&path) {
            error_msg("server::storage::chunk_storage::new".to_string(), "path should be absolute".to_string());
            return None;
        }
        let perm = std::fs::metadata(path).unwrap().permissions();
        let mode: u32 = perm.mode();
        if mode & S_IRUSR == 0 || mode & S_IWUSR == 0{
            error_msg("server::storage::chunk_storage::new".to_string(), "can't create chunk storage with enough permissions".to_string());
        }
        Some(ChunkStorage{
            root_path_: path.clone(),
            chunk_size_: chunk_size
        })
    }
    pub async fn destroy_chunk_space(file_path: &String){
        let chunk_dir = ChunkStorage::absolute(&ChunkStorage::get_chunks_dir(file_path));
        if let Err(_e) = fs::remove_dir_all(Path::new(&chunk_dir)).await{
            error_msg("server::storage::chunk_storage::destroy_chunk_space".to_string(), "fail to remove chunk directory".to_string());
        }
    }
    pub async fn write_chunk(file_path: &String, chunk_id: u64, buf: &[u8], size: u64, offset: u64) -> Result<u64, i32>{
        if size+ offset > CNK.lock().unwrap().get_chunk_size(){
            error_msg("server::storage::chunk_storage::write_chunk".to_string(), "beyond chunk storage range".to_string());
        }
        ChunkStorage::init_chunk_space(file_path).await;
        let chunk_path = ChunkStorage::absolute(&ChunkStorage::get_chunks_path(file_path, chunk_id));
        let f = std::fs::OpenOptions::new().create(true).write(true).read(true).open(chunk_path).unwrap();
        let mut wrote_tot:u64 = 0;
        while wrote_tot != size{
            if let Ok(bytes) = f.write_at(&buf[wrote_tot as usize..size as usize], offset + wrote_tot){
                wrote_tot += bytes as u64;
            }
            else{
                error_msg("server::storage::chunk_storage::write_chunk".to_string(), "error occured while writing to chunk".to_string());
                return Err(-1);
            }
        }
        Ok(wrote_tot)
    }
    pub async fn read_chunk(file_path: &String, chunk_id: u64, buf: &mut Vec<u8>, size: u64, mut offset: u64) -> Result<u64, i32>{
        if size + offset > CNK.lock().unwrap().get_chunk_size(){
            error_msg("server::storage::chunk_storage::read_chunk".to_string(), "beyond chunk storage range".to_string());
        }
        ChunkStorage::init_chunk_space(file_path).await;
        let chunk_path = ChunkStorage::absolute(&ChunkStorage::get_chunks_path(file_path, chunk_id));
        let open_res = std::fs::OpenOptions::new().write(true).read(true).open(chunk_path);
        if let Err(_e) = open_res{
            error_msg("server::storage::chunk_storage::read_chunk".to_string(), "fail to open chunk file".to_string());
            return Err(-1);
        }
        let f = open_res.unwrap();
        let mut read_tot:u64 = 0;
        let tmp = buf;
        let mut buf = &mut tmp[0..size as usize];
        while !buf.is_empty() {
            match f.read_at(buf, offset) {
                Ok(0) => break,
                Ok(n) => {
                    let tmp = buf;
                    buf = &mut tmp[n..size as usize];
                    offset += n as u64;
                    read_tot += n as u64;
                },
                Err(_e) =>{
                    error_msg("server::storage::chunk_storage::read_chunk".to_string(), "error occurs while reading from chunks".to_string());
                    return Err(-1);
                },
            }
        }
        if !buf.is_empty() {
            error_msg("server::storage::chunk_storage::read_chunk".to_string(), "unable to fill the buf because of reaching EOF".to_string());
            Ok(read_tot)
        } else {
            Ok(read_tot)
        }
    }
    pub async fn trim_chunk_space(file_path: &String, chunk_start: u64){
        let chunk_dir = ChunkStorage::absolute(&ChunkStorage::get_chunks_dir(file_path));
        let dir_iter = std::fs::read_dir(Path::new(&chunk_dir)).unwrap();
        let err = false;
        for entry in dir_iter{
            let entry = entry.unwrap();
            let chunk_path = entry.path();
            let chunk_id = chunk_path.file_name().unwrap().to_str().unwrap().parse::<u64>().unwrap();
            if chunk_id >= chunk_start{
                if let Err(_e) = fs::remove_file(chunk_path.as_path()).await{
                    continue;
                    //error_msg("server::storage::chunk_storage::trim_chunk_space".to_string(), "fail to remove file".to_string());
                    //err = true;
                }
            }
        }
        if err{
            error_msg("server::storage::chunk_storage::trim_chunk_space".to_string(), "error occurs while truncating".to_string());
        }
    }
    pub async fn truncate_chunk_file(file_path: &String, chunk_id: u64, length: u64){
        if length > CNK.lock().unwrap().get_chunk_size(){
            error_msg("server::storage::chunk_storage::truncate_chunk_file".to_string(), "invalid length".to_string());
            return;
        }
        let chunk_path = ChunkStorage::absolute(&ChunkStorage::get_chunks_path(file_path, chunk_id));
        let f_res = fs::OpenOptions::new().write(true).read(true).open(Path::new(&chunk_path)).await;
        if let Err(_e) = f_res{
            return;
        }
        let f = f_res.unwrap();
        if let Err(_e) = f.set_len(length).await{
            error_msg("server::storage::chunk_storage::truncate_chunk_file".to_string(), "error occurs while truncating chunk file".to_string());
        }
        
    }
    pub fn chunk_stat() -> ChunkStat{
        let statfs = statfs(Path::new(CNK.lock().unwrap().get_root_path()));
        if let Err(_e) = statfs{
            error_msg("server::storage::chunk_storage::chunk_stat".to_string(), "error occurs while get fs stat".to_string());
            return ChunkStat{
                chunk_size: 0,
                chunk_total: 0,
                chunk_free: 0,
            }
        }
        let statfs = statfs.unwrap();
        let bytes_tot = statfs.block_size() as u64 * statfs.blocks();
        let bytes_free = statfs.block_size() as u64 * statfs.blocks_available();
        ChunkStat{
            chunk_size: CNK.lock().unwrap().get_chunk_size(),
            chunk_total: bytes_tot / CNK.lock().unwrap().get_chunk_size(),
            chunk_free: bytes_free / CNK.lock().unwrap().get_chunk_size()
        }
    }
    pub fn get_chunk_size(&self) -> u64{
        self.chunk_size_
    }
    pub fn get_root_path(&self) -> &String{
        &self.root_path_
    }
}