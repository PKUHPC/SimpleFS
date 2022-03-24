use std::collections::HashMap;

use sfs_lib_server::{
    global::{
        distributor::{Distributor, SimpleHashDistributor},
        network::{
            config::CHUNK_SIZE,
            forward_data::{ReadData, ReadResult, TruncData, WriteData},
            post::PostResult,
        },
        util::arith_util::{block_index, block_overrun},
    },
    server::{
        filesystem::storage_context::StorageContext, storage::data::chunk_storage::ChunkStorage,
    },
};
use tokio::task::JoinHandle;

use crate::task::{ReadChunkTask, WriteChunkTask};

pub async fn handle_write(input: WriteData) -> String {
    let path = input.path;

    let mut chunk_ids_host: Vec<u64> = vec![0; input.chunk_n as usize];

    let mut chunk_id_curr = 0;

    let mut chunk_size: Vec<u64> = vec![0; input.chunk_n as usize];

    let mut buf_ptr: Vec<u64> = vec![0; input.chunk_n as usize];

    let mut task_args: Vec<WriteChunkTask> = vec![WriteChunkTask::new(); input.chunk_n as usize];

    let mut tasks: Vec<JoinHandle<u64>> = Vec::new();
    tasks.reserve(input.chunk_n as usize);

    let host_id = input.host_id;
    let host_size = input.host_size;
    let mut chunk_size_left_host = input.total_chunk_size;

    let buf = input.buffers.as_bytes();
    let mut chunk_ptr = 0;

    let mut transfer_size = CHUNK_SIZE;

    let mut distributor = SimpleHashDistributor::new(host_id, host_size);
    for chunk_id_file in input.chunk_start..(input.chunk_end + 1) {
        if chunk_id_curr >= input.chunk_n {
            break;
        }
        if distributor.locate_data(&path, chunk_id_file) != host_id {
            continue;
        }
        chunk_ids_host[chunk_id_curr as usize] = chunk_id_file;
        if chunk_id_file == input.chunk_start && input.offset > 0 {
            let offset_size = CHUNK_SIZE - input.offset as u64;

            buf_ptr[chunk_id_curr as usize] = chunk_ptr;
            chunk_size[chunk_id_curr as usize] = offset_size;
            chunk_ptr += offset_size;
            chunk_size_left_host -= offset_size;
        } else {
            let local_offset = input.total_chunk_size - chunk_size_left_host;
            let mut origin_offset = (chunk_id_file - input.chunk_start) * CHUNK_SIZE;
            if input.offset > 0 {
                origin_offset = (CHUNK_SIZE - input.offset as u64)
                    + ((chunk_id_file - input.chunk_start) - 1) * CHUNK_SIZE;
            }
            chunk_ptr = origin_offset;
            if chunk_id_curr == input.chunk_n - 1 {
                transfer_size = chunk_size_left_host;
            }

            buf_ptr[chunk_id_curr as usize] = chunk_ptr;
            chunk_size[chunk_id_curr as usize] = transfer_size;
            chunk_ptr += transfer_size;
            chunk_size_left_host -= transfer_size;
        }

        let write_task = WriteChunkTask {
            path: path.clone(),
            buf: String::from_utf8(
                buf[buf_ptr[chunk_id_curr as usize] as usize..chunk_ptr as usize].to_vec(),
            )
            .unwrap(),
            chunk_id: chunk_ids_host[chunk_id_curr as usize],
            size: chunk_size[chunk_id_curr as usize],
            offset: if chunk_id_file == input.chunk_start {
                input.offset as u64
            } else {
                0
            },
        };
        task_args[chunk_id_curr as usize] = write_task.clone();
        // write to chunk

        tasks.push(tokio::spawn(async move { write_file(&write_task).await }));
        chunk_id_curr += 1;
    }
    let mut write_tot = 0;
    for t in tasks {
        write_tot += t.await.unwrap();
    }
    let post_res = PostResult {
        err: false,
        data: write_tot.to_string(),
    };
    return serde_json::to_string(&post_res).unwrap();
}
async fn write_file(args: &WriteChunkTask) -> u64 {
    //println!("{:?}", args);
    //println!("writing...");
    if let Ok(nwrite) = ChunkStorage::write_chunk(
        &args.path,
        args.chunk_id,
        args.buf.as_bytes(),
        args.size,
        args.offset,
    )
    .await
    {
        nwrite
    } else {
        0
    }
}

