use std::{fs, sync::Arc, path::Components};

use super::{util::path_util::is_absolute, error_msg::error_msg};
use crate::client::client_context::ClientContext;

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

pub fn resolve(path: String, resolve_last_link: bool) -> (bool, String){
    let excluded_path = vec!["proc/".to_string(), "sys/".to_string()];
    /*
    if !is_absolute(path.clone()){
        error_msg("global::path::resolve".to_string(), "path needs to be absolute".to_string());
        return (false, "".to_string());
    }
    */
    for exclude in excluded_path{
        if path[1..path.len()].to_string().starts_with(&exclude.clone()){
            return (false, path);
        }
    }
    //let mut st = Stat::init();
    let mnt_components = ClientContext::get_instance().get_mountdir_components();
    let mut matched_components: usize = 0;
    let mut resolved_components: usize = 0;
    let mut comp_size: usize = 0;
    let mut start: usize = 0; // start index of curr component
    let mut end: usize = 0; // end index of curr component (last processed Path Separator "separator")
    let mut last_slash_pos: usize = 0; // index of last slash in resolved path
    let mut resolved: String = String::from("");
    resolved.reserve(path.len());
    while end + 1 < path.len() {
        end += 1;
        start = end.clone();
        while start < path.len() && path.as_bytes()[start] as char == SEPERATOR {
            start = start + 1;
        }
        if let Some(index) = path[start..path.len()].to_string().find(SEPERATOR){
            end = index + start;
        }
        else{
            end = path.len();
        }
        print!("{}..{}\n", start, end);
        comp_size = end - start;
        if comp_size == 1 && path.as_bytes()[start] as char == '.' {
            continue;
        }
        if comp_size == 2 && path.as_bytes()[start] as char == '.' && path.as_bytes()[start + 1] as char == '.' {
            if !resolved.is_empty() {
                resolved = resolved[0..last_slash_pos].to_string();
                last_slash_pos = resolved.rfind(SEPERATOR).unwrap();
            }
            if resolved_components > 0 {
                if matched_components == resolved_components {
                    matched_components -= 1;
                }
                resolved_components -= 1;
            }
            continue;
        }
        resolved.push(SEPERATOR);
        last_slash_pos = resolved.len() - 1;
        resolved.push_str(&path[start..start + comp_size].to_string());

        print!("{} - {}\n", path[start..(start + comp_size)].to_string(), resolved);
        if matched_components < mnt_components.len(){
            // outside of custom file system
            if matched_components == resolved_components && path[start..(start + comp_size)].to_string().eq(&mnt_components[matched_components]) {
                matched_components += 1;
            }
            // need to be checked on linux file system
            if let Ok(md) = fs::metadata(resolved.clone()){
                if md.is_symlink() {
                    if !resolve_last_link && end == path.len() {
                        continue;
                    }
                    if let Ok(realpath) = fs::canonicalize(resolved.clone()){
                        resolved = realpath.to_str().unwrap().to_string();
                        let matche_res = match_components(resolved.clone(), Arc::clone(&mnt_components));
                        matched_components = matche_res.0;
                        resolved_components = matche_res.1;
                        last_slash_pos = resolved.rfind(SEPERATOR).unwrap();
                        continue;
                    }
                    else{
                        error_msg("global::path::resolve::get_real_path".to_string(), "failed to get realpath for link".to_string());
                        return (false, resolved);
                    }
                }
                else if !md.is_dir() && end != path.len(){
                    error_msg("global::path::resolve::file_process".to_string(), "path not match".to_string());
                    return (false, resolved);
                }
            }
            else{
                error_msg("global::path::resolve::file_existence_check".to_string(), "file not exists".to_string());
                return (false, resolved);
            }
        }
        else{
            matched_components += 1;
        }
        resolved_components += 1;
    }

    if matched_components >= mnt_components.len() {
        resolved = resolved[0..0].to_string() + &resolved[(ClientContext::get_instance().get_mountdir().len() + 1)..resolved.len()].to_string();
        return (true, resolved)
    }
    if resolved.is_empty() {
        resolved.push(SEPERATOR);
    }
    (false, resolved)
}