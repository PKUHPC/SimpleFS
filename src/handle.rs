use std::{net::TcpStream, thread::{self, JoinHandle}};
use sfs_lib::{global::network::{forward_data::WriteData, config::CHUNK_SIZE}, client::client_distributor::{LocalOnlyDistributor, Distributor}, server::filesystem::storage_context::StorageContext};

use crate::task::WriteChunkTask;

pub fn handle_write(input: WriteData){
    let path = input.path;

    let mut chunk_ids_host: Vec<u64> = vec![0; input.chunk_n as usize];

    let mut chunk_id_curr = 0;

    let mut chunk_size: Vec<u64> = vec![0; input.chunk_n as usize];

    let mut buf_ptr: Vec<u64> = vec![0; input.chunk_n as usize];
    
    let mut task_args: Vec<WriteChunkTask> = vec![WriteChunkTask::new(); input.chunk_n as usize];

    let mut tasks: Vec<JoinHandle<u64>> = Vec::new();
    tasks.reserve(input.chunk_n as usize);

    let host_id = input.host_id;
    let mut chunk_size_left_host = input.total_chunk_size;

    let buf = input.buffers.as_bytes();
    let mut chunk_ptr = 0;

    let mut transfer_size = CHUNK_SIZE;

    let distributor = LocalOnlyDistributor::new();
    for chunk_id_file in input.chunk_start..(input.chunk_end + 1){
        if chunk_id_curr >= input.chunk_n{
            break;
        }
        if distributor.locate_data(&path, chunk_id_file) != host_id{
            continue;
        }
        chunk_ids_host[chunk_id_curr as usize] = chunk_id_file;
        if chunk_id_file == input.chunk_start && input.offset > 0{
            let offset_size = CHUNK_SIZE - input.offset as u64;

            buf_ptr[chunk_id_curr as usize] = chunk_ptr;          
            chunk_size[chunk_id_curr as usize] = offset_size;
            chunk_ptr += offset_size;
            chunk_size_left_host -= offset_size;
        }
        else{
            let local_offset = input.total_chunk_size - chunk_size_left_host;
            let mut origin_offset = (chunk_id_file - input.chunk_start) * CHUNK_SIZE;
            if input.offset > 0{
                origin_offset = (CHUNK_SIZE - input.offset as u64) + ((chunk_id_file - input.chunk_start) - 1) * CHUNK_SIZE;
            }
            if chunk_id_curr == input.chunk_n - 1{
                transfer_size = chunk_size_left_host;
            }

            buf_ptr[chunk_id_curr as usize] = chunk_ptr;      
            chunk_size[chunk_id_curr as usize] = transfer_size;
            chunk_ptr += transfer_size;
            chunk_size_left_host -= transfer_size;
        }

        let write_task = WriteChunkTask{
            path: path.clone(),
            buf: String::from_utf8(buf[(chunk_ptr - transfer_size) as usize..chunk_ptr as usize].to_vec()).unwrap(),
            chunk_id: chunk_ids_host[chunk_id_curr as usize],
            size: chunk_size[chunk_id_curr as usize],
            offset: if chunk_id_file == 0 { input.offset as u64 } else { 0 },
        };
        task_args[chunk_id_curr as usize] = write_task.clone();
        // write to chunk

        tasks.push(thread::spawn(move || write_file(&write_task)));
        chunk_id_curr += 1;
    }
    for t in tasks{
        t.join().unwrap();
    }
}
fn write_file(args: &WriteChunkTask) -> u64{
    if let Ok(nwrite) = StorageContext::get_instance().get_storage().unwrap().write_chunk(&args.path, args.chunk_id, args.buf.as_bytes(), args.size, args.offset){
        nwrite
    }
    else { 0 }
}