use lazy_static::*;
use std::{fs, io::Read, path::Path};

use crate::server::{config::ServerConfig, storage::metadata::db::MetadataDB};

pub fn init_context() -> StorageContext {
    let mut context = StorageContext::new();
    let mut json: Vec<u8> = Vec::new();
    let mut f = std::fs::OpenOptions::new()
        .read(true)
        .open("config.json".to_string())
        .unwrap();

    f.read_to_end(&mut json).expect("fail to read config file");
    let s = String::from_utf8(json.clone()).unwrap();
    let config: ServerConfig =
        serde_json::from_str(s.as_str()).expect("JSON was not well-formatted");

    fs::create_dir_all(Path::new(&config.mountdir)).expect("fail to create mount directory");
    fs::create_dir_all(Path::new(&config.metadir)).expect("fail to create meta directory");
    context.set_mountdir(
        fs::canonicalize(&config.mountdir)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(),
    );
    let root_dirpath = config.rootdir;
    //let root_dirpath = root_dir + &std::process::id().to_string();
    fs::create_dir_all(Path::new(&root_dirpath)).expect("fail to create root directory");
    context.set_rootdir(root_dirpath);
    context.set_metadir(
        fs::canonicalize(&config.metadir)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(),
    );
    context.set_hosts_file(config.hosts_file);
    //context.set_bind_addr(format!("{}://{}", RPC_PROTOCOL, config.listen));
    context.set_bind_addr(config.listen);

    context.set_atime_state(true);
    context.set_mtime_state(true);
    context.set_ctime_state(true);
    context.set_link_count_state(true);
    context.set_blocks_state(true);

    context
}
pub struct StorageContext {
    host_id_: u64,
    rootdir_: String,
    mountdir_: String,
    metadir_: String,

    rpc_protocol_: String,
    bind_addr_: String,
    hosts_file_: String,
    use_auto_sm_: bool,

    atime_state_: bool,
    mtime_state_: bool,
    ctime_state_: bool,
    link_count_state_: bool,
    blocks_state_: bool,
}
lazy_static! {
    static ref CTX: StorageContext = init_context();
}
impl StorageContext {
    pub fn get_instance() -> &'static StorageContext {
        &CTX
    }
    pub fn new() -> StorageContext {
        StorageContext {
            host_id_: 0,
            rootdir_: String::from(""),
            mountdir_: String::from(""),
            metadir_: String::from(""),

            rpc_protocol_: String::from(""),
            bind_addr_: String::from(""),
            hosts_file_: String::from(""),
            use_auto_sm_: true,

            atime_state_: true,
            mtime_state_: true,
            ctime_state_: true,
            link_count_state_: true,
            blocks_state_: true,
        }
    }
    pub fn get_rootdir(&self) -> &String {
        &self.rootdir_
    }
    pub fn set_rootdir(&mut self, rootdir_: String) {
        self.rootdir_ = rootdir_;
    }
    pub fn get_mountdir(&self) -> &String {
        &self.mountdir_
    }
    pub fn set_mountdir(&mut self, mountdir_: String) {
        self.mountdir_ = mountdir_;
    }
    pub fn get_metadir(&self) -> &String {
        &self.metadir_
    }
    pub fn set_metadir(&mut self, metadir_: String) {
        self.metadir_ = metadir_;
    }
    pub fn get_rpc_protocol(&self) -> &String {
        &self.rpc_protocol_
    }
    pub fn set_rpc_protocol(&mut self, rpc_protocol_: String) {
        self.rpc_protocol_ = rpc_protocol_;
    }
    pub fn get_bind_addr(&self) -> &String {
        &self.bind_addr_
    }
    pub fn set_bind_addr(&mut self, bind_addr_: String) {
        self.bind_addr_ = bind_addr_;
    }
    pub fn get_hosts_file(&self) -> &String {
        &self.hosts_file_
    }
    pub fn set_hosts_file(&mut self, hosts_file_: String) {
        self.hosts_file_ = hosts_file_;
    }
    pub fn get_use_auto_sm(&self) -> bool {
        self.use_auto_sm_.clone()
    }
    pub fn set_use_auto_sm(&mut self, use_auto_sm_: bool) {
        self.use_auto_sm_ = use_auto_sm_;
    }
    pub fn get_atime_state(&self) -> bool {
        self.atime_state_.clone()
    }
    pub fn set_atime_state(&mut self, atime_state_: bool) {
        self.atime_state_ = atime_state_;
    }
    pub fn get_ctime_state(&self) -> bool {
        self.ctime_state_.clone()
    }
    pub fn set_ctime_state(&mut self, ctime_state_: bool) {
        self.ctime_state_ = ctime_state_;
    }
    pub fn get_mtime_state(&self) -> bool {
        self.mtime_state_.clone()
    }
    pub fn set_mtime_state(&mut self, mtime_state_: bool) {
        self.mtime_state_ = mtime_state_;
    }
    pub fn get_link_count_state(&self) -> bool {
        self.link_count_state_.clone()
    }
    pub fn set_link_count_state(&mut self, link_count_state_: bool) {
        self.link_count_state_ = link_count_state_;
    }
    pub fn get_blocks_state(&self) -> bool {
        self.use_auto_sm_.clone()
    }
    pub fn set_blocks_state(&mut self, blocks_state_: bool) {
        self.blocks_state_ = blocks_state_;
    }

    pub fn get_mdb(&self) -> &'static MetadataDB {
        MetadataDB::get_instance()
    }
    //pub fn get_storage() -> &'static ChunkStorage{
    //    ChunkStorage::get_instance()
    //}
    pub fn set_host_id(&mut self, id: u64) {
        self.host_id_ = id;
    }
    pub fn get_host_id(&self) -> u64 {
        self.host_id_
    }
}
