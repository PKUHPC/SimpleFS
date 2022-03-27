use std::collections::HashMap;
use std::ffi::CStr;
use std::io::Error;
use std::sync::{Arc, Mutex};

use libc::{c_char, strncpy, EBUSY};

use crate::client::endpoint::SFSEndpoint;
use crate::client::openfile::{FileType, OpenFile, O_RDONLY};
use crate::client::{context::StaticContext, network::network_service::NetworkService};
use crate::global::distributor::Distributor;
use crate::global::error_msg::error_msg;
use crate::global::fsconfig::SFSConfig;
use crate::global::network::config::CHUNK_SIZE;
use crate::global::network::forward_data::{
    ChunkStat, CreateData, DecrData, DirentData, ReadData, ReadResult, SerdeString, TruncData,
    UpdateMetadentryData, WriteData,
};
use crate::global::network::post::{option2i, PostOption};
use crate::global::util::arith_util::{block_index, chunk_lpad, chunk_rpad, offset_to_chunk_id};
use sfs_rpc::sfs_server::{Post, PostResult};

pub fn forward_stat(path: &String) -> Result<String, Error> {
    let endp_id = StaticContext::get_instance()
        .get_distributor()
        .locate_file_metadata(path);
    let post_res = NetworkService::post::<SerdeString>(
        StaticContext::get_instance()
            .get_hosts()
            .get(endp_id as usize)
            .unwrap(),
        SerdeString { str: path.clone() },
        PostOption::Stat,
    );
    if let Err(e) = post_res {
        error_msg(
            "client::network::forward_stat".to_string(),
            format!("error {} occurs while fetching file stat", e),
        );
        return Err(e);
    }
    let result = post_res.unwrap();
    if result.err {
        return Err(Error::new(
            std::io::ErrorKind::NotFound,
            "metadata not exist",
        ));
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
            path: path.clone(),
            mode: mode,
        },
        PostOption::Create,
    );
    if let Err(e) = post_res {
        error_msg(
            "client::network::forward_create".to_string(),
            format!("error {} occurs while fetching file stat", e),
        );
        return Err(e);
    } else {
        let result = post_res.unwrap();
        if result.err {
            return Ok(result.data.as_str().parse::<i32>().unwrap());
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
        SerdeString { str: path.clone() },
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
                option: option2i(PostOption::Remove),
                data: serde_json::to_string(&SerdeString { str: path.clone() }).unwrap(),
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
                    option: option2i(PostOption::Remove),
                    data: serde_json::to_string(&SerdeString { str: path.clone() }).unwrap(),
                },
            ));
        }
    } else {
        for endp in StaticContext::get_instance().get_hosts().iter() {
            posts.push((
                endp.clone(),
                Post {
                    option: option2i(PostOption::Remove),
                    data: serde_json::to_string(&SerdeString { str: path.clone() }).unwrap(),
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
            if result.err {
                return Ok(result.data.as_str().parse::<i32>().unwrap());
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
                option: option2i(PostOption::ChunkStat),
                data: "0".to_string(),
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
            if result.err {
                return (
                    result.data.as_str().parse::<i32>().unwrap(),
                    ChunkStat::new(),
                );
            }
            let chunk_stat: ChunkStat = serde_json::from_str(&result.data).unwrap();
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
        SerdeString { str: path.clone() },
        PostOption::UpdateMetadentry,
    );
    if let Err(_e) = post_result {
        return (-1, 0);
    } else {
        let result = post_result.unwrap();
        if result.err {
            return (result.data.as_str().parse::<i32>().unwrap(), 0);
        }
        return (0, result.data.as_str().parse::<i64>().unwrap());
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
            path: path.clone(),
            new_size,
        },
        PostOption::DecrSize,
    );
    if let Err(_e) = post_result {
        return -1;
    } else {
        let result = post_result.unwrap();
        if result.err {
            return result.data.as_str().parse::<i32>().unwrap();
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
            path: path.clone(),
            new_size,
        };
        let post = Post {
            option: option2i(PostOption::Trunc),
            data: serde_json::to_string(&trunc_data).unwrap(),
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
        return -1;
    }
    let results = post_results.unwrap();
    for result in results {
        if result.err {
            return 5;
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
        path: path.clone(),
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
            if res.err {
                res.data.as_str().parse::<i32>().unwrap()
            } else {
                0
            },
            res.data.as_str().parse::<i64>().unwrap(),
        );
    }
}
pub fn forward_write(
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

    let mut chunk_start_target = 0;
    let mut chunk_end_target = 0;
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
        if chunk_id == chunk_start {
            chunk_start_target = target;
        }
        if chunk_id == chunk_end {
            chunk_end_target = target;
        }
    }
    let mut tot_write = 0;
    for target in targets {
        let mut tot_chunk_size = target_chunks.get(&target).unwrap().len() as u64 * CHUNK_SIZE;
        if target == chunk_start_target {
            tot_chunk_size -= chunk_lpad(offset, CHUNK_SIZE);
        }
        if target == chunk_end_target {
            tot_chunk_size -= chunk_rpad(offset + write_size, CHUNK_SIZE);
        }

        let input = WriteData {
            path: path.clone(),
            offset: chunk_lpad(offset, CHUNK_SIZE) as i64,
            host_id: target,
            host_size: StaticContext::get_instance().get_hosts().len() as u64,
            chunk_n: target_chunks.get(&target).unwrap().len() as u64,
            chunk_start: chunk_start,
            chunk_end: chunk_end,
            total_chunk_size: tot_chunk_size,
            buffers: unsafe { CStr::from_ptr(buf).to_string_lossy().into_owned() },
        };
        if let Ok(p) = NetworkService::post::<WriteData>(
            StaticContext::get_instance()
                .get_hosts()
                .get(target as usize)
                .unwrap(),
            input,
            PostOption::Write,
        ) {
            if p.err {
                return (-1, 0);
            }
            tot_write += p
                .data
                .as_str()
                .parse::<i64>()
                .expect("response should be 'i64'");
        } else {
            return (-1, 0);
        }
    }
    return (0, tot_write);
}
pub fn forward_read(path: &String, buf: *mut c_char, offset: i64, read_size: i64) -> (i32, u64) {
    let chunk_start = offset_to_chunk_id(offset, CHUNK_SIZE);
    let chunk_end = offset_to_chunk_id(offset + read_size - 1, CHUNK_SIZE);
    let mut target_chunks: HashMap<u64, Vec<u64>> = HashMap::new();
    let mut targets: Vec<u64> = Vec::new();

    let mut chunk_start_target = 0;
    let mut chunk_end_target = 0;
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
        if chunk_id == chunk_start {
            chunk_start_target = target;
        }
        if chunk_id == chunk_end {
            chunk_end_target = target;
        }
    }

    let mut tot_read = 0;
    for target in targets {
        let mut tot_chunk_size = target_chunks.get(&target).unwrap().len() as u64 * CHUNK_SIZE;
        if target == chunk_start_target {
            tot_chunk_size -= chunk_lpad(offset, CHUNK_SIZE);
        }
        if target == chunk_end_target {
            tot_chunk_size -= chunk_rpad(offset + read_size, CHUNK_SIZE);
        }

        let input = ReadData {
            path: path.clone(),
            offset: chunk_lpad(offset, CHUNK_SIZE) as i64,
            host_id: target,
            host_size: StaticContext::get_instance().get_hosts().len() as u64,
            chunk_n: target_chunks.get(&target).unwrap().len() as u64,
            chunk_start: chunk_start,
            chunk_end: chunk_end,
            total_chunk_size: tot_chunk_size,
        };
        if let Ok(p) = NetworkService::post::<ReadData>(
            StaticContext::get_instance()
                .get_hosts()
                .get(target as usize)
                .unwrap(),
            input,
            PostOption::Read,
        ) {
            if p.err {
                return (-1, 0);
            }
            let read_res: ReadResult = serde_json::from_str(p.data.as_str()).unwrap();
            tot_read += read_res.nreads;
            let data = read_res.data;
            for chnk in data {
                let local_offset = if chnk.0 == chunk_start {
                    0
                } else {
                    chnk.0 * CHUNK_SIZE - (offset as u64 % CHUNK_SIZE)
                };
                let data = if chnk.0 == chunk_end {
                    chnk.1 + "\0"
                } else {
                    chnk.1
                };
                unsafe {
                    strncpy(
                        buf.offset(local_offset as isize),
                        data.as_ptr() as *const i8,
                        data.len(),
                    );
                }
            }
        } else {
            return (-1, 0);
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
                option: option2i(PostOption::GetDirents),
                data: serde_json::to_string(&DirentData { path: path.clone() }).unwrap(),
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
        let entries: Vec<(String, bool)> = serde_json::from_str(&result.data).unwrap();
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
    if result.err {
        return false;
    }
    let config: SFSConfig = serde_json::from_str(&result.data.as_str()).unwrap();
    context.set_mountdir(config.mountdir.clone());
    context.set_fsconfig(config);
    return true;
}
