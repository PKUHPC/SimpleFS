#[warn(non_snake_case)]
#[warn(dead_code)]
#[warn(unused_assignments)]
pub mod handle;
pub mod task;
use std::{fs::{self, OpenOptions}, io::{Error, BufWriter, Write}, path::Path, net::{Ipv4Addr, IpAddr, SocketAddr}};
use libc::{S_IFDIR, S_IRWXU, S_IRWXG, S_IRWXO};
use sfs_lib_server::{global::network::post::PostOption::*, global::{network::{rpc::SFSServer, forward_data::{WriteData, ReadData, CreateData, UpdateMetadentryData}, config::CHUNK_SIZE, post::PostResult}, error_msg::error_msg, util::net_util::get_my_hostname, metadata::Metadata}, server::{config::ServerConfig, network::network_context::NetworkContext}};
use sfs_lib_server::{server::{filesystem::storage_context::StorageContext, storage::metadata::db::MetadataDB, storage::data::chunk_storage::*}, global::network::post::Post};

use futures::{future, prelude::*};
use tarpc::{
    context,
    server::{self, incoming::Incoming, Channel},
    tokio_serde::formats::Json,
};

use crate::handle::{handle_write, handle_read};

#[derive(Clone)]
struct ServerHandler(SocketAddr);
#[tarpc::server]
impl SFSServer for ServerHandler {
    async fn handle(self, _: context::Context, post: String) -> String {
        println!("handling....");
        let post: Post = serde_json::from_str(post.as_str()).unwrap();
        match post.option {
            Stat => {
                let path = post.data;
                let md_res = MetadataDB::get_instance().get(&path);
                if let Some(md) = md_res{
                    return serde_json::to_string(&PostResult{err: false, data: md}).unwrap();
                }
                else{
                    return serde_json::to_string(&PostResult{err: true, data: "".to_string()}).unwrap();
                }
            },
            Create => {
                let create_data: CreateData = serde_json::from_str(post.data.as_str()).unwrap();
                let mode = create_data.mode;
                let mut md = Metadata::new();
                md.set_mode(mode);
                let create_res = MetadataDB::get_instance().put(&create_data.path, &md.serialize());
                return serde_json::to_string(&PostResult{err: create_res != 0, data: create_res.to_string()}).unwrap();
            },
            Remove => {
                ChunkStorage::get_instance().destroy_chunk_space(&post.data);
                return serde_json::to_string(&PostResult{err: false, data: "0".to_string()}).unwrap();
            },
            RemoveMeta => {
                let path = post.data;
                let md_res = MetadataDB::get_instance().get(&path);
                if let None = md_res{
                    return serde_json::to_string(&PostResult{
                        err: true,
                        data: "1".to_string()
                    }).unwrap();
                }
                MetadataDB::get_instance().remove(&path);
                return serde_json::to_string(&PostResult{
                    err: false,
                    data: "0".to_string()
                }).unwrap();
            },
            Read => {
                let read_data: ReadData = serde_json::from_str(&post.data).unwrap();
                return handle_read(read_data).await;
            },
            Write => {
                let write_data: WriteData = serde_json::from_str(&post.data).unwrap();
                return handle_write(write_data).await;
            },
            Lookup => {
                let id: u64 = serde_json::from_str(&post.data).unwrap();
                StorageContext::get_instance().set_host_id(id);
            },
            FsConfig => todo!(),
            UpdateMetadentry => {
                let update_data: UpdateMetadentryData = serde_json::from_str(&post.data).unwrap();
                MetadataDB::get_instance().increase_size(&update_data.path, update_data.size as usize + update_data.offset as usize, update_data.append);
                return serde_json::to_string(&PostResult{
                    err: false,
                    data: (update_data.size as usize + update_data.offset as usize).to_string()
                }).unwrap();
            },
            GetMetadentry => {
                let path = post.data;
                let md_str = MetadataDB::get_instance().get(&path);
                if let None = md_str{
                    return serde_json::to_string(&PostResult{
                        err: true,
                        data: "1".to_string()
                    }).unwrap();
                }
                let md = Metadata::deserialize(&md_str.unwrap()).unwrap();
                return serde_json::to_string(&PostResult{
                    err: false,
                    data: md.get_size().to_string()
                }).unwrap();
            },
            ChunkStat => {
                let chunk_stat = ChunkStorage::get_instance().chunk_stat();
                let post_result = PostResult{
                    err: false,
                    data: serde_json::to_string(&chunk_stat).unwrap()
                };
                return serde_json::to_string(&post_result).unwrap();
            },
            DecrSize => {
                
            }
        }
        serde_json::to_string(&PostResult{
            err: false,
            data: "".to_string()
        }).unwrap()
    }
}

