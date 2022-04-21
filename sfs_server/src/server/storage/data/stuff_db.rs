use std::path::Path;

use lazy_static::*;
use libc::EINVAL;
use rocksdb::{Options, WriteOptions, DB};
use sfs_global::global::{error_msg::error_msg, util::serde_util::serialize};

use crate::{
    config::USE_WRITE_AHEAD_LOG,
    server::{config::TRUNCATE_DIRECTORY, filesystem::storage_context::StorageContext},
};

use super::merge::{self, Operand};

#[allow(unused_must_use)]
pub fn init_sdb() -> StuffDB {
    let stuff_path = StorageContext::get_instance().get_rootdir().clone() + &"/stuff".to_string();

    if TRUNCATE_DIRECTORY {
        std::fs::remove_dir_all(Path::new(&stuff_path));
    }
    return StuffDB::new(&stuff_path).unwrap();
}
#[allow(dead_code)]
pub struct StuffDB {
    pub db: DB,
    options: Options,
    write_opts: WriteOptions,
    path: String,
}
lazy_static! {
    static ref SDB: StuffDB = init_sdb();
}
impl StuffDB {
    pub fn get_instance() -> &'static StuffDB {
        &SDB
    }
    pub fn optimize_rocksdb_options(options: &mut Options) {
        options.set_max_successive_merges(125);
    }
    pub fn new(path: &String) -> Option<StuffDB> {
        let mut options = Options::default();
        options.increase_parallelism(10);
        options.optimize_level_style_compaction(512 * 1024 * 1024);
        options.create_if_missing(true);
        // merge operator need to be checked
        options.set_merge_operator(
            "simplefs stuff merge operator",
            merge::full_merge,
            merge::partial_merge,
        );
        StuffDB::optimize_rocksdb_options(&mut options);
        let mut write_options = WriteOptions::default();
        write_options.disable_wal(!USE_WRITE_AHEAD_LOG);
        if let Ok(rdb) = DB::open(&options, Path::new(path)) {
            Some(StuffDB {
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
    pub fn write(&self, key: &String, offset: u64, size: u64, data: &[u8]) -> i32 {
        //println!("putting key: {}", key);
        let op = Operand::Write {
            offset,
            size,
            data: data.to_vec(),
        };
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
    pub fn truncate(&self, key: &String, offset: u64) -> i32 {
        let op = Operand::Truncate { offset };
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
}
