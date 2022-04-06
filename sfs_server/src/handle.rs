#[allow(unused)]
use std::time::Instant;

use sfs_global::global::{
    network::{
        config::CHUNK_SIZE,
        forward_data::{ReadData, ReadResult, TruncData, WriteData},
    },
    util::{arith_util::{block_index, block_overrun}, serde_util::serialize},
};
use sfs_lib_server::server::storage::data::chunk_storage::ChunkStorage;
use sfs_rpc::sfs_server::PostResult;

pub fn handle_write(input: &WriteData) -> PostResult {
    let write_tot = if let Ok(nwrite) = ChunkStorage::write_chunk(
        &input.path.to_string(),
        input.chunk_id,
        input.buffers.as_bytes(),
        input.write_size,
        input.offset as u64,
    )
    {
        nwrite
    } else {
        0
    };
    let post_res = PostResult {
        err: 0,
        data: write_tot.to_string().as_bytes().to_vec(),
    };
    return post_res;
}

#[allow(unused_variables)]
#[allow(unused_assignments)]
pub fn handle_read(input: &ReadData) -> PostResult {
    let read_res = read_file(&input);
    let post_res = PostResult {
        err: 0,
        data: serialize(&ReadResult {
            nreads: read_res.1,
            chunk_id: read_res.0,
            data: read_res.2.as_str(),
        }),
    };
    return post_res;
}

fn read_file(args: &ReadData<'_>) -> (u64, u64, String) {
    //println!("{:?}", args);
    //println!("reading...");
    let mut buf = vec![0; CHUNK_SIZE as usize];
    if let Ok(nreads) = ChunkStorage::read_chunk(
        &args.path.to_string(),
        args.chunk_id,
        &mut buf,
        args.read_size,
        args.offset as u64,
    )
    
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
    let post_res = PostResult {
        err: 0,
        data: vec![0; 1],
    };
    return post_res;
}
