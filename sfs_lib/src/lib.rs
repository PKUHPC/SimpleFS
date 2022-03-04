#![allow(dead_code)]
pub mod global;
pub mod client;
pub mod server;

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use libc::dirent;

    use crate::{server::{storage::data::chunk_storage::{ChunkStorage, ChunkStat}, self}, client::client_openfile::OpenFile};
    #[test]
    fn it_works() {
        let a = 30;
        let dir = dirent{
            d_ino: 0,
            d_off: 0,
            d_reclen: 0,
            d_type: 0,
            d_name: [0; 256],
        };
        dir.d_ino = a;
        print!("a: {}\n", a);
    }
}
