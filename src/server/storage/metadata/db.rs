use std::sync::Arc;

use rocksdb::{DB, Options, WriteOptions};

use crate::{server::storage::metadata::merge, global::{util::path_util::{is_absolute, has_trailing_slash}, error_msg::error_msg, metadata::{Metadata, self}}};
static use_write_ahead_log: bool = false;

pub struct MetadataDB{
    db: Arc<DB>,
    options: Options,
    write_opts: WriteOptions,
    path: String
}
impl MetadataDB{
    pub fn optimize_rocksdb_options(options:&mut Options){
        options.set_max_successive_merges(128);
    }
    pub fn new(path: String) -> Option<MetadataDB>{
        let mut options = Options::default();
        options.increase_parallelism(3);
        options.optimize_level_style_compaction(512 * 1024 * 1024);
        options.create_if_missing(true);
        // merge operator need to be checked
        options.set_merge_operator("simplefs merge operator", merge::full_merge, merge::partial_merge);
        MetadataDB::optimize_rocksdb_options(&mut options);
        let mut write_options = WriteOptions::default();
        write_options.disable_wal(use_write_ahead_log);
        if let Ok(rdb) = DB::open(&options, path.clone()){
            Some(MetadataDB{
                db: Arc::new(rdb),
                options: options,
                write_opts: write_options,
                path: path
            })
        }
        else{
            None
        }
    }
    pub fn get(&self, key: String) -> Option<String>{
        if let Ok(Some(val)) = self.db.get(key){
            Some(String::from_utf8(val).unwrap())
        }
        else{
            None
        }
    }
    pub fn put(&mut self, key: String, val: String){
        if !is_absolute(&key) {
            error_msg("server::storage::metadata::db::put".to_string(), "key must be absolute path".to_string());
            return;
        }
        if !key.eq(&"/".to_string()) && has_trailing_slash(&key) {
            error_msg("server::storage::metadata::db::put".to_string(), "key mustn't have trailing slash".to_string());
            return;
        }
        if let Err(e) = self.db.merge_opt(key, val, &self.write_opts){
            error_msg("server::storage::metadata::db::put".to_string(), "fail to merge value".to_string());
        }
        
    }
    pub fn remove(&mut self, key: String){
        if let Err(e) = self.db.delete(key){
            error_msg("server::storage::metadata::db::delete".to_string(), "fail to delete key".to_string());
        }
    }
    pub fn exists(&self, key: String) -> bool{
        if let Ok(res) = self.db.get(key){
            if let Some(value) = res{true}
            else{false}
        }
        else{
            error_msg("server::storage::metadata::db::exists".to_string(), "fail to read key".to_string());
            false
        }
    }
    pub fn update(&mut self, old_key: String, new_key: String, val: String){
        let mut batch = rocksdb::WriteBatch::default();
        batch.delete(old_key);
        batch.put(new_key, val);
        if let Err(e) = self.db.write_opt(batch, &self.write_opts){
            error_msg("server::storage::metadata::db::update".to_string(), "fail to write batch".to_string());
        }
    }
    pub fn increase_size(&mut self, key: String, size: usize, append: bool){
        let op_s = format!("i|{}|{}", size, append);
        if let Err(e) = self.db.merge_opt(key, op_s, &self.write_opts){
            error_msg("server::storage::metadata::db::increase_size".to_string(), "fail to merge operands".to_string()); 
        }
    }
    pub fn decrease_size(&mut self, key: String, size: usize){
        let op_s = format!("d|{}", size);
        if let Err(e) = self.db.merge_opt(key, op_s, &self.write_opts){
            error_msg("server::storage::metadata::db::decrease_size".to_string(), "fail to merge operands".to_string()); 
        }
    }
    pub fn get_dirents(&self, dir: String) -> Vec<(String, bool)>{
        let mut root_path = dir;
        if !is_absolute(&root_path) {
            error_msg("server::storage::metadata::db::get_dirents".to_string(), "dir is not absolute".to_string()); 
            return Vec::new();
        }
        if !has_trailing_slash(&root_path) && root_path.len() == 1{
            root_path.push('/');
        }
        let iter = self.db.prefix_iterator(root_path.clone());
        let mut entries: Vec<(String, bool)> = Vec::new();
        for (k, v) in iter{
            let s = String::from_utf8(k.to_vec()).unwrap();
            if !s.starts_with(&root_path) || s.len() == root_path.len(){
                continue;
            }
            if let Some(idx) = s[root_path.len()..].to_string().find('/'){
                continue;
            }
            let name = s[root_path.len()..].to_string();
            if name.len() == 0{
                continue;
            }
            if let Ok(md) = Metadata::deserialize(String::from_utf8(v.to_vec()).unwrap()){
                entries.push((name, md.get_mode() & metadata::S_IFDIR != 0));
            }
            else {continue;}
        }
        entries
    }
    pub fn iterate_all(&self){
        let mut key: String;
        let mut value: String;
        let iter = self.db.iterator(rocksdb::IteratorMode::Start);
        for (k, v) in iter{
            key = String::from_utf8(k.to_vec()).unwrap();
            value = String::from_utf8(v.to_vec()).unwrap();
        }
    }
}