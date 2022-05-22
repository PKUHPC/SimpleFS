pub mod config;
pub mod error_msg;
pub mod handle;
pub mod server;
use crate::server::{config::IGNORE_IF_EXISTS, network::network_context::NetworkContext};
use crate::server::{
    filesystem::storage_context::StorageContext, storage::data::chunk_storage::*,
    storage::metadata::db::MetadataDB,
};
use config::ENABLE_PRECREATE;
use futures::channel::oneshot;
use futures::executor::block_on;
use futures::{FutureExt, SinkExt, TryFutureExt, TryStreamExt};
use grpcio::{Environment, ServerBuilder, WriteFlags};
use handle::handle_precreate;
use libc::{getgid, getuid, EINVAL, ENOENT, S_IFDIR, S_IRWXG, S_IRWXO, S_IRWXU};
use server::network::network_service::NetworkService;
use server::storage::data::stuff_db::StuffDB;
use sfs_global::global::distributor::Distributor;
use sfs_global::global::fsconfig::ENABLE_STUFFING;
use sfs_global::global::network::forward_data::PreCreateData;
use sfs_global::global::network::post::{i2option, post_result, PostOption};
use sfs_global::global::util::serde_util::{deserialize, serialize};
use sfs_global::global::util::arith_util::offset_to_chunk_id;
use sfs_global::{
    global::network::post::PostOption::*,
    global::{
        fsconfig::SFSConfig,
        metadata::Metadata,
        network::{
            config::CHUNK_SIZE,
            forward_data::{
                CreateData, DecrData, DirentData, ReadData, TruncData, UpdateMetadentryData,
                WriteData,
            },
        },
        util::net_util::get_my_hostname,
    },
};
use sfs_rdma::chunk_operation::ChunkOp;
use sfs_rdma::rdma::RDMA;
use sfs_rpc::proto::server::{Post, PostResult};
use sfs_rpc::proto::server_grpc::{create_sfs_handle, SfsHandle};
use std::collections::HashMap;
use std::io::Read;
use std::sync::Arc;
use std::thread;
use std::{
    fs::OpenOptions,
    io::{BufWriter, Error, Write},
    net::Ipv4Addr,
    path::Path,
};

use crate::handle::{handle_read, handle_trunc};

#[allow(unused)]
use std::time::Instant;

