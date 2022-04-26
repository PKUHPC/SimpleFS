use std::{path::Path, time::{self, UNIX_EPOCH}};

use libc::{EEXIST, EINVAL};
use rocksdb::{Options, WriteOptions, DB};

use crate::{
    config::USE_WRITE_AHEAD_LOG,
    error_msg::error_msg,
    server::{
        config::TRUNCATE_DIRECTORY, filesystem::storage_context::StorageContext,
        storage::metadata::merge,
    },
};
use sfs_global::global::{
    metadata::{Metadata, S_ISDIR},
    util::{
        path_util::{has_trailing_slash, is_absolute},
        serde_util::serialize,
    },
};

use lazy_static::*;

use super::merge::Operand;

#[allow(unused_must_use)]
pub fn init_mdb() -> MetadataDB {
    let metadata_path =
        StorageContext::get_instance().get_metadir().clone() + &"/rocksdb".to_string();

    if TRUNCATE_DIRECTORY {
        std::fs::remove_dir_all(Path::new(&metadata_path));
    }
    return MetadataDB::new(&metadata_path).unwrap();
}
#[allow(dead_code)]
pub struct MetadataDB {
    pub db: DB,
    options: Options,
    write_opts: WriteOptions,
    path: String,
}
lazy_static! {
    static ref MDB: MetadataDB = init_mdb();
}
impl MetadataDB {
    pub fn get_instance() -> &'static MetadataDB {
        &MDB
    }
    pub fn optimize_rocksdb_options(options: &mut Options) {
        options.set_max_successive_merges(32);
    }
    pub fn new(path: &String) -> Option<MetadataDB> {
        let mut options = Options::default();
        options.increase_parallelism(10);
        options.optimize_level_style_compaction(512 * 1024 * 1024);
        options.create_if_missing(true);
        // merge operator need to be checked
        options.set_merge_operator(
            "simplefs merge operator",
            merge::full_merge,
            merge::partial_merge,
        );
        MetadataDB::optimize_rocksdb_options(&mut options);
        let mut write_options = WriteOptions::default();
        write_options.disable_wal(!USE_WRITE_AHEAD_LOG);
        if let Ok(rdb) = DB::open(&options, Path::new(path)) {
            Some(MetadataDB {
                db: rdb,
                options: options,
                write_opts: write_options,
                path: path.clone(),
            })
        } else {
            error_msg(
                "server::storage::metadata_db::new".to_string(),
                "fail to open database".to_string(),
            );
            None
        }
    }
    pub fn get(&self, key: &String) -> Option<Vec<u8>> {
        //println!("getting key: {}", key);
        if let Ok(Some(val)) = self.db.get(key) {
            Some(val)
        } else {
            None
        }
    }
    pub fn put(&self, key: &String, val: Vec<u8>, ignore_if_exists: bool) -> i32 {
        //println!("putting key: {}", key);
        if ignore_if_exists && self.exists(key) {
            return EEXIST;
        }
        if !is_absolute(key) {
            error_msg(
                "server::storage::metadata::db::put".to_string(),
                "key must be absolute path".to_string(),
            );
            return EINVAL;
        }
        if !key.eq(&"/".to_string()) && has_trailing_slash(key) {
            error_msg(
                "server::storage::metadata::db::put".to_string(),
                "key mustn't have trailing slash".to_string(),
            );
            return EINVAL;
        }
        let op = Operand::Create { md: val };
        let v = serialize(op);
        if let Err(_e) = self.db.merge_opt(key, v, &self.write_opts) {
            error_msg(
                "server::storage::metadata::db::put".to_string(),
                "fail to merge value".to_string(),
            );
            return EINVAL;
        }
        return 0;
    }
    pub fn remove(&self, key: &String) {
        if let Err(_e) = self.db.delete(key) {
            error_msg(
                "server::storage::metadata::db::delete".to_string(),
                "fail to delete key".to_string(),
            );
        }
    }
    pub fn exists(&self, key: &String) -> bool {
        if let Ok(res) = self.db.get(key) {
            if let Some(_value) = res {
                true
            } else {
                false
            }
        } else {
            error_msg(
                "server::storage::metadata::db::exists".to_string(),
                "fail to read key".to_string(),
            );
            false
        }
    }
    pub fn update(&self, old_key: &String, new_key: &String, val: &String) {
        let mut batch = rocksdb::WriteBatch::default();
        batch.delete(old_key);
        batch.put(new_key, val);
        if let Err(_e) = self.db.write_opt(batch, &self.write_opts) {
            error_msg(
                "server::storage::metadata::db::update".to_string(),
                "fail to write batch".to_string(),
            );
        }
    }
    pub fn increase_size(&self, key: &String, size: usize, append: bool) {
        let op_s = Operand::IncreaseSize {
            size,
            append,
            time: time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64,
        };
        let v = serialize(&op_s);
        if let Err(_e) = self.db.merge_opt(key, v, &self.write_opts) {
            error_msg(
                "server::storage::metadata::db::increase_size".to_string(),
                "fail to merge operands".to_string(),
            );
        }
    }
    pub fn decrease_size(&self, key: &String, size: usize) {
        let op_s = Operand::DecreaseSize {
            size,
            time: time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64,
        };
        let v = serialize(&op_s);
        if let Err(_e) = self.db.merge_opt(key, v, &self.write_opts) {
            error_msg(
                "server::storage::metadata::db::decrease_size".to_string(),
                "fail to merge operands".to_string(),
            );
        }
    }
    pub fn get_dirents(&self, dir: &String) -> Vec<(String, bool)> {
        let mut root_path = dir.clone();
        if !is_absolute(&root_path) {
            error_msg(
                "server::storage::metadata::db::get_dirents".to_string(),
                "dir is not absolute".to_string(),
            );
            return Vec::new();
        }
        if !has_trailing_slash(&root_path) && root_path.len() != 1 {
            root_path.push('/');
        }
        let iter = self.db.prefix_iterator(root_path.as_bytes());
        let mut entries: Vec<(String, bool)> = Vec::new();
        for (k, v) in iter {
            let s = String::from_utf8(k.to_vec()).unwrap();
            if !s.starts_with(&root_path) || s.len() == root_path.len() {
                continue;
            }
            if let Some(_idx) = s[root_path.len()..].to_string().find('/') {
                continue;
            }
            let name = s[root_path.len()..].to_string();
            if name.len() == 0 {
                continue;
            }
            let md = Metadata::deserialize(&v.to_vec());
            entries.push((name, S_ISDIR(md.get_mode())));
        }
        entries
    }
    #[allow(unused_variables)]
    #[allow(unused_assignments)]
    pub fn iterate_all(&self) {
        let mut key: String;
        let mut value: String;
        let iter = self.db.iterator(rocksdb::IteratorMode::Start);
        for (k, v) in iter {
            key = String::from_utf8(k.to_vec()).unwrap();
            value = String::from_utf8(v.to_vec()).unwrap();
        }
    }
}
