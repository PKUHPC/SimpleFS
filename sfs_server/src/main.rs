pub mod handle;
pub mod task;
use libc::{getgid, getuid, EINVAL, ENOENT, S_IFDIR, S_IRWXG, S_IRWXO, S_IRWXU};
use sfs_global::global::network::post::i2option;
use sfs_global::{
    global::network::post::PostOption::*,
    global::{
        fsconfig::{SFSConfig, ENABLE_OUTPUT},
        metadata::Metadata,
        network::{
            config::CHUNK_SIZE,
            forward_data::{
                CreateData, DecrData, DirentData, ReadData, SerdeString, TruncData,
                UpdateMetadentryData, WriteData,
            },
        },
        util::net_util::get_my_hostname,
    },
};
use sfs_lib_server::server::{
    config::{ServerConfig, IGNORE_IF_EXISTS, TRUNCATE_DIRECTORY},
    network::network_context::NetworkContext,
};
use sfs_lib_server::server::{
    filesystem::storage_context::StorageContext, storage::data::chunk_storage::*,
    storage::metadata::db::MetadataDB,
};
use std::net::SocketAddrV4;
use std::{
    fs::{self, OpenOptions},
    io::{BufWriter, Error, Read, Write},
    net::{Ipv4Addr, SocketAddr},
    path::Path,
};
use tokio::sync::mpsc;
use tonic::{transport::Server, Request, Response, Status};

use crate::handle::{handle_read, handle_trunc, handle_write};
use sfs_rpc::sfs_server::sfs_handle_server::{SfsHandle, SfsHandleServer};
use sfs_rpc::sfs_server::{Post, PostResult};
use tokio_stream::wrappers::ReceiverStream;

