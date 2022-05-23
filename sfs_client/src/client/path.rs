use sfs_global::global::{
    error_msg::error_msg, fsconfig::CWD, path::match_components, util::path_util::is_absolute,
};
use std::{
    env::{current_dir, remove_var, set_var},
    fs,
    sync::Arc,
};

use super::context::{DynamicContext, StaticContext};

static SEPERATOR: char = '/';
pub const MAX_LENGTH: i64 = 4096;

pub fn resolve(path: &String, resolve_last_link: bool) -> (bool, String) {
    let excluded_path = vec!["proc/".to_string(), "sys/".to_string()];
    if !is_absolute(&path) {
        //error_msg("global::path::resolve".to_string(), "path needs to be absolute".to_string());
        return (false, path.clone());
    }
    for exclude in excluded_path {
        if path[1..path.len()]
            .to_string()
            .starts_with(&exclude.clone())
        {
            return (false, path.clone());
        }
    }
    let mnt_components = StaticContext::get_instance().get_mountdir_components();
    let mut matched_components: usize = 0;
    let mut resolved_components: usize = 0;
    let mut comp_size: usize;
    let mut start: usize; // start index of curr component
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
        if let Some(index) = path[start..path.len()].to_string().find(SEPERATOR) {
            end = index + start;
        } else {
            end = path.len();
        }
        //print!("{}..{}\n", start, end);
        comp_size = end - start;
        if comp_size == 1 && path.as_bytes()[start] as char == '.' {
            continue;
        }
        if comp_size == 2
            && path.as_bytes()[start] as char == '.'
            && path.as_bytes()[start + 1] as char == '.'
        {
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

        //print!("{} - {}\n", path[start..(start + comp_size)].to_string(), resolved);
        if matched_components < mnt_components.len() {
            // outside of custom file system
            if matched_components == resolved_components
                && path[start..(start + comp_size)]
                    .to_string()
                    .eq(&mnt_components[matched_components])
            {
                matched_components += 1;
            }
            // need to be checked on linux file system
            if let Ok(md) = fs::metadata(resolved.clone()) {
                if std::fs::symlink_metadata(resolved.as_str())
                    .unwrap()
                    .file_type()
                    .is_symlink()
                {
                    if !resolve_last_link && end == path.len() {
                        continue;
                    }
                    if let Ok(realpath) = fs::canonicalize(resolved.clone()) {
                        resolved = realpath.to_str().unwrap().to_string();
                        let matche_res =
                            match_components(resolved.clone(), Arc::clone(&mnt_components));
                        matched_components = matche_res.0;
                        resolved_components = matche_res.1;
                        last_slash_pos = resolved.rfind(SEPERATOR).unwrap();
                        continue;
                    } else {
                        error_msg(
                            "global::path::resolve::get_real_path".to_string(),
                            "failed to get realpath for link".to_string(),
                        );
                        return (false, resolved);
                    }
                } else if !md.is_dir() && end != path.len() {
                    error_msg(
                        "global::path::resolve::file_process".to_string(),
                        "path not match".to_string(),
                    );
                    return (false, resolved);
                }
            } else {
                error_msg(
                    "global::path::resolve::file_existence_check".to_string(),
                    "file not exists".to_string(),
                );
                return (false, resolved);
            }
        } else {
            matched_components += 1;
        }
        resolved_components += 1;
    }

    if matched_components >= mnt_components.len() {
        if resolved.len() > StaticContext::get_instance().get_mountdir().len() {
            resolved.replace_range(
                1..StaticContext::get_instance().get_mountdir().len() + 1,
                "",
            );
        } else {
            resolved.replace_range(1..StaticContext::get_instance().get_mountdir().len(), "");
        }
        return (true, resolved);
    }
    if resolved.is_empty() {
        resolved.push(SEPERATOR);
    }
    (false, resolved)
}
/*
pub fn set_sys_cwd(path: &String) -> i32 {
    unsafe { syscall_no_intercept(SYS_chdir, path.as_ptr() as *const c_char) as i32 }
}
*/
pub fn set_env_cwd(path: &String) {
    set_var(CWD, path);
}
pub fn unset_env_cwd() {
    remove_var(CWD);
}
pub fn get_sys_cwd() -> String {
    let path = current_dir().unwrap().to_str().unwrap().to_string();
    if !is_absolute(&path) {
        error_msg(
            "client::path::get_sys_cwd".to_string(),
            "current directory is unreachable".to_string(),
        );
    }
    return path;
    /*
    let temp = [0; max_length as usize];
    unsafe { syscall_no_intercept(SYS_getcwd, temp.as_ptr() as *mut c_char, max_length) };
    if temp[0] as char != SEPERATOR {
        error_msg(
            "client::path::get_sys_cwd".to_string(),
            "current directory is unreachable".to_string(),
        );
        return "".to_string();
    }
    return String::from_utf8(temp.to_vec()).unwrap();
    */
}
pub fn set_cwd(path: &String, internal: bool) -> i32 {
    if internal {
        //let sys_res = set_sys_cwd(StaticContext::get_instance().get_mountdir());
        //if sys_res != 0 {
        //    return sys_res;
        //}
        set_env_cwd(path);
    } else {
        //let sys_res = set_sys_cwd(path);
        //if sys_res != 0 {
        //    return sys_res;
        //}
        unset_env_cwd();
    }
    DynamicContext::get_instance().set_cwd(path.clone());
    return 0;
}
/*
#[link(name = "syscall_intercept", kind = "static")]
extern "C" {
    pub fn syscall_no_intercept(
        syscall_number: ::std::os::raw::c_long,
        ...
    ) -> ::std::os::raw::c_long;
}
*/