async fn init_server(addr: &String) -> Result<(), Error>{
    NetworkContext::get_instance().set_self_addr(addr.clone());
    
    let server_addr = (IpAddr::V4(Ipv4Addr::LOCALHOST), 8082);
    let mut listener = tarpc::serde_transport::tcp::listen(&server_addr, Json::default).await?;
    listener.config_mut().max_frame_length(usize::MAX);
    listener
        // Ignore accept errors.
        .filter_map(|r| future::ready(r.ok()))
        .map(server::BaseChannel::with_defaults)
        // Limit channels to 3 per IP.
        .max_channels_per_key(3, |t| t.transport().peer_addr().unwrap().ip())
        // serve is generated by the service attribute. It takes as input any type implementing
        // the generated World trait.
        .map(|channel| {
            let server = ServerHandler(channel.transport().peer_addr().unwrap());
            channel.execute(server.serve())
        })
        // Max 10 channels.
        .buffer_unordered(10)
        .for_each(|_| async {})
        .await;

    Ok(())
}

fn populates_host_file() -> Option<Error>{
    let open_res = OpenOptions::new().read(true).open(Path::new(StorageContext::get_instance().get_hosts_file().as_str()));
    if let Err(e) = open_res{
        return Some(e);
    }
    let host_file = open_res.unwrap();
    let mut host_writer = BufWriter::new(host_file);
    if let Err(e) = host_writer.write(format!("{} {}", get_my_hostname(true), NetworkContext::get_instance().get_self_addr()).as_bytes()){
        return Some(e)
    }
    None
}
async fn init_environment() -> Result<(), Error>{
    // init metadata storage
    let metadata_path = StorageContext::get_instance().get_metadir().clone() + &"/rocksdb".to_string();
    StorageContext::set_mdb(MetadataDB::new(metadata_path).expect("fail to create metadata data base"));

    // init chunk storage
    let chunk_storage_path = StorageContext::get_instance().get_rootdir().clone() + &"/data/chunks".to_string();
    fs::create_dir_all(Path::new(&chunk_storage_path)).expect("fail to create chunk storage");
    StorageContext::set_storage(ChunkStorage::new(&chunk_storage_path, CHUNK_SIZE).expect("fail to create chunk storage"));

    init_server(StorageContext::get_instance().get_bind_addr()).await?;

    StorageContext::get_instance().set_atime_state(true);
    StorageContext::get_instance().set_mtime_state(true);
    StorageContext::get_instance().set_ctime_state(true);
    StorageContext::get_instance().set_link_count_state(true);
    StorageContext::get_instance().set_blocks_state(true);
    let mut root_md = Metadata::new();
    root_md.set_mode(S_IFDIR | S_IRWXU | S_IRWXG | S_IRWXO);

    MetadataDB::get_instance().put(&"/".to_string(), &root_md.serialize());
    
    if !StorageContext::get_instance().get_hosts_file().len() == 0{
        populates_host_file();
    }
    Ok(())
}
fn destroy_environment(){

}
#[tokio::main]
pub async fn main() -> Result<(), Error>{
    /*
    let RPC_PROTOCOL: String = String::from("tcp");
    
    let mut json: Vec<u8> = Vec::new();
    let mut f =  fs::OpenOptions::new().read(true).open("config.json".to_string())?;

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

    init_environment().await?;
    */
    let chunk_storage_path = "/home/dev/Desktop/storage/data/chunks".to_string();fs::create_dir_all(Path::new(&chunk_storage_path)).expect("fail to create chunk storage");
    StorageContext::set_storage(ChunkStorage::new(&chunk_storage_path, CHUNK_SIZE).expect("fail to create chunk storage"));
    init_server(&"192.168.230.137".to_string()).await?;
    Ok(())
}