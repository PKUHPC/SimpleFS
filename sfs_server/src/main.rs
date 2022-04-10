pub mod config;
pub mod handle;
pub mod server;
use crate::server::{config::IGNORE_IF_EXISTS, network::network_context::NetworkContext};
use crate::server::{
    filesystem::storage_context::StorageContext, storage::data::chunk_storage::*,
    storage::metadata::db::MetadataDB,
};
use config::ENABLE_PRECREATE;
use handle::handle_precreate;
use libc::{getgid, getuid, EINVAL, ENOENT, S_IFDIR, S_IRWXG, S_IRWXO, S_IRWXU};
use server::network::network_service::NetworkService;
use sfs_global::global::distributor::Distributor;
use sfs_global::global::network::forward_data::PreCreateData;
use sfs_global::global::network::post::i2option;
use sfs_global::global::util::serde_util::{deserialize, serialize};
use sfs_global::{
    global::network::post::PostOption::*,
    global::{
        fsconfig::SFSConfig,
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
use std::collections::HashMap;
use std::net::SocketAddrV4;
use std::{
    fs::OpenOptions,
    io::{BufWriter, Error, Write},
    net::{Ipv4Addr, SocketAddr},
    path::Path,
};
use tokio::sync::mpsc;
use tonic::{transport::Server, Request, Response, Status};

use crate::config::ENABLE_OUTPUT;
use crate::handle::{handle_read, handle_trunc, handle_write};
use sfs_rpc::sfs_server::sfs_handle_server::{SfsHandle, SfsHandleServer};
use sfs_rpc::sfs_server::{Post, PostResult};
use tokio_stream::wrappers::ReceiverStream;

#[allow(unused)]
use std::time::Instant;

async fn handle_request(post: &Post) -> PostResult {
    let option = i2option(post.option);
    match option {
        Stat => {
            let serde_string: SerdeString = deserialize::<SerdeString>(&post.data);
            let path = serde_string.str;
            if ENABLE_OUTPUT {
                println!("handling metadata of '{}'....", path);
            }
            let md_res = MetadataDB::get_instance().get(&path.to_string());
            if let Some(md) = md_res {
                return PostResult {
                    err: 0,
                    data: md,
                    extra: vec![0; 0],
                };
            } else {
                return PostResult {
                    err: ENOENT,
                    data: ENOENT.to_string().as_bytes().to_vec(),
                    extra: vec![0; 0],
                };
            }
        }
        Create => {
            let create_data: CreateData = deserialize::<CreateData>(&post.data);
            if ENABLE_OUTPUT {
                println!("handling create of '{}'....", create_data.path);
            }
            let mode = create_data.mode;
            let mut md = Metadata::new();
            md.set_mode(mode);
            let create_res = MetadataDB::get_instance().put(
                &create_data.path.to_string(),
                md.serialize(),
                IGNORE_IF_EXISTS,
            );
            return PostResult {
                err: create_res,
                data: create_res.to_string().as_bytes().to_vec(),
                extra: vec![0; 0],
            };
        }
        Remove => {
            let serde_string: SerdeString = deserialize::<SerdeString>(&post.data);
            let path = serde_string.str;
            if ENABLE_OUTPUT {
                println!("handling remove of '{}'....", path);
            }
            ChunkStorage::destroy_chunk_space(&path.to_string());
            return PostResult {
                err: 0,
                data: "0".to_string().as_bytes().to_vec(),
                extra: vec![0; 0],
            };
        }
        RemoveMeta => {
            let serde_string: SerdeString = deserialize::<SerdeString>(&post.data);
            let path = serde_string.str;
            if ENABLE_OUTPUT {
                println!("handling remove metadata of '{}'....", path);
            }
            let md_res = MetadataDB::get_instance().get(&path.to_string());
            if let None = md_res {
                return PostResult {
                    err: ENOENT,
                    data: ENOENT.to_string().as_bytes().to_vec(),
                    extra: vec![0; 0],
                };
            } else {
                MetadataDB::get_instance().remove(&path.to_string());
                return PostResult {
                    err: 0,
                    data: "0".to_string().as_bytes().to_vec(),
                    extra: vec![0; 0],
                };
            }
        }
        Lookup => {
            if ENABLE_OUTPUT {
                println!("handling look up....");
            }
            return PostResult {
                err: 0,
                data: "0".to_string().as_bytes().to_vec(),
                extra: vec![0; 0],
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
                data: serialize(&fs_config),
                extra: vec![0; 0],
            };
        }
        UpdateMetadentry => {
            let update_data: UpdateMetadentryData = deserialize::<UpdateMetadentryData>(&post.data);
            if ENABLE_OUTPUT {
                println!("handling update metadentry of '{}'....", update_data.path);
            }
            let path = update_data.path.to_string();
            
            if ENABLE_PRECREATE {
                let chunk_start =
                    if let Some(md) = MetadataDB::get_instance().get(&path){
                        Metadata::deserialize(&md).get_size() as u64 / CHUNK_SIZE + 1
                    }
                    else {0};
                let chunk_end = (update_data.size + update_data.offset as u64) / CHUNK_SIZE;
                let path = update_data.path.clone().to_string();
                tokio::spawn(async move {
                    let mut hosts = HashMap::new();
                    let distributor = NetworkContext::get_instance().get_distributor();
                    for chunk_id in chunk_start..(chunk_end + 1) {
                        let host = distributor.locate_data(&path, chunk_id);
                        if !hosts.contains_key(&chunk_id) {
                            hosts.insert(host, Vec::new());
                        } else {
                            hosts.get_mut(&host).unwrap().push(chunk_id);
                        }
                    }
                    for (host, chunks) in hosts {
                        let endp = NetworkContext::get_instance()
                            .get_hosts()
                            .get(host as usize)
                            .unwrap();
                        let pre_create = PreCreateData {
                            path: path.as_str(),
                            chunks,
                        };
                        NetworkService::post::<PreCreateData>(endp, pre_create, PreCreate)
                            .await
                            .unwrap();
                    }
                });
            }
            MetadataDB::get_instance().increase_size(
                &path,
                update_data.size as usize + update_data.offset as usize,
                update_data.append,
            );
            return PostResult {
                err: 0,
                data: (update_data.size as usize + update_data.offset as usize)
                    .to_string()
                    .as_bytes()
                    .to_vec(),
                extra: vec![0; 0],
            };
        }
        GetMetadentry => {
            if ENABLE_OUTPUT {
                println!("handling get metadentry....");
            }
            let serde_string: SerdeString = deserialize::<SerdeString>(&post.data);
            let path = serde_string.str;
            let md_str = MetadataDB::get_instance().get(&path.to_string());
            match md_str {
                None => {
                    return PostResult {
                        err: ENOENT,
                        data: ENOENT.to_string().as_bytes().to_vec(),
                        extra: vec![0; 0],
                    };
                }
                Some(str) => {
                    let md = Metadata::deserialize(&str);
                    return PostResult {
                        err: 0,
                        data: md.get_size().to_string().as_bytes().to_vec(),
                        extra: vec![0; 0],
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
                data: serialize(&chunk_stat),
                extra: vec![0; 0],
            };
            return post_result;
        }
        DecrSize => {
            let decr_data: DecrData = deserialize::<DecrData>(&post.data);
            if ENABLE_OUTPUT {
                println!("handling decrease size of '{}'....", decr_data.path);
            }
            MetadataDB::get_instance()
                .decrease_size(&decr_data.path.to_string(), decr_data.new_size as usize);
            return PostResult {
                err: 0,
                data: "0".to_string().as_bytes().to_vec(),
                extra: vec![0; 0],
            };
        }
        Trunc => {
            let trunc_data: TruncData = deserialize::<TruncData>(&post.data);
            if ENABLE_OUTPUT {
                println!("handling truncate of '{}'....", trunc_data.path);
            }
            return handle_trunc(trunc_data);
        }
        GetDirents => {
            let data: DirentData = deserialize::<DirentData>(&post.data);
            let path = data.path;
            if ENABLE_OUTPUT {
                println!("handling get dirents of '{}'....", path);
            }
            let entries = MetadataDB::get_instance().get_dirents(&path.to_string());
            if entries.len() == 0 {
                return PostResult {
                    err: 0,
                    data: serialize(&(Vec::new() as Vec<(String, bool)>)),
                    extra: vec![0; 0],
                };
            } else {
                return PostResult {
                    err: 0,
                    data: serialize(&entries),
                    extra: vec![0; 0],
                };
            }
        }
        PreCreate => {
            let data: PreCreateData = deserialize::<PreCreateData>(&post.data);
            if ENABLE_OUTPUT {
                println!("handling precreate of '{}'....", data.path);
            }
            handle_precreate(&data);
            return PostResult {
                err: 0,
                data: vec![0; 0],
                extra: vec![0; 0],
            };
        }
        _ => {
            println!("invalid option on 'handle': {:?}", option);
            return PostResult {
                err: EINVAL,
                data: EINVAL.to_string().as_bytes().to_vec(),
                extra: vec![0; 0],
            };
        }
    }
}
#[derive(Clone, Default)]
struct ServerHandler {}
#[tonic::async_trait]
impl SfsHandle for ServerHandler {
    type handle_streamStream = ReceiverStream<Result<PostResult, Status>>;
    async fn handle(
        &self,
        request: tonic::Request<Post>,
    ) -> Result<tonic::Response<PostResult>, tonic::Status> {
        let post = request.into_inner();
        let handle_result = tokio::spawn(async move { handle_request(&post).await })
            .await
            .unwrap();
        return Ok(Response::new(handle_result));
    }
    async fn handle_stream(
        &self,
        request: Request<tonic::Streaming<Post>>,
    ) -> Result<Response<Self::handle_streamStream>, Status> {
        let mut streamer = request.into_inner();
        let (tx, rx) = mpsc::channel(2 * CHUNK_SIZE as usize);
        tokio::spawn(async move {
            while let Some(post) = streamer.message().await.unwrap() {
                let option = i2option(post.option);
                match option {
                    Read => {
                        let read_args: ReadData = deserialize::<ReadData>(&post.data);
                        if ENABLE_OUTPUT {
                            println!("handling read of '{}'....", read_args.path);
                        }
                        tx.send(Ok(handle_read(&read_args))).await.unwrap();
                    }
                    Write => {
                        let write_args: WriteData = deserialize::<WriteData>(&post.data);
                        if ENABLE_OUTPUT {
                            println!("handling write of '{}'....", write_args.path);
                        }
                        let data = post.extra;
                        tx.send(Ok(handle_write(&write_args, &data))).await.unwrap();
                    }
                    _ => {
                        println!("invalid option on 'handle_stream': {:?}", option);
                        tx.send(Ok(PostResult {
                            err: EINVAL,
                            data: EINVAL.to_string().as_bytes().to_vec(),
                            extra: vec![0; 0],
                        }))
                        .await
                        .unwrap();
                    }
                }
            }
        });
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
async fn init_server(addr: &String) -> Result<(), Error> {
    let server_addr: (Ipv4Addr, u16) = (addr.parse().unwrap(), 8082);
    println!("listening on {:?}", server_addr);
    let handler = ServerHandler::default();
    Server::builder()
        .concurrency_limit_per_connection(32)
        .max_concurrent_streams(12)
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
    ChunkStorage::get_instance();
    MetadataDB::get_instance();
    NetworkContext::get_instance();

    let mut root_md = Metadata::new();
    root_md.set_mode(S_IFDIR | S_IRWXU | S_IRWXG | S_IRWXO);

    MetadataDB::get_instance().put(&"/".to_string(), root_md.serialize(), IGNORE_IF_EXISTS);

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
    StorageContext::get_instance();

    init_environment().await?;
    Ok(())
}
