#![allow(dead_code)]
mod global;
mod client;
mod server;

#[cfg(test)]
mod tests {
    use std::fs;

    use bit_vec::*;

    use crate::global::error_msg::error_msg;
    use crate::server;
    use crate::{global::util::path_util, client::simplefs_context::ClientContext};
    use crate::global::path;

    fn get_param_for_increase_size(op_s: &String) -> Option<(i64, bool)>{
        let s = op_s.split('|');
        let vec = s.collect::<Vec<&str>>();
        if vec.len() != 3{
            error_msg("server::merge::get_param_for_increase_size".to_string(), "invalid string format".to_string());
            return None;
        }
        Some((vec[1].parse::<i64>().unwrap(), vec[2].parse::<bool>().unwrap()))
    }
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
        print!("{}\n", "a/b/b/c/s/e/f/d/s".to_string().replace("/", ":"));
    }
}