#[derive(Clone, Default)]
struct ServerHandler {}
#[tonic::async_trait]
impl SfsHandle for ServerHandler {
    type handleStream = ReceiverStream<Result<PostResult, Status>>;
    async fn handle(
        &self,
        request: Request<tonic::Streaming<Post>>,
    ) -> Result<Response<Self::handleStream>, Status> {
        let mut streamer = request.into_inner();
        let (tx, rx) = mpsc::channel((CHUNK_SIZE / 8) as usize);
        tokio::spawn(async move {
            while let Some(post) = streamer.message().await.unwrap() {
                tx.send(Ok(handle_post(&post).await)).await.unwrap();
            }
        });
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
async fn handle_post(post: &Post) -> PostResult {
    //println!("recived post: {:?}", post);
    let option = i2option(post.option);
    match option {
        Stat => {
            let serde_string: SerdeString = serde_json::from_str(&post.data).unwrap();
            let path = serde_string.str;
            if ENABLE_OUTPUT {
                println!("handling metadata of '{}'....", path);
            }
            let md_res = MetadataDB::get_instance().get(&path);
            if let Some(md) = md_res {
                return PostResult { err: 0, data: md };
            } else {
                return PostResult {
                    err: ENOENT,
                    data: ENOENT.to_string(),
                };
            }
        }
        Create => {
            let create_data: CreateData = serde_json::from_str(post.data.as_str()).unwrap();
            if ENABLE_OUTPUT {
                println!("handling create of '{}'....", create_data.path);
            }
            let mode = create_data.mode;
            let mut md = Metadata::new();
            md.set_mode(mode);
            let create_res = MetadataDB::get_instance().put(
                &create_data.path,
                &md.serialize(),
                IGNORE_IF_EXISTS,
            );
            return PostResult {
                err: create_res,
                data: create_res.to_string(),
            };
        }
        Remove => {
            let serde_string: SerdeString = serde_json::from_str(&post.data).unwrap();
            let path = serde_string.str;
            if ENABLE_OUTPUT {
                println!("handling remove of '{}'....", path);
            }
            ChunkStorage::destroy_chunk_space(&path).await;
            return PostResult {
                err: 0,
                data: "0".to_string(),
            };
        }
        RemoveMeta => {
            let serde_string: SerdeString = serde_json::from_str(&post.data).unwrap();
            let path = serde_string.str;
            if ENABLE_OUTPUT {
                println!("handling remove metadata of '{}'....", path);
            }
            let md_res = MetadataDB::get_instance().get(&path);
            if let None = md_res {
                return PostResult {
                    err: ENOENT,
                    data: ENOENT.to_string(),
                };
            }
            MetadataDB::get_instance().remove(&path);
            return PostResult {
                err: 0,
                data: "0".to_string(),
            };
        }
        Read => {
            let read_data: ReadData = serde_json::from_str(&post.data).unwrap();
            if ENABLE_OUTPUT {
                println!("handling read of '{}'....", read_data.path);
            }
            return handle_read(read_data).await;
        }
        Write => {
            let write_data: WriteData = serde_json::from_str(&post.data).unwrap();
            if ENABLE_OUTPUT {
                println!("handling write of '{}'....", write_data.path);
            }
            return handle_write(write_data).await;
        }
        Lookup => {
            if ENABLE_OUTPUT {
                println!("handling look up....");
            }
            let id: u64 = serde_json::from_str(&post.data).unwrap();
            StorageContext::get_instance().set_host_id(id);
            return PostResult {
                err: 0,
                data: "0".to_string(),
            };
        }
        FsConfig => {
            if ENABLE_OUTPUT {
                println!("handling fsconfig....");
            }
            let mut fs_config = SFSConfig::new();
            fs_config.mountdir = StorageContext::get_instance().get_mountdir().to_string();
            fs_config.rootdir = StorageContext::get_instance().get_rootdir().to_string();
            fs_config.atime_state = StorageContext::get_instance().get_atime_state();
            fs_config.ctime_state = StorageContext::get_instance().get_ctime_state();
            fs_config.mtime_state = StorageContext::get_instance().get_mtime_state();
            fs_config.link_cnt_state = StorageContext::get_instance().get_link_count_state();
            fs_config.blocks_state = StorageContext::get_instance().get_blocks_state();
            fs_config.uid = unsafe { getuid() };
            fs_config.gid = unsafe { getgid() };
            return PostResult {
                err: 0,
                data: serde_json::to_string(&fs_config).unwrap(),
            };
        }
        UpdateMetadentry => {
            let update_data: UpdateMetadentryData = serde_json::from_str(&post.data).unwrap();
            if ENABLE_OUTPUT {
                println!("handling update metadentry of '{}'....", update_data.path);
            }
            MetadataDB::get_instance().increase_size(
                &update_data.path,
                update_data.size as usize + update_data.offset as usize,
                update_data.append,
            );
            return PostResult {
                err: 0,
                data: (update_data.size as usize + update_data.offset as usize).to_string(),
            };
        }
        GetMetadentry => {
            if ENABLE_OUTPUT {
                println!("handling get metadentry....");
            }
            let serde_string: SerdeString = serde_json::from_str(&post.data).unwrap();
            let path = serde_string.str;
            let md_str = MetadataDB::get_instance().get(&path);
            match md_str {
                None => {
                    return PostResult {
                        err: ENOENT,
                        data: ENOENT.to_string(),
                    };
                }
                Some(str) => {
                    let md = Metadata::deserialize(&str).unwrap();
                    return PostResult {
                        err: 0,
                        data: md.get_size().to_string(),
                    };
                }
            }
        }
        ChunkStat => {
            if ENABLE_OUTPUT {
                println!("handling chunk stat....");
            }
            let chunk_stat = ChunkStorage::chunk_stat();
            let post_result = PostResult {
                err: 0,
                data: serde_json::to_string(&chunk_stat).unwrap(),
            };
            return post_result;
        }
        DecrSize => {
            let decr_data: DecrData = serde_json::from_str(&post.data).unwrap();
            if ENABLE_OUTPUT {
                println!("handling decrease size of '{}'....", decr_data.path);
            }
            MetadataDB::get_instance().decrease_size(&decr_data.path, decr_data.new_size as usize);
            return PostResult {
                err: 0,
                data: "0".to_string(),
            };
        }
        Trunc => {
            let trunc_data: TruncData = serde_json::from_str(&post.data).unwrap();
            if ENABLE_OUTPUT {
                println!("handling truncate of '{}'....", trunc_data.path);
            }
            return handle_trunc(trunc_data).await;
        }
        GetDirents => {
            let data: DirentData = serde_json::from_str(&post.data).unwrap();
            let path = data.path;
            if ENABLE_OUTPUT {
                println!("handling get dirents of '{}'....", path);
            }
            let entries = MetadataDB::get_instance().get_dirents(&path);
            if entries.len() == 0 {
                return PostResult {
                    err: 0,
                    data: serde_json::to_string(&(Vec::new() as Vec<(String, bool)>)).unwrap(),
                };
            }
            return PostResult {
                err: 0,
                data: serde_json::to_string(&entries).unwrap(),
            };
        }
        _ => {
            return PostResult {
                err: EINVAL,
                data: EINVAL.to_string(),
            };
        }
    }
}

async fn init_server(addr: &String) -> Result<(), Error> {
    NetworkContext::get_instance().set_self_addr(addr.clone());
    let server_addr: (Ipv4Addr, u16) = (addr.parse().unwrap(), 8082);
    println!("listening on {:?}", server_addr);
    /*
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
    */
    let handler = ServerHandler::default();
    Server::builder()
        .add_service(SfsHandleServer::new(handler))
        .serve(SocketAddr::V4(SocketAddrV4::new(
            server_addr.0,
            server_addr.1,
        )))
        .await
        .unwrap();
    Ok(())
}

fn populates_host_file() -> Option<Error> {
    let open_res = OpenOptions::new().read(true).open(Path::new(
        StorageContext::get_instance().get_hosts_file().as_str(),
    ));
    if let Err(e) = open_res {
        return Some(e);
    }
    let host_file = open_res.unwrap();
    let mut host_writer = BufWriter::new(host_file);
    if let Err(e) = host_writer.write(
        format!(
            "{} {}",
            get_my_hostname(true),
            NetworkContext::get_instance().get_self_addr()
        )
        .as_bytes(),
    ) {
        return Some(e);
    }
    None
}
async fn init_environment() -> Result<(), Error> {
    let chunk_storage_path =
        StorageContext::get_instance().get_rootdir().clone() + &"/data/chunks".to_string();
    let metadata_path =
        StorageContext::get_instance().get_metadir().clone() + &"/rocksdb".to_string();

    if TRUNCATE_DIRECTORY {
        fs::remove_dir_all(Path::new(&chunk_storage_path))?;
        fs::remove_dir_all(Path::new(&metadata_path))?;
    }
    // init metadata storage

    fs::create_dir_all(Path::new(&metadata_path))
        .expect("fail to create metadata data base directory");
    StorageContext::set_mdb(
        MetadataDB::new(&metadata_path).expect("fail to create metadata data base"),
    );

    // init chunk storage
    fs::create_dir_all(Path::new(&chunk_storage_path))
        .expect("fail to create chunk storage directory");
    StorageContext::set_storage(
        ChunkStorage::new(&chunk_storage_path, CHUNK_SIZE).expect("fail to create chunk storage"),
    );

    StorageContext::get_instance().set_atime_state(true);
    StorageContext::get_instance().set_mtime_state(true);
    StorageContext::get_instance().set_ctime_state(true);
    StorageContext::get_instance().set_link_count_state(true);
    StorageContext::get_instance().set_blocks_state(true);

    let mut root_md = Metadata::new();
    root_md.set_mode(S_IFDIR | S_IRWXU | S_IRWXG | S_IRWXO);

    MetadataDB::get_instance().put(&"/".to_string(), &root_md.serialize(), IGNORE_IF_EXISTS);

    if !StorageContext::get_instance().get_hosts_file().len() == 0 {
        populates_host_file();
    }
    let addr = StorageContext::get_instance().get_bind_addr().clone();
    init_server(&addr).await?;
    Ok(())
}
//fn destroy_environment() {}
#[tokio::main]
pub async fn main() -> Result<(), Error> {
    //let RPC_PROTOCOL: String = String::from("tcp");

    let mut json: Vec<u8> = Vec::new();
    let mut f = std::fs::OpenOptions::new()
        .read(true)
        .open("config.json".to_string())?;

    f.read_to_end(&mut json).expect("fail to read config file");
    let s = String::from_utf8(json.clone()).unwrap();
    let config: ServerConfig =
        serde_json::from_str(s.as_str()).expect("JSON was not well-formatted");

    fs::create_dir_all(Path::new(&config.mountdir)).expect("fail to create mount directory");
    fs::create_dir_all(Path::new(&config.metadir)).expect("fail to create meta directory");
    StorageContext::get_instance().set_mountdir(
        fs::canonicalize(&config.mountdir)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(),
    );
    let root_dirpath = config.rootdir;
    //let root_dirpath = root_dir + &std::process::id().to_string();
    fs::create_dir_all(Path::new(&root_dirpath)).expect("fail to create root directory");
    StorageContext::get_instance().set_rootdir(root_dirpath);
    StorageContext::get_instance().set_metadir(
        fs::canonicalize(&config.metadir)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(),
    );
    StorageContext::get_instance().set_hosts_file(config.hosts_file);
    //StorageContext::get_instance().set_bind_addr(format!("{}://{}", RPC_PROTOCOL, config.listen));
    StorageContext::get_instance().set_bind_addr(config.listen);

    init_environment().await?;
    /*
    let chunk_storage_path = "/home/dev/Desktop/storage/data/chunks".to_string();
    fs::remove_dir_all(Path::new(&chunk_storage_path));
    fs::create_dir_all(Path::new(&chunk_storage_path)).expect("fail to create chunk storage directory");
    StorageContext::set_storage(ChunkStorage::new(&chunk_storage_path, CHUNK_SIZE).expect("fail to create chunk storage"));

    let metadata_path = "/home/dev/Desktop/metadata/rocksdb".to_string();
    StorageContext::get_instance().set_metadir(metadata_path.clone());
    fs::remove_dir_all(Path::new(&metadata_path));
    fs::create_dir_all(Path::new(&metadata_path)).expect("fail to create metadata directory");
    StorageContext::set_mdb(MetadataDB::new(&metadata_path).expect("fail to create metadata data base"));

    let mut root_md = Metadata::new();
    root_md.set_mode(S_IFDIR | S_IRWXU | S_IRWXG | S_IRWXO);

    MetadataDB::get_instance().put(&"/".to_string(), &root_md.serialize(), IGNORE_IF_EXISTS);

    //MetadataDB::get_instance().put(&"/sfs/test/async_write/a".to_string(), &"c|32768|0|0|0|0|1|0".to_string());
    //println!("?");
    init_server(&"192.168.230.137".to_string()).await?;
    */
    Ok(())
}
