use std::collections::HashMap;
use std::io::Error;
use std::slice;
use std::sync::{Arc, Mutex};
#[allow(unused)]
use std::time::Instant;

use futures::stream::iter;
use libc::{c_char, c_void, memcpy, EBUSY};
use sfs_global::global::endpoint::SFSEndpoint;
use sfs_global::global::util::serde_util::{deserialize, serialize};
use sfs_rpc::sfs_server::sfs_handle_client::SfsHandleClient;

use crate::client::context::StaticContext;
use crate::client::openfile::{FileType, OpenFile, O_RDONLY};
use sfs_global::global::distributor::Distributor;
use sfs_global::global::error_msg::error_msg;
use sfs_global::global::fsconfig::SFSConfig;
use sfs_global::global::network::config::CHUNK_SIZE;
use sfs_global::global::network::forward_data::{
    ChunkStat, CreateData, DecrData, DirentData, ReadData, ReadResult, SerdeString, TruncData,
    UpdateMetadentryData, WriteData,
};
use sfs_global::global::network::post::{option2i, PostOption};
use sfs_global::global::util::arith_util::{
    block_index, chunk_lpad, chunk_rpad, offset_to_chunk_id,
};
use sfs_rpc::sfs_server::{Post, PostResult};

use super::network_service::NetworkService;

