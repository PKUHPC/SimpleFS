#![allow(dead_code)]
mod global;
mod client;
mod server;

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use crate::{server::{storage::data::chunk_storage::{ChunkStorage, ChunkStat}, self}, client::client_openfile::OpenFile};
    #[test]
    fn it_works() {
        /* 
        let s1 = "/proc/config".to_string();
        let s2 = "folder/file".to_string();
        let s3 = path_util::prepend_path(s1, s2);
        let tokens = path_util::split_path(s3.clone());
        for token in tokens{
            print!("{}\n", token);
        }
        //print!("{}\n", path_util::absolute_to_relative("/proc/config/folder".to_string(), s3.clone()));
        print!("{}\n", path_util::dirname(s3.clone()))
        */

        /* 
        print!("{}\n", ClientContext::get_instance().get_mountdir());
        ClientContext::get_instance().set_mountdir("/temp/sfs/data".to_string());
        for token in ClientContext::get_instance().get_mountdir_components().iter(){
            print!("{}\n", token);
        }
        let resolve_res = path::resolve("/temp/./sfs/../sfs/./data/file.txt".to_string(), true);
        print!("{}, {}", resolve_res.0, resolve_res.1);
        */
        /*
        let cs = ChunkStorage::new(&"/home/dev/Desktop/storage".to_string(), 4096).unwrap();
        let s = String::from("hello file system");
        let buf = s.as_bytes();
        //println!("{}", cs.absolute(&ChunkStorage::get_chunks_dir(&"/data/cnk".to_string())));
        let file_path = "/data/cnk".to_string();
        cs.init_chunk_space(&file_path);
        if let Ok(wrote) = cs.write_chunk(&file_path, 1, buf, buf.len() as u64, 0){
            assert_eq!(wrote, buf.len() as u64);
            let mut read_buf = [0 as u8; 12];
            cs.read_chunk(&file_path, 1, &mut read_buf, 12, 0);
            print!("{}\n", String::from_utf8(read_buf.to_vec()).unwrap());
        }
        else{
            print!("????\n");
        }
        */
        //server::main::main();
        let f = OpenFile::new("".to_string(), 0, crate::client::client_openfile::FileType::SFS_REGULAR);
        let s = Mutex::new(f);
        let a = s.lock().unwrap().get_pos();
        s.lock().unwrap().set_pos(a + 20);
        print!("{}\n", s.lock().unwrap().get_pos());
    }
}
