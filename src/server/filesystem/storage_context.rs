use lazy_static::*;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::server::storage::{metadata::db::MetadataDB, data::chunk_storage::ChunkStorage};


pub struct StorageContext{
    rootdir_: String,
    mountdir_: String,
    metadir_: String,

    rpc_protocol_: String,
    bind_addr_: String,
    hosts_file_: String,
    use_auto_sm_: bool,

    mdb_: Option<Arc<MetadataDB>>,
    storage_: Option<Arc<ChunkStorage>>,

    atime_state_: bool,
    mtime_state_: bool,
    ctime_state_: bool,
    link_count_state_: bool,
    blocks_state_: bool
}
lazy_static!{
    static ref CTX: Mutex<StorageContext> = Mutex::new(StorageContext{
        rootdir_: String::from(""),
        mountdir_: String::from(""),
        metadir_: String::from(""),

        rpc_protocol_: String::from(""),
        bind_addr_: String::from(""),
        hosts_file_: String::from(""),
        use_auto_sm_: true,

        mdb_: None,
        storage_: None,

        atime_state_: true,
        mtime_state_: true,
        ctime_state_: true,
        link_count_state_: true,
        blocks_state_: true
    });
}
impl StorageContext{
    pub fn get_instance() -> MutexGuard<'static, StorageContext>{
        CTX.lock().unwrap()
    }
    pub fn get_rootdir(&self) -> &String{
        &self.rootdir_
    }
    pub fn set_rootdir(&mut self, rootdir_: String){
        self.rootdir_ = rootdir_;
    }
    pub fn get_mountdir(&self) -> &String{
        &self.mountdir_
    }
    pub fn set_mountdir(&mut self, mountdir_: String){
        self.mountdir_ = mountdir_;
    }
    pub fn get_metadir(&self) -> &String{
        &self.metadir_
    }
    pub fn set_metadir(&mut self, metadir_: String){
        self.metadir_ = metadir_;
    }
    pub fn get_rpc_protocol(&self) -> &String{
        &self.rpc_protocol_
    }
    pub fn set_rpc_protocol(&mut self, rpc_protocol_: String){
        self.rpc_protocol_ = rpc_protocol_;
    }
    pub fn get_bind_addr(&self) -> &String{
        &self.bind_addr_
    }
    pub fn set_bind_addr(&mut self, bind_addr_: String){
        self.bind_addr_ = bind_addr_;
    }
    pub fn get_hosts_file(&self) -> &String{
        &self.hosts_file_
    }
    pub fn set_hosts_file(&mut self, hosts_file_: String){
        self.hosts_file_ = hosts_file_;
    }
    pub fn get_use_auto_sm(&self) -> bool{
        self.use_auto_sm_.clone()
    }
    pub fn set_use_auto_sm(&mut self, use_auto_sm_: bool){
        self.use_auto_sm_ = use_auto_sm_;
    }
    pub fn get_atime_state(&self) -> bool{
        self.atime_state_.clone()
    }
    pub fn set_atime_state(&mut self, atime_state_: bool){
        self.atime_state_ = atime_state_;
    }
    pub fn get_ctime_state(&self) -> bool{
        self.ctime_state_.clone()
    }
    pub fn set_ctime_state(&mut self, ctime_state_: bool){
        self.ctime_state_ = ctime_state_;
    }
    pub fn get_mtime_state(&self) -> bool{
        self.mtime_state_.clone()
    }
    pub fn set_mtime_state(&mut self, mtime_state_: bool){
        self.mtime_state_ = mtime_state_;
    }
    pub fn get_link_count_state(&self) -> bool{
        self.link_count_state_.clone()
    }
    pub fn set_link_count_state(&mut self, link_count_state_: bool){
        self.link_count_state_ = link_count_state_;
    }
    pub fn get_blocks_state(&self) -> bool{
        self.use_auto_sm_.clone()
    }
    pub fn set_blocks_state(&mut self, blocks_state_: bool){
        self.blocks_state_ = blocks_state_;
    }

    pub fn get_mdb(&self) -> Option<Arc<MetadataDB>>{
        if let Some(mdb) = &self.mdb_{
            Some(Arc::clone(mdb))
        }
        else {
            None
        }
    }
    pub fn set_mdb(&mut self, mdb_: MetadataDB){
        self.mdb_ = Some(Arc::new(mdb_));
    }
    pub fn close_db(&mut self){
        self.mdb_ = None;
    }

    pub fn get_storage(&self) -> Option<Arc<ChunkStorage>>{
        if let Some(storage) = &self.storage_{
            Some(Arc::clone(storage))
        }
        else {
            None
        }
    }
    pub fn set_storage(&mut self, storage_: ChunkStorage){
        self.storage_ = Some(Arc::new(storage_));
    }
}