pub fn forward_stat(path: &String) -> Result<Vec<u8>, i32> {
    let endp_id = StaticContext::get_instance()
        .get_distributor()
        .locate_file_metadata(path);
    let post_res = NetworkService::post::<SerdeString>(
        StaticContext::get_instance()
            .get_hosts()
            .get(endp_id as usize)
            .unwrap(),
        SerdeString { str: path.as_str() },
        PostOption::Stat,
    );
    if let Err(e) = post_res {
        error_msg(
            "client::network::forward_stat".to_string(),
            format!("error {} occurs while fetching file stat", e),
        );
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
            .get_hosts()
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
pub fn forward_remove(path: String, remove_metadentry_only: bool, size: i64) -> Result<i32, Error> {
    let endp_id = StaticContext::get_instance()
        .get_distributor()
        .locate_file_metadata(&path);
    let _post_res = NetworkService::post::<SerdeString>(
        StaticContext::get_instance()
            .get_hosts()
            .get(endp_id as usize)
            .unwrap(),
        SerdeString { str: path.as_str() },
        PostOption::RemoveMeta,
    )?;
    if remove_metadentry_only {
        return Ok(0);
    }
    let mut posts: Vec<(SFSEndpoint, Post)> = Vec::new();
    if (size / CHUNK_SIZE as i64) < StaticContext::get_instance().get_hosts().len() as i64 {
        let meta_host_id = StaticContext::get_instance()
            .get_distributor()
            .locate_file_metadata(&path);

        let chunk_start = 0;
        let chunk_end = size as u64 / CHUNK_SIZE;
        posts.push((
            StaticContext::get_instance()
                .get_hosts()
                .get(meta_host_id as usize)
                .unwrap()
                .clone(),
            Post {
                option: option2i(&PostOption::Remove),
                data: serialize(&SerdeString { str: path.as_str() }),
                extra: vec![0; 0],
            },
        ));

        for chunk_id in chunk_start..(chunk_end + 1) {
            let chunk_host_id = StaticContext::get_instance()
                .get_distributor()
                .locate_data(&path, chunk_id);
            if chunk_host_id == meta_host_id {
                continue;
            }
            posts.push((
                StaticContext::get_instance()
                    .get_hosts()
                    .get(chunk_host_id as usize)
                    .unwrap()
                    .clone(),
                Post {
                    option: option2i(&PostOption::Remove),
                    data: serialize(&SerdeString { str: path.as_str() }),
                    extra: vec![0; 0],
                },
            ));
        }
    } else {
        for endp in StaticContext::get_instance().get_hosts().iter() {
            posts.push((
                endp.clone(),
                Post {
                    option: option2i(&PostOption::Remove),
                    data: serialize(&SerdeString { str: path.as_str() }),
                    extra: vec![0; 0],
                },
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
    let mut posts: Vec<(SFSEndpoint, Post)> = Vec::new();
    for endp in StaticContext::get_instance().get_hosts().iter() {
        posts.push((
            endp.clone(),
            Post {
                option: option2i(&PostOption::ChunkStat),
                data: "0".as_bytes().to_vec(),
                extra: vec![0; 0],
            },
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
    let post_result = NetworkService::post::<SerdeString>(
        StaticContext::get_instance()
            .get_hosts()
            .get(
                StaticContext::get_instance()
                    .get_distributor()
                    .locate_file_metadata(&path) as usize,
            )
            .unwrap(),
        SerdeString { str: path.as_str() },
        PostOption::UpdateMetadentry,
    );
    if let Err(_e) = post_result {
        return (-1, 0);
    } else {
        let result = post_result.unwrap();
        if result.err != 0 {
            return (result.err, 0);
        }
        return (
            0,
            String::from_utf8(result.data)
                .unwrap()
                .as_str()
                .parse::<i64>()
                .unwrap(),
        );
    }
}
pub fn forward_decr_size(path: &String, new_size: i64) -> i32 {
    let host_id = StaticContext::get_instance()
        .get_distributor()
        .locate_file_metadata(&path) as usize;
    let post_result = NetworkService::post::<DecrData>(
        StaticContext::get_instance()
            .get_hosts()
            .get(host_id)
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
    let mut posts: Vec<(SFSEndpoint, Post)> = Vec::new();
    for host in hosts {
        let trunc_data = TruncData {
            path: path.as_str(),
            new_size,
        };
        let post = Post {
            option: option2i(&PostOption::Trunc),
            data: serialize(&trunc_data),
            extra: vec![0; 0],
        };
        posts.push((
            StaticContext::get_instance()
                .get_hosts()
                .get(host as usize)
                .unwrap()
                .clone(),
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
) -> (i32, i64) {
    let update_data = UpdateMetadentryData {
        path: path.as_str(),
        size,
        offset,
        append: append_flag,
    };
    let host_id = StaticContext::get_instance()
        .get_distributor()
        .locate_file_metadata(&path) as usize;
    let post_result = NetworkService::post::<UpdateMetadentryData>(
        StaticContext::get_instance()
            .get_hosts()
            .get(host_id)
            .unwrap(),
        update_data,
        PostOption::UpdateMetadentry,
    );
    if let Err(_e) = post_result {
        return (EBUSY, 0);
    } else {
        let res = post_result.unwrap();
        return (
            if res.err != 0 { res.err } else { 0 },
            String::from_utf8(res.data)
                .unwrap()
                .as_str()
                .parse::<i64>()
                .unwrap(),
        );
    }
}
#[tokio::main]
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
    let chunk_start = offset_to_chunk_id(offset.clone(), CHUNK_SIZE);
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
    let buf = unsafe { slice::from_raw_parts(buf as *const u8, write_size as usize) };
    let mut handles = Vec::new();
    for target in targets.iter() {
        let mut posts = Vec::new();
        for chunk in target_chunks.get(&target).unwrap() {
            let total_size = if *chunk == chunk_start {
                if *chunk == chunk_end {
                    write_size as u64
                } else {
                    chunk_rpad(offset, CHUNK_SIZE)
                }
            } else if *chunk == chunk_end {
                let pad = chunk_lpad(offset + write_size, CHUNK_SIZE);
                if pad == 0 {
                    CHUNK_SIZE
                } else {
                    pad
                }
            } else {
                CHUNK_SIZE
            };

            let offset_start = if *chunk == chunk_start {
                0
            } else {
                (*chunk - chunk_start) * CHUNK_SIZE - offset as u64 % CHUNK_SIZE
            } as usize;

            let offset_end = if *chunk == chunk_end {
                write_size as u64
            } else if *chunk == chunk_start {
                chunk_rpad(offset, CHUNK_SIZE)
            } else {
                (*chunk - chunk_start + 1) * CHUNK_SIZE - offset as u64 % CHUNK_SIZE
            } as usize;
            let data = WriteData {
                path: path.as_str(),
                offset: if *chunk == chunk_start {
                    chunk_lpad(offset, CHUNK_SIZE) as i64
                } else {
                    0
                },
                chunk_id: *chunk,
                write_size: total_size,
            };
            posts.push(Post {
                option: option2i(&PostOption::Write),
                data: serialize(&data),
                extra: buf[offset_start..offset_end].to_vec(),
            });
        }
        let addr = &StaticContext::get_instance()
            .get_hosts()
            .get(target.clone() as usize)
            .unwrap()
            .addr;
        handles.push(tokio::spawn(async move {
            let mut client = SfsHandleClient::connect(format!("http://{}:{}", addr, 8082))
                .await
                .unwrap();
            let request = tonic::Request::new(iter(posts));
            return client.handle_stream(request).await;
        }));
    }
    for handle in handles {
        let post_result = handle.await.unwrap();
        if let Err(_e) = post_result {
            return (EBUSY, tot_write);
        }
        let mut response = post_result.unwrap().into_inner();
        while let Some(res) = response.message().await.unwrap() {
            if res.err != 0 {
                return (res.err, tot_write);
            }
            tot_write += String::from_utf8(res.data)
                .unwrap()
                .as_str()
                .parse::<i64>()
                .expect("response should be 'i64'");
        }
    }
    return (0, tot_write);
}
#[tokio::main]
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
    let mut handles = Vec::new();
    for target in targets {
        let mut read_datas: Vec<ReadData> = Vec::new();
        for chunk in target_chunks.get(&target).unwrap() {
            let total_size = if *chunk == chunk_start {
                if *chunk == chunk_end {
                    read_size as u64
                } else {
                    chunk_rpad(offset, CHUNK_SIZE)
                }
            } else if *chunk == chunk_end {
                let pad = chunk_lpad(offset + read_size, CHUNK_SIZE);
                if pad == 0 {
                    CHUNK_SIZE
                } else {
                    pad
                }
            } else {
                CHUNK_SIZE
            };
            read_datas.push(ReadData {
                path: path.as_str(),
                offset: if *chunk == chunk_start {
                    chunk_lpad(offset, CHUNK_SIZE) as i64
                } else {
                    0
                },
                chunk_id: *chunk,
                read_size: total_size,
            })
        }
        let posts = read_datas
            .iter()
            .map(|x| Post {
                option: option2i(&PostOption::Read),
                data: serialize(&x),
                extra: vec![0; 0],
            })
            .collect::<Vec<_>>();
        let request = tonic::Request::new(iter(posts));

        handles.push(tokio::spawn(async move {
            let mut client = SfsHandleClient::connect(format!(
                "http://{}:{}",
                StaticContext::get_instance()
                    .get_hosts()
                    .get(target.clone() as usize)
                    .unwrap()
                    .addr,
                8082
            ))
            .await
            .unwrap();
            client.handle_stream(request).await
        }));
    }
    for handle in handles {
        let post_result = handle.await.unwrap();
        if let Err(_e) = post_result {
            return (EBUSY, tot_read);
        }
        let mut response = post_result.unwrap().into_inner();
        while let Some(res) = response.message().await.unwrap() {
            if res.err != 0 {
                return (res.err, tot_read);
            }
            let read_res: ReadResult = deserialize::<ReadResult>(&res.data);
            tot_read += read_res.nreads;
            let data = res.extra;
            let local_offset = if read_res.chunk_id == chunk_start {
                0
            } else {
                (read_res.chunk_id - chunk_start) * CHUNK_SIZE - (offset as u64 % CHUNK_SIZE)
            };
            unsafe {
                memcpy(
                    buf.offset(local_offset as isize) as *mut c_void,
                    data.as_ptr() as *const c_void,
                    data.len(),
                );
            }
        }
    }
    return (0, tot_read);
}
pub fn forward_get_dirents(path: &String) -> (i32, Arc<Mutex<OpenFile>>) {
    let targets = StaticContext::get_instance()
        .get_distributor()
        .locate_dir_metadata(path);
    //let buf: Box<[u8; DIRENT_BUF_SIZE as usize]> = Box::new([0; DIRENT_BUF_SIZE as usize]);
    //let buf_size_per_host = DIRENT_BUF_SIZE / targets.len() as u64;
    let mut posts: Vec<(SFSEndpoint, Post)> = Vec::new();
    for target in targets.iter() {
        posts.push((
            StaticContext::get_instance()
                .get_hosts()
                .get(*target as usize)
                .unwrap()
                .clone(),
            Post {
                option: option2i(&PostOption::GetDirents),
                data: serialize(&DirentData {
                    path: path.as_str(),
                }),
                extra: vec![0; 0],
            },
        ));
    }
    let post_results = NetworkService::group_post(posts);
    if let Err(_e) = post_results {
        return (
            -1,
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
        let entries: Vec<(String, bool)> = deserialize::<Vec<(String, bool)>>(&result.data);
        for entry in entries {
            open_dir.add(
                entry.0,
                if entry.1 {
                    FileType::SFS_DIRECTORY
                } else {
                    FileType::SFS_REGULAR
                },
            );
        }
    }
    (0, Arc::new(Mutex::new(open_dir)))
}
pub fn forward_get_fs_config(context: &mut StaticContext) -> bool {
    let host_id = context.get_local_host_id() as usize;
    let fsconf_res = NetworkService::post(
        context.get_hosts().get(host_id).unwrap(),
        (),
        PostOption::FsConfig,
    );
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
