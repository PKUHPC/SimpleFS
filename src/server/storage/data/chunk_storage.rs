use std::{fs, path};
use std::os::unix::fs::PermissionsExt;

use crate::global::metadata;
use crate::global::{util::path_util::is_absolute, error_msg::error_msg};

pub struct ChunkStat{
    chunk_size: u64,
    chunk_total: u64,
    chunk_free: u64
}
pub struct ChunkStorage{
    root_path_: String,
    chunk_size_: u64,
}
impl ChunkStorage{
    fn absolute(&self, internel_path: &String) -> String{
        if is_absolute(&internel_path) {
            error_msg("server::storage::chunk_storage::absolute".to_string(), "path should be relative".to_string());
            return internel_path.clone();
        }
        format!("{}/{}", self.root_path_, internel_path)
    }
    fn get_chunks_dir(file_path: &String) -> String{
        if !is_absolute(&file_path) {
            error_msg("server::storage::chunk_storage::get_chunks_dir".to_string(), "path should be absolute".to_string());
            return file_path.replace("/", ":");
        }
        let chunk_dir = file_path[1..].to_string();
        chunk_dir.replace("/", ":")
    }
    fn get_chunks_path(file_path: &String, chunk_id: u64) -> String{
        format!("{}/{}", ChunkStorage::get_chunks_dir(file_path), chunk_id)
    }
    fn init_chunk_space(&self, file_path: &String){
        let chunk_dir = self.absolute(&ChunkStorage::get_chunks_dir(file_path));
        if let Err(e) = fs::create_dir(chunk_dir){
            error_msg("server::storage::chunk_storage::init_chunk_space".to_string(), "fail to create chunk directory".to_string());
        }
    }

    pub fn new(path: &String, chunk_size: u64) -> Option<ChunkStorage>{
        if !is_absolute(&path) {
            error_msg("server::storage::chunk_storage::new".to_string(), "path should be absolute".to_string());
            return None;
        }
        let perm = fs::metadata(path).unwrap().permissions();
        let mode: i32 = perm.mode();
        if perm & metadata::S_IRUSR == 0 || perm & metadata::S_IWUSR == 0{
            error_msg("server::storage::chunk_storage::new".to_string(), "can't create chunk storage with enough permissions".to_string());
        }
        Some(ChunkStorage{
            root_path_: path.clone(),
            chunk_size_: chunk_size
        })
    }
    pub fn destroy_chunk_space(&self, file_path: String){

    }
    pub fn write_chunk(&self, file_path: &String, chunk_id: u64, buf: &mut [u8], size: u64, offset: i64) -> u64{
        todo!()
    }
    pub fn read_chunk(&self, file_path: &String, chunk_id: u64, buf: &mut [u8], size: u64, offset: i64) -> u64{
        todo!()
    }
    pub fn trim_chunk_space(&self, file_path: &String, chunk_start: u64){

    }
    pub fn truncate_chunk_file(&self, file_path: &String, chunk_id: u64, length: u64){
        
    }
    pub fn chunk_stat() -> ChunkStat{
        todo!()
    }
}