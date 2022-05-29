use std::collections::HashMap;
#[allow(unused)]
use std::slice;
use std::sync::{Arc, Mutex};
#[allow(unused)]
use std::time::Instant;

use futures::{TryStreamExt};
use grpcio::Error;
use libc::{c_char, EBUSY};
use sfs_global::global::util::serde_util::{deserialize, serialize};
use sfs_rdma::{RDMA_WRITE_PORT, RDMA_READ_PORT};
use sfs_rdma::chunk_operation::ChunkOp;
use sfs_rdma::transfer::{ChunkTransferTask, ChunkMetadata};
use sfs_rpc::post;
use sfs_rpc::proto::server::PostResult;

#[allow(unused)]
use crate::client::context::{DynamicContext, StaticContext};
use crate::client::openfile::{FileType, OpenFile, O_RDONLY};
use sfs_global::global::distributor::Distributor;
use sfs_global::global::error_msg::error_msg;
use sfs_global::global::fsconfig::SFSConfig;
use sfs_global::global::network::config::CHUNK_SIZE;
use sfs_global::global::network::forward_data::{
    ChunkStat, CreateData, DecrData, DirentData, TruncData,
    UpdateMetadentryData, //WriteData,
};
use sfs_global::global::network::post::{option2i, PostOption};
use sfs_global::global::util::arith_util::{
    block_index, offset_to_chunk_id,
};

use super::network_service::NetworkService;
use super::rdma_read::recver_client;
use super::rdma_write::sender_client;

