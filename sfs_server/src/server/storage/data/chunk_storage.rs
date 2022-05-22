use std::fs;
use std::os::unix::fs::PermissionsExt;
#[allow(unused)]
use std::os::unix::prelude::FileExt;
use std::path::Path;

use libc::{S_IRUSR, S_IWUSR};
use nix::sys::statfs::statfs;

use sfs_global::global::fsconfig::ENABLE_STUFFING;
use sfs_global::global::network::config::CHUNK_SIZE;
use sfs_global::global::network::forward_data::ChunkStat;
use sfs_global::global::util::path_util::is_absolute;

use lazy_static::*;

use crate::error_msg::error_msg;
use crate::server::config::{STUFF_WITH_ROCKSDB, TRUNCATE_DIRECTORY};
use crate::server::filesystem::storage_context::StorageContext;

use super::stuff_db::StuffDB;

#[allow(unused_must_use)]
pub fn init_chunk() -> ChunkStorage {
    let chunk_storage_path =
        StorageContext::get_instance().get_rootdir().clone() + &"/data/chunks".to_string();
    if TRUNCATE_DIRECTORY {
        std::fs::remove_dir_all(Path::new(&chunk_storage_path));
    }
    std::fs::create_dir_all(Path::new(&chunk_storage_path))
        .expect("fail to create chunk storage directory");

    return ChunkStorage::new(&chunk_storage_path, CHUNK_SIZE)
        .expect("fail to create chunk storage");
}
pub struct ChunkStorage {
    pub root_path_: String,
    pub chunk_size_: u64,
}
lazy_static! {
    static ref CNK: ChunkStorage = init_chunk();
}
impl ChunkStorage {
    pub fn get_instance() -> &'static ChunkStorage {
        &CNK
    }
    pub fn default() -> ChunkStorage {
        ChunkStorage {
            root_path_: "".to_string(),
            chunk_size_: CHUNK_SIZE,
        }
    }
    pub fn absolute(internel_path: &String) -> String {
        if is_absolute(&internel_path) {
            error_msg(
                "server::storage::chunk_storage::absolute".to_string(),
                "path should be relative".to_string(),
            );
            return internel_path.clone();
        }
        format!("{}/{}", CNK.get_root_path(), internel_path)
    }
    pub fn get_chunks_dir(file_path: &String) -> String {
        if !is_absolute(file_path) {
            error_msg(
                "server::storage::chunk_storage::get_chunks_dir".to_string(),
                "path should be absolute".to_string(),
            );
            return file_path.replace("/", ":");
        }
        let chunk_dir = file_path[1..].to_string();
        chunk_dir.replace("/", ":")
    }
    pub fn get_chunks_path(file_path: &String, chunk_id: u64) -> String {
        format!("{}/{}", ChunkStorage::get_chunks_dir(file_path), chunk_id)
    }
    pub fn init_chunk_space(file_path: &String) {
        let chunk_dir = ChunkStorage::absolute(&ChunkStorage::get_chunks_dir(file_path));
        let path = Path::new(&chunk_dir);
        if path.exists() {
            return;
        }
        if let Err(_e) = fs::create_dir_all(path) {
            error_msg(
                "server::storage::chunk_storage::init_chunk_space".to_string(),
                "fail to create chunk directory".to_string(),
            );
        }
    }

    pub fn new(path: &String, chunk_size: u64) -> Option<ChunkStorage> {
        if !is_absolute(&path) {
            error_msg(
                "server::storage::chunk_storage::new".to_string(),
                "path should be absolute".to_string(),
            );
            return None;
        }
        let perm = std::fs::metadata(path).unwrap().permissions();
        let mode: u32 = perm.mode();
        if mode & S_IRUSR == 0 || mode & S_IWUSR == 0 {
            error_msg(
                "server::storage::chunk_storage::new".to_string(),
                "can't create chunk storage with enough permissions".to_string(),
            );
        }
        Some(ChunkStorage {
            root_path_: path.clone(),
            chunk_size_: chunk_size,
        })
    }
    pub fn destroy_chunk_space(file_path: &String) {
        if ENABLE_STUFFING && STUFF_WITH_ROCKSDB {
            StuffDB::get_instance().remove(file_path);
        }
        let chunk_dir = ChunkStorage::absolute(&ChunkStorage::get_chunks_dir(file_path));
        if let Err(_e) = fs::remove_dir_all(Path::new(&chunk_dir)) {
            error_msg(
                "server::storage::chunk_storage::destroy_chunk_space".to_string(),
                "fail to remove chunk directory".to_string(),
            );
        }
    }
    pub fn write_chunk(
        file_path: &String,
        chunk_id: u64,
        buf: *mut u8,
        size: u64,
        offset: u64,
    ) -> Result<i64, i32> {
        let buf = unsafe{std::slice::from_raw_parts(buf.cast(), size as usize)};
        if size + offset > CNK.get_chunk_size() {
            error_msg(
                "server::storage::chunk_storage::write_chunk".to_string(),
                "beyond chunk storage range".to_string(),
            );
        }
        if ENABLE_STUFFING && STUFF_WITH_ROCKSDB && chunk_id == 0 {
            StuffDB::get_instance().write(file_path, offset, size, buf);
            return Ok(size as i64);
        }
        ChunkStorage::init_chunk_space(file_path);
        let chunk_path =
            ChunkStorage::absolute(&ChunkStorage::get_chunks_path(file_path, chunk_id));
        let f = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(chunk_path)
            .unwrap();
        let mut wrote_tot: u64 = 0;

        while wrote_tot != size {
            if let Ok(bytes) =
                f.write_at(&buf[wrote_tot as usize..size as usize], offset + wrote_tot)
            {
                wrote_tot += bytes as u64;
            } else {
                error_msg(
                    "server::storage::chunk_storage::write_chunk".to_string(),
                    "error occured while writing to chunk".to_string(),
                );
                return Err(-1);
            }
        }
        Ok(wrote_tot as i64)
    }
    pub fn read_chunk(
        file_path: &String,
        chunk_id: u64,
        buf: &mut Vec<u8>,
        size: u64,
        mut offset: u64,
    ) -> Result<u64, i32> {
        if size + offset > CNK.get_chunk_size() {
            error_msg(
                "server::storage::chunk_storage::read_chunk".to_string(),
                "beyond chunk storage range".to_string(),
            );
        }
        if ENABLE_STUFFING && STUFF_WITH_ROCKSDB && chunk_id == 0 {
            if let Some(data) = StuffDB::get_instance().get(file_path) {
                if offset as usize > data.len() {
                    return Err(-1);
                }
                let start = offset as usize;
                let end = std::cmp::min((offset + size) as usize, data.len());
                let mut read = data[start..end].to_vec();
                let len = read.len() as u64;
                buf.clear();
                buf.append(&mut read);
                return Ok(len);
            } else {
                return Err(-1);
            }
        }
        ChunkStorage::init_chunk_space(file_path);
        let chunk_path =
            ChunkStorage::absolute(&ChunkStorage::get_chunks_path(file_path, chunk_id));
        let open_res = std::fs::OpenOptions::new().read(true).open(chunk_path);
        if let Err(_e) = open_res {
            error_msg(
                "server::storage::chunk_storage::read_chunk".to_string(),
                "fail to open chunk file".to_string(),
            );
            return Err(-1);
        }
        let f = open_res.unwrap();
        let mut read_tot: u64 = 0;
        let tmp = buf;
        let mut buf = &mut tmp[0..size as usize];
        while !buf.is_empty() {
            match f.read_at(buf, offset) {
                Ok(0) => break,
                Ok(n) => {
                    let tmp = buf;
                    buf = &mut tmp[n..];
                    offset += n as u64;
                    read_tot += n as u64;
                }
                Err(_e) => {
                    error_msg(
                        "server::storage::chunk_storage::read_chunk".to_string(),
                        "error occurs while reading from chunks".to_string(),
                    );
                    return Err(-1);
                }
            }
        }
        if !buf.is_empty() {
            error_msg(
                "server::storage::chunk_storage::read_chunk".to_string(),
                "unable to fill the buf because of reaching EOF".to_string(),
            );
            Ok(read_tot as u64)
        } else {
            Ok(read_tot as u64)
        }
    }
    pub fn trim_chunk_space(file_path: &String, chunk_start: u64) {
        if ENABLE_STUFFING && STUFF_WITH_ROCKSDB && chunk_start == 0 {
            StuffDB::get_instance().remove(file_path);
        }
        let chunk_dir = ChunkStorage::absolute(&ChunkStorage::get_chunks_dir(file_path));
        let dir_res = std::fs::read_dir(Path::new(&chunk_dir));
        if let Err(_e) = dir_res {
            return;
        }
        let dir_iter = dir_res.unwrap();
        let err = false;
        for entry in dir_iter {
            let entry = entry.unwrap();
            let chunk_path = entry.path();
            let chunk_id = chunk_path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .parse::<u64>()
                .unwrap();
            if chunk_id >= chunk_start {
                if let Err(_e) = fs::remove_file(chunk_path.as_path()) {
                    continue;
                    //error_msg("server::storage::chunk_storage::trim_chunk_space".to_string(), "fail to remove file".to_string());
                    //err = true;
                }
            }
        }
        if err {
            error_msg(
                "server::storage::chunk_storage::trim_chunk_space".to_string(),
                "error occurs while truncating".to_string(),
            );
        }
    }
    pub fn truncate_chunk_file(file_path: &String, chunk_id: u64, length: u64) {
        if length > CNK.get_chunk_size() {
            error_msg(
                "server::storage::chunk_storage::truncate_chunk_file".to_string(),
                "invalid length".to_string(),
            );
            return;
        }
        if ENABLE_STUFFING && STUFF_WITH_ROCKSDB && chunk_id == 0 {
            StuffDB::get_instance().truncate(file_path, length);
            return;
        }
        let chunk_path =
            ChunkStorage::absolute(&ChunkStorage::get_chunks_path(file_path, chunk_id));
        let f_res = fs::OpenOptions::new()
            .write(true)
            .read(true)
            .open(Path::new(&chunk_path));
        if let Err(_e) = f_res {
            return;
        }
        let f = f_res.unwrap();
        if let Err(_e) = f.set_len(length) {
            error_msg(
                "server::storage::chunk_storage::truncate_chunk_file".to_string(),
                "error occurs while truncating chunk file".to_string(),
            );
        }
    }
    pub fn chunk_stat() -> ChunkStat {
        let statfs = statfs(Path::new(CNK.get_root_path()));
        if let Err(_e) = statfs {
            error_msg(
                "server::storage::chunk_storage::chunk_stat".to_string(),
                "error occurs while get fs stat".to_string(),
            );
            return ChunkStat {
                chunk_size: 0,
                chunk_total: 0,
                chunk_free: 0,
            };
        }
        let statfs = statfs.unwrap();
        let bytes_tot = statfs.block_size() as u64 * statfs.blocks();
        let bytes_free = statfs.block_size() as u64 * statfs.blocks_available();
        ChunkStat {
            chunk_size: CNK.get_chunk_size(),
            chunk_total: bytes_tot / CNK.get_chunk_size(),
            chunk_free: bytes_free / CNK.get_chunk_size(),
        }
    }
    pub fn get_chunk_size(&self) -> u64 {
        self.chunk_size_
    }
    pub fn get_root_path(&self) -> &String {
        &self.root_path_
    }
}