fn handle_request(post: &Post) -> PostResult {
    let option = i2option(post.option);
    match option {
        Stat => {
            let path = deserialize::<&str>(&post.data);
            if StorageContext::get_instance().output() {
                println!("handling metadata of '{}'....", path);
            }
            let md_res = MetadataDB::get_instance().get(&path.to_string());
            if let Some(md) = md_res {
                return post_result(0, md, vec![0; 0]);
            } else {
                return post_result(ENOENT, vec![0; 0], vec![0; 0]);
            }
        }
        Create => {
            let create_data: CreateData = deserialize::<CreateData>(&post.data);
            if StorageContext::get_instance().output() {
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
            return post_result(create_res, vec![0; 0], vec![0; 0]);
        }
        Remove => {
            let path = deserialize::<&str>(&post.data);
            if StorageContext::get_instance().output() {
                println!("handling remove of '{}'....", path);
            }
            ChunkStorage::destroy_chunk_space(&path.to_string());
            return post_result(0, vec![0; 0], vec![0; 0]);
        }
        RemoveMeta => {
            let path = deserialize::<&str>(&post.data);
            if StorageContext::get_instance().output() {
                println!("handling remove metadata of '{}'....", path);
            }
            let md_res = MetadataDB::get_instance().get(&path.to_string());
            if let None = md_res {
                return post_result(ENOENT, vec![0; 0], vec![0; 0]);
            } else {
                MetadataDB::get_instance().remove(&path.to_string());
                return post_result(0, vec![0; 0], vec![0; 0]);
            }
        }
        Lookup => {
            if StorageContext::get_instance().output() {
                println!("handling look up....");
            }
            return post_result(0, vec![0; 0], vec![0; 0]);
        }
        FsConfig => {
            if StorageContext::get_instance().output() {
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
            return post_result(0, serialize(&fs_config), vec![0; 0]);
        }
        UpdateMetadentry => {
            let update_data: UpdateMetadentryData = deserialize::<UpdateMetadentryData>(&post.data);
            if StorageContext::get_instance().output() {
                println!("handling update metadentry of '{}'....", update_data.path);
            }
            let path = update_data.path.to_string();

            MetadataDB::get_instance().increase_size(
                &path,
                update_data.size as usize + update_data.offset as usize,
                update_data.append,
            );
            let extra = vec![0; 0];
            if ENABLE_STUFFING {
                /*
                let md = Metadata::deserialize(&MetadataDB::get_instance().get(&path).unwrap());
                if md.is_stuffed() {
                    let write_tot = ChunkStorage::write_chunk(
                        &path,
                        0,
                        &post.extra,
                        update_data.size,
                        update_data.offset as u64,
                    );
                    if let Ok(len) = write_tot {
                        extra = serialize(len);
                    }
                }
                */
            }
            return post_result(
                0,
                serialize(update_data.size + update_data.offset as u64),
                extra,
            );
        }
        GetMetadentry => {
            if StorageContext::get_instance().output() {
                println!("handling get metadentry....");
            }
            let path = deserialize::<&str>(&post.data);
            let md_str = MetadataDB::get_instance().get(&path.to_string());
            match md_str {
                None => {
                    return post_result(ENOENT, vec![0; 0], vec![0; 0]);
                }
                Some(str) => {
                    let md = Metadata::deserialize(&str);
                    return post_result(0, serialize(md.get_size()), vec![0; 0]);
                }
            }
        }
        ChunkStat => {
            if StorageContext::get_instance().output() {
                println!("handling chunk stat....");
            }
            let chunk_stat = ChunkStorage::chunk_stat();
            let post_result = post_result(0, serialize(&chunk_stat), vec![0; 0]);
            return post_result;
        }
        DecrSize => {
            let decr_data: DecrData = deserialize::<DecrData>(&post.data);
            if StorageContext::get_instance().output() {
                println!("handling decrease size of '{}'....", decr_data.path);
            }
            MetadataDB::get_instance()
                .decrease_size(&decr_data.path.to_string(), decr_data.new_size as usize);
            return post_result(0, vec![0; 0], vec![0; 0]);
        }
        Trunc => {
            let trunc_data: TruncData = deserialize::<TruncData>(&post.data);
            if StorageContext::get_instance().output() {
                println!("handling truncate of '{}'....", trunc_data.path);
            }
            return handle_trunc(trunc_data);
        }
        PreCreate => {
            let data: PreCreateData = deserialize::<PreCreateData>(&post.data);
            if StorageContext::get_instance().output() {
                println!("handling precreate of '{}'....", data.path);
            }
            handle_precreate(&data);
            return post_result(0, vec![0; 0], vec![0; 0]);
        }
        Write => {
            let write_data: WriteData = deserialize::<WriteData>(&post.data);
            if StorageContext::get_instance().output() {
                println!("handling stream write of '{}'....", write_data.path);
                println!("  - {:?}", write_data);
            }
            let op = ChunkOp{
                path: write_data.path.to_string(),
                offset: write_data.offset as u64 % CHUNK_SIZE,
                chunk_start: offset_to_chunk_id(write_data.offset, CHUNK_SIZE),
                size: write_data.write_size,
                op: ChunkStorage::write_chunk,
            };
            let result = RDMA::recver_client(&write_data.rdma_addr.to_string(), write_data.rdma_port, op);
            if let Err(e) = result{
                return post_result(e, vec![0; 0], vec![0; 0]);
            }
            let write_tot = result.unwrap();
            return post_result(0, serialize(write_tot), vec![0; 0]);
        }
        _ => {
            println!("invalid option on 'handle': {:?}", option);
            return post_result(EINVAL, vec![0; 0], vec![0; 0]);
        }
    }
}
#[derive(Clone, Default)]
struct ServerHandler {}
impl SfsHandle for ServerHandler {
    fn handle(
        &mut self,
        ctx: grpcio::RpcContext,
        req: sfs_rpc::proto::server::Post,
        sink: grpcio::UnarySink<sfs_rpc::proto::server::PostResult>,
    ) {
        if ENABLE_PRECREATE {
            if let PostOption::UpdateMetadentry = i2option(req.option) {
                let update_data: UpdateMetadentryData =
                    deserialize::<UpdateMetadentryData>(&req.data);
                let chunk_start = if let Some(md) =
                    MetadataDB::get_instance().get(&update_data.path.to_string())
                {
                    Metadata::deserialize(&md).get_size() as u64 / CHUNK_SIZE + 1
                } else {
                    0
                };
                let chunk_end = (update_data.size + update_data.offset as u64) / CHUNK_SIZE;
                let path = update_data.path.clone().to_string();
                let f = async move {
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
                        let client = NetworkContext::get_instance()
                            .get_clients()
                            .get(host as usize)
                            .unwrap();
                        let pre_create = PreCreateData {
                            path: path.as_str(),
                            chunks,
                        };
                        NetworkService::post::<PreCreateData>(client, pre_create, PreCreate)
                            .unwrap();
                    }
                };
                ctx.spawn(f);
            }
        }
        let f = async move {
            let handle_result = handle_request(&req);
            sink.success(handle_result).await.unwrap();
        };
        ctx.spawn(f);
    }

    fn handle_stream(
        &mut self,
        ctx: grpcio::RpcContext,
        mut stream: grpcio::RequestStream<sfs_rpc::proto::server::Post>,
        mut sink: grpcio::DuplexSink<sfs_rpc::proto::server::PostResult>,
    ) {
        let f = async move {
            while let Some(post) = stream.try_next().await? {
                let option = i2option(post.option);
                match option {
                    Read => {
                        let read_args: ReadData = deserialize::<ReadData>(&post.data);
                        if StorageContext::get_instance().output() {
                            println!("handling stream read of '{}'....", read_args.path);
                        }
                        sink.send((handle_read(&read_args), WriteFlags::default()))
                            .await?;
                    }
                    _ => {
                        println!("invalid option on 'handle_stream': {:?}", option);
                        sink.send((
                            post_result(EINVAL, vec![0; 0], vec![0; 0]),
                            WriteFlags::default(),
                        ))
                        .await?;
                    }
                }
            }
            sink.close().await?;
            Ok(())
        }
        .map_err(|e: grpcio::Error| {
            println!("server::handle_stream failed to handle stream: {:?}", e);
        })
        .map(|_| ());
        ctx.spawn(f);
    }

    fn handle_dirents(
        &mut self,
        ctx: grpcio::RpcContext,
        req: sfs_rpc::proto::server::Post,
        mut sink: grpcio::ServerStreamingSink<sfs_rpc::proto::server::PostResult>,
    ) {
        let f = async move {
            let option = i2option(req.option);
            match option {
                GetDirents => {
                    let data: DirentData = deserialize::<DirentData>(&req.data);
                    let path = data.path;
                    if StorageContext::get_instance().output() {
                        println!("handling get dirents of '{}'....", path);
                    }
                    let entries = MetadataDB::get_instance().get_dirents(&path.to_string());
                    for entry in entries {
                        sink.send((
                            post_result(0, serialize(entry), vec![0; 0]),
                            WriteFlags::default(),
                        ))
                        .await?
                    }
                }
                _ => {
                    println!("invalid option on 'handle_dirents': {:?}", option);
                    sink.send((
                        post_result(EINVAL, vec![0; 0], vec![0; 0]),
                        WriteFlags::default(),
                    ))
                    .await?;
                }
            }
            sink.close().await?;
            Ok(())
        }
        .map_err(|e: grpcio::Error| {
            println!("server::handle_dirents failed to handle stream: {:?}", e);
        })
        .map(|_| ());
        ctx.spawn(f);
    }
}
async fn init_server(addr: &String) -> Result<(), Error> {
    let server_addr: (Ipv4Addr, u16) = (addr.parse().unwrap(), 8082);
    println!("listening on {:?}", server_addr);
    let env = Arc::new(Environment::new(48));
    let instance = ServerHandler {};
    let service = create_sfs_handle(instance);
    let mut server = ServerBuilder::new(env)
        .register_service(service)
        .bind(addr, 8082)
        .build()
        .unwrap();
    server.start();
    NetworkContext::get_instance();

    let (tx, rx) = oneshot::channel();
    thread::spawn(move || {
        println!("Press ENTER to exit...");
        let _ = std::io::stdin().read(&mut [0]).unwrap();
        tx.send(())
    });
    block_on(rx).unwrap();
    block_on(server.shutdown()).unwrap();
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
    StuffDB::get_instance();

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