pub fn forward_stat(path: &String) -> Result<Vec<u8>, i32> {
    let endp_id = StaticContext::get_instance()
        .get_distributor()
        .locate_file_metadata(path);
    let post_res = NetworkService::post::<&str>(
        StaticContext::get_instance()
            .get_clients()
            .get(endp_id as usize)
            .unwrap(),
        path.as_str(),
        PostOption::Stat,
    );
    if let Err(e) = post_res {
        error_msg(
            "client::network::forward_stat".to_string(),
            format!("error {} occurs while fetching file stat", e),
        );
        println!("{:?}", e);
        return Err(EBUSY);
    }
    let result = post_res.unwrap();
    if result.err != 0 {
        return Err(result.err);
    }
    return Ok(result.data);
}
pub fn forward_create(path: &String, mode: u32) -> Result<i32, Error> {
    let endp_id = StaticContext::get_instance()
        .get_distributor()
        .locate_file_metadata(path);
    let post_res = NetworkService::post::<CreateData>(
        StaticContext::get_instance()
            .get_clients()
            .get(endp_id as usize)
            .unwrap(),
        CreateData {
            path: path.as_str(),
            mode: mode,
        },
        PostOption::Create,
    );
    if let Err(e) = post_res {
        error_msg(
            "client::network::forward_create".to_string(),
            format!("error {} occurs while fetching file stat", e),
        );
        return Ok(EBUSY);
    } else {
        let result = post_res.unwrap();
        if result.err != 0 {
            return Ok(result.err);
        }
        return Ok(0);
    }
}
pub fn forward_remove(
    path: &String,
    remove_metadentry_only: bool,
    size: i64,
) -> Result<i32, Error> {
    let endp_id = StaticContext::get_instance()
        .get_distributor()
        .locate_file_metadata(path);
    let _post_res = NetworkService::post::<&str>(
        StaticContext::get_instance()
            .get_clients()
            .get(endp_id as usize)
            .unwrap(),
        path.as_str(),
        PostOption::RemoveMeta,
    )?;
    if remove_metadentry_only {
        return Ok(0);
    }
    let mut posts = Vec::new();
    if (size / CHUNK_SIZE as i64) < StaticContext::get_instance().get_hosts().len() as i64 {
        let meta_host_id = StaticContext::get_instance()
            .get_distributor()
            .locate_file_metadata(path);

        let chunk_start = 0;
        let chunk_end = size as u64 / CHUNK_SIZE;
        posts.push((
            StaticContext::get_instance()
                .get_clients()
                .get(meta_host_id as usize)
                .unwrap(),
            post(
                option2i(&PostOption::Remove),
                serialize(path.as_str()),
                vec![0; 0],
            ),
        ));

        for chunk_id in chunk_start..(chunk_end + 1) {
            let chunk_host_id = StaticContext::get_instance()
                .get_distributor()
                .locate_data(path, chunk_id);
            if chunk_host_id == meta_host_id {
                continue;
            }
            posts.push((
                StaticContext::get_instance()
                    .get_clients()
                    .get(chunk_host_id as usize)
                    .unwrap(),
                post(
                    option2i(&PostOption::Remove),
                    serialize(path.as_str()),
                    vec![0; 0],
                ),
            ));
        }
    } else {
        for client in StaticContext::get_instance().get_clients().iter() {
            posts.push((
                client,
                post(
                    option2i(&PostOption::Remove),
                    serialize(path.as_str()),
                    vec![0; 0],
                ),
            ));
        }
    }
    let post_results = NetworkService::group_post(posts);
    if let Err(e) = post_results {
        return Err(e);
    } else {
        let result_vec = post_results.unwrap();
        for result in result_vec {
            if result.err != 0 {
                return Ok(result.err);
            }
        }
    }
    Ok(0)
}
pub fn forward_get_chunk_stat() -> (i32, ChunkStat) {
    let mut posts = Vec::new();
    for client in StaticContext::get_instance().get_clients().iter() {
        posts.push((
            client,
            post(
                option2i(&PostOption::ChunkStat),
                "0".as_bytes().to_vec(),
                vec![0; 0],
            ),
        ));
    }
    let chunk_size = CHUNK_SIZE;
    let mut chunk_total = 0;
    let mut chunk_free = 0;
    let post_results = NetworkService::group_post(posts);
    if let Err(_e) = post_results {
        return (-1, ChunkStat::new());
    } else {
        let result_vec = post_results.unwrap();
        for result in result_vec {
            if result.err != 0 {
                return (result.err, ChunkStat::new());
            }
            let chunk_stat: ChunkStat = deserialize::<ChunkStat>(&result.data);
            assert_eq!(chunk_stat.chunk_size, chunk_size);
            chunk_total += chunk_stat.chunk_total;
            chunk_free += chunk_stat.chunk_free;
        }
    }
    (
        0,
        ChunkStat {
            chunk_size,
            chunk_total,
            chunk_free,
        },
    )
}
pub fn forward_get_metadentry_size(path: &String) -> (i32, i64) {
    let post_result = NetworkService::post::<&str>(
        StaticContext::get_instance()
            .get_clients()
            .get(
                StaticContext::get_instance()
                    .get_distributor()
                    .locate_file_metadata(&path) as usize,
            )
            .unwrap(),
        path.as_str(),
        PostOption::UpdateMetadentry,
    );
    if let Err(_e) = post_result {
        return (-1, 0);
    } else {
        let result = post_result.unwrap();
        if result.err != 0 {
            return (result.err, 0);
        }
        return (0, deserialize::<i64>(&result.data));
    }
}
pub fn forward_decr_size(path: &String, new_size: i64) -> i32 {
    let host_id = StaticContext::get_instance()
        .get_distributor()
        .locate_file_metadata(&path);
    let post_result = NetworkService::post::<DecrData>(
        StaticContext::get_instance()
            .get_clients()
            .get(host_id as usize)
            .unwrap(),
        DecrData {
            path: path.as_str(),
            new_size,
        },
        PostOption::DecrSize,
    );
    if let Err(_e) = post_result {
        return -1;
    } else {
        let result = post_result.unwrap();
        if result.err != 0 {
            return result.err;
        }
        return 0;
    }
}
pub fn forward_truncate(path: &String, old_size: i64, new_size: i64) -> i32 {
    if old_size < new_size {
        return -1;
    }
    let chunk_start = block_index(new_size, CHUNK_SIZE);
    let chunk_end = block_index(old_size - new_size - 1, CHUNK_SIZE);
    let mut hosts: Vec<u64> = Vec::new();
    for chunk_id in chunk_start..(chunk_end + 1) {
        let host_id = StaticContext::get_instance()
            .get_distributor()
            .locate_data(path, chunk_id);
        if !hosts.contains(&host_id) {
            hosts.push(host_id);
        }
    }
    let mut posts = Vec::new();
    for host in hosts {
        let trunc_data = TruncData {
            path: path.as_str(),
            new_size,
        };
        let post = post(
            option2i(&PostOption::Trunc),
            serialize(&trunc_data),
            vec![0; 0],
        );
        posts.push((
            StaticContext::get_instance()
                .get_clients()
                .get(host as usize)
                .unwrap(),
            post,
        ));
    }
    let post_results = NetworkService::group_post(posts);
    if let Err(_e) = post_results {
        return EBUSY;
    }
    let results = post_results.unwrap();
    for result in results {
        if result.err != 0 {
            return result.err;
        }
    }

    return 0;
}
pub fn forward_update_metadentry_size(
    path: &String,
    size: u64,
    offset: i64,
    append_flag: bool,
    stuff: Vec<u8>,
) -> (i32, i64) {
    let update_data = UpdateMetadentryData {
        path: path.as_str(),
        size,
        offset,
        append: append_flag,
    };
    let host_id = StaticContext::get_instance()
        .get_distributor()
        .locate_file_metadata(&path);
    let post_result = NetworkService::post_stuff::<UpdateMetadentryData>(
        StaticContext::get_instance()
            .get_clients()
            .get(host_id as usize)
            .unwrap(),
        update_data,
        stuff,
        PostOption::UpdateMetadentry,
    );
    if let Err(_e) = post_result {
        return (EBUSY, 0);
    } else {
        let res = post_result.unwrap();
        // stuffing enabled and the file is stuffed
        if res.extra.len() != 0 {
            return (
                if res.err != 0 { res.err } else { -1 },
                deserialize::<u64>(&res.extra) as i64,
            );
        }
        return (
            if res.err != 0 { res.err } else { 0 },
            deserialize::<i64>(&res.data),
        );
    }
}
pub async fn forward_write(
    path: &String,
    buf: *const c_char,
    append_flag: bool,
    in_offset: i64,
    write_size: i64,
    updated_metadentry_size: i64,
) -> (i32, i64) {
    if write_size < 0 {
        return (-1, 0);
    }
    let offset = if append_flag {
        in_offset
    } else {
        updated_metadentry_size - write_size
    };
    let chunk_start = offset_to_chunk_id(offset, CHUNK_SIZE);
    let chunk_end = offset_to_chunk_id(offset + write_size - 1, CHUNK_SIZE);
    let mut target_chunks: HashMap<u64, Vec<u64>> = HashMap::new();
    let mut targets: Vec<u64> = Vec::new();

    for chunk_id in chunk_start..(chunk_end + 1) {
        let target = StaticContext::get_instance()
            .get_distributor()
            .locate_data(path, chunk_id);
        if !target_chunks.contains_key(&target) {
            target_chunks.insert(target, Vec::new());
            target_chunks.get_mut(&target).unwrap().push(chunk_id);
            targets.push(target);
        } else {
            target_chunks.get_mut(&target).unwrap().push(chunk_id);
        }
    }
    let mut tot_write = 0;
    //let buf = unsafe { CStr::from_ptr(buf).to_string_lossy().into_owned() };
    //let buf = unsafe { slice::from_raw_parts(buf as *const u8, write_size as usize) };
    //let mut handles = Vec::new();
    let mut rdma_handles = Vec::new();
    for target in targets {
        let addr = buf as u64;
        let chunk_ids = target_chunks.remove(&target).unwrap();
        let op = ChunkOp::none();
        let chunk_transfer = ChunkTransferTask {
            chunk_id: chunk_ids,
            metadata: ChunkMetadata{
                path: path.to_string(),
                chunk_start: chunk_start as u64,
                offset: offset as u64 % CHUNK_SIZE,
                size: write_size as u64,
            },
            addr: addr,
        };
        rdma_handles.push(
                sender_client(&StaticContext::get_instance().get_hosts().get(target as usize).unwrap().addr, RDMA_WRITE_PORT, chunk_transfer, op).await
        )
    }
    for rdma in rdma_handles {
        let res = rdma.await.unwrap();
        if let Err(e) = res{
            return (e, 0);
        }
        tot_write += res.unwrap();
    }
    return (0, tot_write);
}
pub async fn forward_read(
    path: &String,
    buf: *mut c_char,
    offset: i64,
    read_size: i64,
) -> (i32, u64) {
    let chunk_start = offset_to_chunk_id(offset, CHUNK_SIZE);
    let chunk_end = offset_to_chunk_id(offset + read_size - 1, CHUNK_SIZE);
    let mut target_chunks: HashMap<u64, Vec<u64>> = HashMap::new();
    let mut targets: Vec<u64> = Vec::new();

    for chunk_id in chunk_start..(chunk_end + 1) {
        let target = StaticContext::get_instance()
            .get_distributor()
            .locate_data(path, chunk_id);
        if !target_chunks.contains_key(&target) {
            target_chunks.insert(target, Vec::new());
            target_chunks.get_mut(&target).unwrap().push(chunk_id);
            targets.push(target);
        } else {
            target_chunks.get_mut(&target).unwrap().push(chunk_id);
        }
    }
    let mut tot_read = 0;
    let mut rdma_handles = Vec::new();
    for target in targets {
        let addr = buf as u64;
        let chunk_ids = target_chunks.remove(&target).unwrap();
        let op = ChunkOp::none();
        let chunk_transfer = ChunkTransferTask {
            chunk_id: chunk_ids,
            metadata: ChunkMetadata{
                path: path.to_string(),
                chunk_start: chunk_start as u64,
                offset: offset as u64 % CHUNK_SIZE,
                size: read_size as u64,
            },
            addr: addr,
        };
        rdma_handles.push(
            recver_client(&StaticContext::get_instance().get_hosts().get(target as usize).unwrap().addr, RDMA_READ_PORT, chunk_transfer, op).await
        )
    }
    for rdma in rdma_handles {
        let res = rdma.await.unwrap();
        if let Err(e) = res{
            return (e, 0);
        }
        tot_read += res.unwrap();
    }
    return (0, tot_read as u64);
}
pub async fn forward_get_dirents(path: &String) -> (i32, Arc<Mutex<OpenFile>>) {
    let targets = StaticContext::get_instance()
        .get_distributor()
        .locate_dir_metadata(path);
    //let buf: Box<[u8; DIRENT_BUF_SIZE as usize]> = Box::new([0; DIRENT_BUF_SIZE as usize]);
    //let buf_size_per_host = DIRENT_BUF_SIZE / targets.len() as u64;
    let mut posts = Vec::new();
    for target in targets.iter() {
        posts.push((
            StaticContext::get_instance()
                .get_clients()
                .get(*target as usize)
                .unwrap(),
            post(
                option2i(&PostOption::GetDirents),
                serialize(&DirentData {
                    path: path.as_str(),
                }),
                vec![0; 0],
            ),
        ));
    }
    let post_results: Result<Vec<PostResult>, Error> = {
        let mut post_results = Vec::new();
        for (client, post) in posts {
            let mut receiver = client.handle_dirents(&post).unwrap();
            while let Some(res) = receiver.try_next().await.unwrap() {
                post_results.push(res);
            }
        }
        Ok(post_results)
    };
    if let Err(_e) = post_results {
        println!("fail to get dirents: {} {:?}", path, _e);
        return (
            EBUSY,
            Arc::new(Mutex::new(OpenFile::new(
                &"".to_string(),
                0,
                crate::client::openfile::FileType::SFS_REGULAR,
            ))),
        );
    }
    let results: Vec<PostResult> = post_results.unwrap();
    let mut open_dir = OpenFile::new(
        path,
        O_RDONLY,
        crate::client::openfile::FileType::SFS_DIRECTORY,
    );

    for result in results {
        let entry: (String, bool) = deserialize::<(String, bool)>(&result.data);
        open_dir.add(
            entry.0,
            if entry.1 {
                FileType::SFS_DIRECTORY
            } else {
                FileType::SFS_REGULAR
            },
        );
    }
    (0, Arc::new(Mutex::new(open_dir)))
}
pub fn forward_get_fs_config(context: &mut StaticContext) -> bool {
    let host_id = context.get_local_host_id();
    let post = post(option2i(&PostOption::FsConfig), vec![0; 0], vec![0; 0]);
    let client = context.get_clients().get(host_id as usize).unwrap();
    let fsconf_res = client.handle(&post);
    if let Err(_e) = fsconf_res {
        return false;
    }
    let result = fsconf_res.unwrap();
    if result.err != 0 {
        return false;
    }
    let config: SFSConfig = deserialize::<SFSConfig>(&result.data);
    context.set_mountdir(config.mountdir.clone());
    context.set_fsconfig(config);
    return true;
}
