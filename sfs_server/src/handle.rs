#[allow(unused)]
use std::time::Instant;

use crate::server::{
    storage::data::chunk_storage::ChunkStorage,
};
use sfs_global::global::{
    network::{
        config::CHUNK_SIZE,
        forward_data::{PreCreateData, ReadData, ReadResult, TruncData, WriteData},
        post::post_result,
    },
    util::{
        arith_util::{block_index, block_overrun},
        serde_util::serialize,
    },
};
use sfs_rpc::proto::server::PostResult;

pub fn handle_write(input: &WriteData, data: &[u8]) -> PostResult {
    let write_tot = if let Ok(nwrite) = ChunkStorage::write_chunk(
        &input.path.to_string(),
        input.chunk_id,
        data,
        input.write_size,
        input.offset as u64,
    ) {
        nwrite
    } else {
        0
    };
    let post_res = post_result(0, serialize(write_tot), vec![0; 0]);
    return post_res;
}

#[allow(unused_variables)]
#[allow(unused_assignments)]
pub fn handle_read(input: &ReadData) -> PostResult {
    let read_res = read_file(&input);
    let post_res = post_result(
        0,
        serialize(&ReadResult {
            nreads: read_res.1,
            chunk_id: read_res.0,
        }),
        read_res.2,
    );
    return post_res;
}

fn read_file(args: &ReadData<'_>) -> (u64, u64, Vec<u8>) {
    //println!("{:?}", args);
    //println!("reading...");
    let mut buf = vec![0; CHUNK_SIZE as usize];
    if let Ok(nreads) = ChunkStorage::read_chunk(
        &args.path.to_string(),
        args.chunk_id,
        &mut buf,
        args.read_size,
        args.offset as u64,
    ) {
        (args.chunk_id, nreads, buf[0..(nreads as usize)].to_vec())
    } else {
        (args.chunk_id, 0, vec![0; 1])
    }
}

pub fn handle_trunc(input: TruncData<'_>) -> PostResult {
    let path = input.path;
    let size = input.new_size;
    let mut chunk_id_start = block_index(size, CHUNK_SIZE);
    let left_pad = block_overrun(size, CHUNK_SIZE);
    if left_pad != 0 {
        ChunkStorage::truncate_chunk_file(&path.to_string(), chunk_id_start, left_pad);
        chunk_id_start += 1;
    }
    ChunkStorage::trim_chunk_space(&path.to_string(), chunk_id_start);
    let post_res = post_result(0, vec![0; 0], vec![0; 0]);
    return post_res;
}
pub fn handle_precreate(input: &PreCreateData) {
    let path = input.path.to_string();
    ChunkStorage::init_chunk_space(&path);
    for chunk_id in input.chunks.iter() {
        let chunk_path =
            ChunkStorage::absolute(&ChunkStorage::get_chunks_path(&path, *chunk_id));
        std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&chunk_path)
            .unwrap();
    }
}