pub async fn handle_read(input: ReadData) -> String {
    let path = input.path;

    let mut chunk_ids_host: Vec<u64> = vec![0; input.chunk_n as usize];

    let mut chunk_id_curr = 0;

    let mut chunk_size: Vec<u64> = vec![0; input.chunk_n as usize];

    let mut task_args: Vec<ReadChunkTask> = vec![ReadChunkTask::new(); input.chunk_n as usize];

    let mut tasks: Vec<JoinHandle<(u64, u64, String)>> = Vec::new();
    tasks.reserve(input.chunk_n as usize);

    let host_id = input.host_id;
    let host_size = input.host_size;
    let mut chunk_size_left_host = input.total_chunk_size;

    let mut chunk_ptr = 0;

    let mut transfer_size = CHUNK_SIZE;

    let mut distributor = SimpleHashDistributor::new(host_id, host_size);
    for chunk_id_file in input.chunk_start..(input.chunk_end + 1) {
        if chunk_id_curr >= input.chunk_n {
            break;
        }
        if distributor.locate_data(&path, chunk_id_file) != host_id {
            continue;
        }
        chunk_ids_host[chunk_id_curr as usize] = chunk_id_file;
        if chunk_id_file == input.chunk_start && input.offset > 0 {
            let offset_size = CHUNK_SIZE - input.offset as u64;

            chunk_size[chunk_id_curr as usize] = offset_size;
            chunk_ptr += offset_size;
            chunk_size_left_host -= offset_size;
        } else {
            let local_offset = input.total_chunk_size - chunk_size_left_host;
            let mut origin_offset = (chunk_id_file - input.chunk_start) * CHUNK_SIZE;
            if input.offset > 0 {
                origin_offset = (CHUNK_SIZE - input.offset as u64)
                    + ((chunk_id_file - input.chunk_start) - 1) * CHUNK_SIZE;
            }
            if chunk_id_curr == input.chunk_n - 1 {
                transfer_size = chunk_size_left_host;
            }

            chunk_size[chunk_id_curr as usize] = transfer_size;
            chunk_ptr += transfer_size;
            chunk_size_left_host -= transfer_size;
        }

        let read_task = ReadChunkTask {
            path: path.clone(),
            chunk_id: chunk_ids_host[chunk_id_curr as usize],
            size: chunk_size[chunk_id_curr as usize],
            offset: if chunk_id_file == input.chunk_start {
                input.offset as u64
            } else {
                0
            },
        };
        task_args[chunk_id_curr as usize] = read_task.clone();
        // write to chunk

        tasks.push(tokio::spawn(async move { read_file(&read_task).await }));
        chunk_id_curr += 1;
    }
    let mut read_result: HashMap<u64, String> = HashMap::new();
    let mut read_tot = 0;
    for t in tasks {
        let result = t.await.unwrap();
        read_tot += result.1;
        read_result.insert(result.0, result.2);
    }
    let result_data = ReadResult {
        nreads: read_tot,
        data: read_result,
    };
    //println!("{:?}", result_data);
    let post_res = PostResult {
        err: false,
        data: serde_json::to_string(&result_data).unwrap(),
    };
    return serde_json::to_string(&post_res).unwrap();
}

async fn read_file(args: &ReadChunkTask) -> (u64, u64, String) {
    //println!("{:?}", args);
    //println!("reading...");
    let mut buf = [0 as u8; CHUNK_SIZE as usize];
    if let Ok(nreads) =
        ChunkStorage::read_chunk(&args.path, args.chunk_id, &mut buf, args.size, args.offset).await
    {
        (
            args.chunk_id,
            nreads,
            String::from_utf8(buf[0..(nreads as usize)].to_vec()).unwrap(),
        )
    } else {
        (args.chunk_id, 0, "".to_string())
    }
}

pub async fn handle_trunc(input: TruncData) -> String {
    let path = input.path;
    let size = input.new_size;
    let mut chunk_id_start = block_index(size, CHUNK_SIZE);
    let left_pad = block_overrun(size, CHUNK_SIZE);
    if left_pad != 0 {
        ChunkStorage::truncate_chunk_file(&path, chunk_id_start, left_pad).await;
        chunk_id_start += 1;
    }
    ChunkStorage::trim_chunk_space(&path, chunk_id_start).await;
    let post_res = PostResult {
        err: false,
        data: "".to_string(),
    };
    return serde_json::to_string(&post_res).unwrap();
}
