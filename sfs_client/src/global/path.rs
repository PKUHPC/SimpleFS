use std::{fs, sync::Arc};

use super::{error_msg::error_msg};
use crate::client::context::ClientContext;

static SEPERATOR: char = '/';
pub static max_length: i64 = 4096;
pub struct Stat{
    st_dev: u32,
    st_ino: u16,
    st_mode: u16,
    st_nlink: i16,
    st_uid: i16,
    st_gid: i16,
    st_rdev: u32,
    st_size: i64,
    st_atime: i64,
    st_mtime: i64,
    st_ctime: i64
}
impl Stat{
    pub fn init() -> Stat{
        Stat{
            st_dev: 0,
            st_ino: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_size: 0,
            st_atime: 0,
            st_mtime: 0,
            st_ctime: 0,
        }
    }
}

pub fn match_components(path: String, components: Arc<Vec<String>>) -> (usize, usize) {
    let mut matched: usize = 0;
    let mut processed_components: usize = 0;
    let mut comp_size: usize = 0; // size of current component
    let mut start: usize = 0; // start index of curr component
    let mut end: usize = 0; // end index of curr component (last processed Path Separator "separator")

    while end + 1 < path.len(){
        end += 1;
        start = end.clone();

        if let Some(index) = path[start..path.len()].to_string().find(SEPERATOR){
            end = index + start;
        }
        else{
            end = path.len();
        }
        comp_size = end - start;
        if matched == processed_components && path[start..(start + comp_size)].to_string().eq(&components[matched]) {
            matched += 1;
        }
        processed_components += 1;
    }
    (matched, processed_components)
}

