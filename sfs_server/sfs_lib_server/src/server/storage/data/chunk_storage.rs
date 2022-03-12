use std::os::unix::prelude::FileExt;
use std::path::Path;
use std::{fs, path};
use std::os::unix::fs::PermissionsExt;

use nix::sys::statfs::statfs;
use std::sync::{MutexGuard, Mutex};

use crate::global::metadata;
use crate::global::network::config::CHUNK_SIZE;
use crate::global::{util::path_util::is_absolute, error_msg::error_msg};

use lazy_static::*;

pub struct ChunkStat{
    pub chunk_size: u64,
    pub chunk_total: u64,
    pub chunk_free: u64
}
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
    pub fn absolute(&self, internel_path: &String) -> String{
        if is_absolute(&internel_path) {
            error_msg("server::storage::chunk_storage::absolute".to_string(), "path should be relative".to_string());
            return internel_path.clone();
        }
        format!("{}/{}", self.root_path_, internel_path)
    }
    pub fn get_chunks_dir(file_path: &String) -> String{
        if !is_absolute(&file_path) {
            error_msg("server::storage::chunk_storage::get_chunks_dir".to_string(), "path should be absolute".to_string());
            return file_path.replace("/", ":");
        }
        let chunk_dir = file_path[1..].to_string();
        chunk_dir.replace("/", ":")
    }
    pub fn get_chunks_path(file_path: &String, chunk_id: u64) -> String{
        format!("{}/{}", ChunkStorage::get_chunks_dir(file_path), chunk_id)
    }
    pub fn init_chunk_space(&self, file_path: &String){
        let chunk_dir = self.absolute(&ChunkStorage::get_chunks_dir(file_path));
        let path = Path::new(&chunk_dir);
        if path.exists(){
            return;
        }
        if let Err(e) = fs::create_dir_all(path){
            error_msg("server::storage::chunk_storage::init_chunk_space".to_string(), "fail to create chunk directory".to_string());
        }
    }

    pub fn new(path: &String, chunk_size: u64) -> Option<ChunkStorage>{
        if !is_absolute(&path) {
            error_msg("server::storage::chunk_storage::new".to_string(), "path should be absolute".to_string());
            return None;
        }
        let perm = fs::metadata(path).unwrap().permissions();
        let mode: u32 = perm.mode();
        if mode & metadata::S_IRUSR == 0 || mode & metadata::S_IWUSR == 0{
            error_msg("server::storage::chunk_storage::new".to_string(), "can't create chunk storage with enough permissions".to_string());
        }
        Some(ChunkStorage{
            root_path_: path.clone(),
            chunk_size_: chunk_size
        })
    }
    pub fn destroy_chunk_space(&self, file_path: String){
        let chunk_dir = self.absolute(&ChunkStorage::get_chunks_dir(&file_path));
        if let Err(e) = fs::remove_dir_all(path::Path::new(&chunk_dir)){
            error_msg("server::storage::chunk_storage::destroy_chunk_space".to_string(), "fail to remove chunk directory".to_string());
        }
    }
    pub fn write_chunk(&self, file_path: &String, chunk_id: u64, buf: &[u8], size: u64, offset: u64) -> Result<u64, i32>{
        if size+ offset > self.chunk_size_{
            error_msg("server::storage::chunk_storage::write_chunk".to_string(), "beyond chunk storage range".to_string());
        }
        self.init_chunk_space(&file_path);
        let chunk_path = self.absolute(&ChunkStorage::get_chunks_path(&file_path, chunk_id));
        let f = fs::OpenOptions::new().create(true).write(true).read(true).open(chunk_path).unwrap();
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
    pub fn read_chunk(&self, file_path: &String, chunk_id: u64, mut buf: &mut [u8], size: u64, mut offset: u64) -> Result<u64, i32>{
        if size + offset > self.chunk_size_{
            error_msg("server::storage::chunk_storage::read_chunk".to_string(), "beyond chunk storage range".to_string());
        }
        self.init_chunk_space(&file_path);
        let chunk_path = self.absolute(&ChunkStorage::get_chunks_path(&file_path, chunk_id));
        let open_res = fs::OpenOptions::new().write(true).read(true).open(chunk_path);
        if let Err(e) = open_res{
            error_msg("server::storage::chunk_storage::read_chunk".to_string(), "fail to open chunk file".to_string());
            return Err(-1);
        }
        let f = open_res.unwrap();
        let mut read_tot:u64 = 0;
        let tmp = buf;
        buf = &mut tmp[0..size as usize];
        while !buf.is_empty() {
            match f.read_at(buf, offset) {
                Ok(0) => break,
                Ok(n) => {
                    let tmp = buf;
                    buf = &mut tmp[n..size as usize];
                    offset += n as u64;
                    read_tot += n as u64;
                },
                Err(e) =>{
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
    pub fn trim_chunk_space(&self, file_path: &String, chunk_start: u64){
        let chunk_dir = self.absolute(&ChunkStorage::get_chunks_dir(&file_path));
        let dir_iter = fs::read_dir(Path::new(&chunk_dir)).unwrap();
        let mut err = false;
        for entry in dir_iter{
            let entry = entry.unwrap();
            let chunk_path = entry.path();
            let chunk_id = chunk_path.file_name().unwrap().to_str().unwrap().parse::<u64>().unwrap();
            if chunk_id > chunk_start{
                if let Err(e) = fs::remove_file(chunk_path.as_path()){
                    error_msg("server::storage::chunk_storage::trim_chunk_space".to_string(), "fail to remove file".to_string());
                    err = true;
                }
            }
        }
        if err{
            error_msg("server::storage::chunk_storage::trim_chunk_space".to_string(), "error occurs while truncating".to_string());
        }
    }
    pub fn truncate_chunk_file(&self, file_path: &String, chunk_id: u64, length: u64){
        if length > self.chunk_size_{
            error_msg("server::storage::chunk_storage::truncate_chunk_file".to_string(), "invalid length".to_string());
            return;
        }
        let chunk_path = self.absolute(&ChunkStorage::get_chunks_path(file_path, chunk_id));
        let f = fs::OpenOptions::new().write(true).read(true).open(Path::new(&chunk_path)).unwrap();
        if let Err(e) = f.set_len(length){
            error_msg("server::storage::chunk_storage::truncate_chunk_file".to_string(), "error occurs while truncating chunk file".to_string());
        }
        
    }
    pub fn chunk_stat(&self) -> ChunkStat{
        let statfs = statfs(Path::new(&self.root_path_));
        if let Err(e) = statfs{
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
            chunk_size: self.chunk_size_,
            chunk_total: bytes_tot / self.chunk_size_,
            chunk_free: bytes_free / self.chunk_size_
        }
    }
}