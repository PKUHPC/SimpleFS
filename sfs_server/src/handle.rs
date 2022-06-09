#[allow(unused)]
use std::time::Instant;

use crate::server::storage::data::chunk_storage::ChunkStorage;
#[allow(unused)]
use sfs_global::global::{
    network::{
        config::CHUNK_SIZE,
        forward_data::{PreCreateData, ReadData, ReadResult, TruncData, WriteData},
    },
    util::{
        arith_util::{block_index, block_overrun},
        serde_util::serialize,
    },
};
use sfs_rpc::{post_result, proto::server::PostResult};

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
        let chunk_path = ChunkStorage::absolute(&ChunkStorage::get_chunks_path(&path, *chunk_id));
        std::fs::OpenOptions::new()
            .create(true)
            .open(&chunk_path)
            .unwrap();
    }
}
