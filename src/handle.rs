use std::net::TcpStream;
use serde_json::Value;
use sfs_lib::global::network::forward_data::WriteData;

pub fn handle_write(mut stream: TcpStream, write_data: WriteData){
    let path = write_data.path;
    let data = write_data.buffers;
    let offset = write_data.offset;
}