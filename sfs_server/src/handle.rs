
use sfs_global::global::{
    network::{
        config::CHUNK_SIZE,
        forward_data::{ReadData, ReadResult, TruncData, WriteData},
    },
    util::arith_util::{block_index, block_overrun},
};
use sfs_lib_server::server::storage::data::chunk_storage::ChunkStorage;
use sfs_rpc::sfs_server::PostResult;

pub async fn handle_write(input: WriteData<'_>) -> PostResult {
    let write_tot = write_file(&input).await;
    let post_res = PostResult {
        err: 0,
        data: write_tot.to_string(),
    };
    return post_res;
}
async fn write_file(args: &WriteData<'_>) -> u64 {
    //println!("{:?}", args);
    //println!("writing...");
    if let Ok(nwrite) = ChunkStorage::write_chunk(
        &args.path.to_string(),
        args.chunk_id,
        args.buffers.as_bytes(),
        args.write_size,
        args.offset as u64,
    )
    .await
    {
        nwrite
    } else {
        0
    }
}

#[allow(unused_variables)]
#[allow(unused_assignments)]
pub async fn handle_read(input: ReadData<'_>) -> PostResult {
    let read_res = read_file(&input).await;
    let post_res = PostResult {
        err: 0,
        data: serde_json::to_string(
            &ReadResult{
                nreads: read_res.1,
                chunk_id: read_res.0,
                data: read_res.2.as_str(),
            }
        ).unwrap(),
    };
    return post_res;
}

async fn read_file(args: &ReadData<'_>) -> (u64, u64, String) {
    //println!("{:?}", args);
    //println!("reading...");
    let mut buf = vec![0; CHUNK_SIZE as usize];
    if let Ok(nreads) =
        ChunkStorage::read_chunk(&args.path.to_string(), args.chunk_id, &mut buf, args.read_size, args.offset as u64).await
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

pub async fn handle_trunc(input: TruncData<'_>) -> PostResult {
    let path = input.path;
    let size = input.new_size;
    let mut chunk_id_start = block_index(size, CHUNK_SIZE);
    let left_pad = block_overrun(size, CHUNK_SIZE);
    if left_pad != 0 {
        ChunkStorage::truncate_chunk_file(&path.to_string(), chunk_id_start, left_pad).await;
        chunk_id_start += 1;
    }
    ChunkStorage::trim_chunk_space(&path.to_string(), chunk_id_start).await;
    let post_res = PostResult {
        err: 0,
        data: "".to_string(),
    };
    return post_res;
}
