use std::{fs::{self}, io::Read, path::Path};
use crate::global::error_msg::error_msg;

use super::{config::ServerConfig, filesystem::storage_context::StorageContext, storage::metadata::db::MetadataDB, storage::data::chunk_storage::*};

fn init_server(addr: &String){
    todo!()
}

fn init_environment(){
    let metadata_path = StorageContext::get_instance().get_metadir().clone() + &"/rocksdb".to_string();
    StorageContext::get_instance().set_mdb(MetadataDB::new(metadata_path).expect("fail to create metadata data base"));

    let chunk_storage_path = StorageContext::get_instance().get_rootdir().clone() + &"/data/chunks".to_string();
    fs::create_dir_all(Path::new(&chunk_storage_path)).expect("fail to create chunk storage");
    StorageContext::get_instance().set_storage(ChunkStorage::new(&chunk_storage_path, CHUNKSIZE).expect("fail to create chunk storage"));

    init_server(StorageContext::get_instance().get_bind_addr());

    if !StorageContext::get_instance().get_hosts_file().len() == 0{
        todo!()
    }
}
pub fn main(){
    let RPC_PROTOCOL: String = String::from("http");
    
    let mut json: Vec<u8> = Vec::new();
    let open_res =  fs::OpenOptions::new().read(true).open("config.json".to_string());
    if let Err(e) = open_res{
        error_msg("server_main".to_string(), "fail to open config file".to_string());
        return;
    }
    let mut f = open_res.unwrap();
    f.read_to_end(&mut json).expect("fail to read config file");
    let s = String::from_utf8(json.clone()).unwrap();
    let config: ServerConfig = serde_json::from_str(s.as_str()).expect("JSON was not well-formatted");

    fs::create_dir_all(Path::new(&config.mountdir)).expect("fail to create mount directory");
    StorageContext::get_instance().set_mountdir(fs::canonicalize(&config.mountdir).unwrap().to_str().unwrap().to_string());
    let root_dir = config.rootdir;
    let root_dirpath = root_dir + &std::process::id().to_string();
    fs::create_dir_all(Path::new(&root_dirpath)).expect("fail to create root directory");
    StorageContext::get_instance().set_rootdir(root_dirpath);
    StorageContext::get_instance().set_metadir(fs::canonicalize(&config.metadir).unwrap().to_str().unwrap().to_string());
    StorageContext::get_instance().set_hosts_file(config.hosts_file);
    StorageContext::get_instance().set_bind_addr(format!("{}://{}", RPC_PROTOCOL, config.listen));

    init_environment();
}