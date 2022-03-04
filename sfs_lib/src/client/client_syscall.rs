use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use std::{ffi::CStr};
use std::os::raw::c_char;

use libc::{O_PATH, O_APPEND, O_CREAT, O_DIRECTORY, S_IFREG, O_EXCL, O_TRUNC, O_RDONLY, O_WRONLY, S_IFMT, S_IFDIR, S_IFCHR, S_IFBLK, S_IFIFO, S_IFSOCK, stat, statfs, fsid_t, ST_NOATIME, ST_NODIRATIME, ST_NOSUID, ST_NODEV, ST_SYNCHRONOUS, statvfs, SEEK_SET, SEEK_CUR, SEEK_DATA, SEEK_END, SEEK_HOLE, memset, c_void, dirent, dirent64, DT_REG, DT_DIR, strcpy};

use crate::global;
use crate::global::error_msg::error_msg;
use crate::global::metadata::{self, S_ISDIR, S_ISREG};
use crate::global::util::path_util::dirname;

use super::client_config;
use super::client_context::ClientContext;
use super::client_openfile::{OpenFile, FileType};
use super::client_util::{get_metadata, metadata_to_stat};
use super::network::forward_msg::{forward_create, forward_remove, forward_get_chunk_stat, forward_get_metadentry_size, forward_get_decr_size, forward_truncate, forward_update_metadentry_size, forward_write, forward_read, forward_get_dirents};

#[no_mangle]
pub extern "C" fn sfs_open(path: * const c_char, mode: u32, flag: i32) -> i32{
    let s = unsafe { CStr::from_ptr(path).to_string_lossy().into_owned() };
    if flag & O_PATH != 0{
        error_msg("client::sfs_open".to_string(), "'O_PATH' not supported".to_string());
        return -1;
    }
    if flag & O_APPEND != 0{
        error_msg("client::sfs_open".to_string(), "'O_APPEND' not supported".to_string());
        return -1;
    }
    let mut exists = true;
    let md_res = get_metadata(&s, false);
    if let Err(e) = md_res{
        exists = false;
    }
    if !exists{
        if flag & O_CREAT == 0{
            error_msg("client::sfs_open".to_string(), "file not exists and 'O_CREATE' is not set".to_string());
            return -1;
        }
        if flag & O_DIRECTORY != 0{
            error_msg("client::sfs_open".to_string(), "'O_DIRECTORY' with 'O_CREATE' not supported".to_string());
            return -1;
        }
        if sfs_create(path, mode | S_IFREG) != 0{
            error_msg("client::sfs_open".to_string(), "error occurs while creating non-existing file".to_string());
            return -1;
        }
    }
    else{
        let md = md_res.unwrap();
        if flag & O_EXCL != 0{
            error_msg("client::sfs_open".to_string(), "can't open exising file with 'O_EXCL'".to_string());
            return -1;
        }
        if S_ISDIR(md.get_mode()){
            return sfs_opendir(path);
        }
        if flag & O_TRUNC != 0 && (flag & O_RDONLY != 0 || flag & O_WRONLY != 0){
            if sfs_truncate(path, md.get_size(), 0) != 0{
                error_msg("client::sfs_open".to_string(), "fail to truncate 'O_TRUNC' file".to_string());
                return -1;
            }
        }
    }
    return ClientContext::get_instance().get_ofm().lock().unwrap().add(Arc::new(Mutex::new(OpenFile::new(s, flag, FileType::SFS_REGULAR))));
}

static CHECK_PARENT_DIR: bool = false;
fn check_parent_dir(path: &String) -> i32{
    if !CHECK_PARENT_DIR{
        return 0;
    }
    let p_comp = dirname(path);
    let md_res = get_metadata(&p_comp, false);
    if let Err(e) = md_res{
        if e == 1{
            error_msg("client::check_parent_dir".to_string(), "parent component doesn't exist".to_string());
        }
        else {
            error_msg("client::check_parent_dir".to_string(), "fail to fetch parent dir metadata".to_string());
        }
        return -1;
    }
    let md = md_res.unwrap();
    if md.get_mode() & metadata::S_IFDIR == 0{
        error_msg("client::check_parent_dir".to_string(), "parent is not directory".to_string());
        return -1;
    }
    return 0;
}
#[no_mangle]
pub extern "C" fn sfs_create(path: * const c_char, mut mode: u32) -> i32{
    match mode & S_IFMT{
        0 => { mode |= S_IFREG; }
        S_IFREG => {},
        S_IFDIR => {},
        S_IFCHR => { error_msg("client:sfs_create".to_string(), "unsupported node type".to_string()); return -1; },
        S_IFBLK => { error_msg("client:sfs_create".to_string(), "unsupported node type".to_string()); return -1; },
        S_IFIFO => { error_msg("client:sfs_create".to_string(), "unsupported node type".to_string()); return -1; },
        S_IFSOCK => { error_msg("client:sfs_create".to_string(), "unsupported node type".to_string()); return -1; },
        _ => { error_msg("client:sfs_create".to_string(), "unsupported node type".to_string()); return -1; },
    }
    let path = unsafe { CStr::from_ptr(path).to_string_lossy().into_owned() };
    if check_parent_dir(&path) != 0{
        return -1;
    }
    let create_res= forward_create(&path, mode);
    if let Err(e) = create_res{
        return -1;
    }
    return 0;
}

#[no_mangle]
pub extern "C" fn sfs_remove(path: * const c_char) -> i32{
    let path = unsafe { CStr::from_ptr(path).to_string_lossy().into_owned() };
    let md_res = get_metadata(&path, false);
    if let Err(e) = md_res{
        error_msg("client::sfs_remove".to_string(), "fail to fetch metadata".to_string());
        return -1;
    }
    let md = md_res.unwrap();
    let has_data = S_ISREG(md.get_mode()) && md.get_size() != 0;
    let rm_res = forward_remove(&path, !has_data, md.get_size());
    if let Err(e) = rm_res{
        error_msg("client::sfs_remove".to_string(), "fail to remove file".to_string());
        return -1;
    }
    return 0;
}

#[no_mangle]
pub extern "C" fn sfs_access(path: * const c_char, mask: i32, follow_links: bool) -> i32{
    let path = unsafe { CStr::from_ptr(path).to_string_lossy().into_owned() };
    let md_res = get_metadata(&path, false);
    if let Err(e) = md_res{
        return -1;
    }
    return 0;
}

#[no_mangle]
pub extern "C" fn sfs_stat(path: * const c_char, buf: *mut stat, follow_links: bool) -> i32{
    let path = unsafe { CStr::from_ptr(path).to_string_lossy().into_owned() };
    let md_res = get_metadata(&path, false);
    if let Err(e) = md_res{
        return -1;
    }
    let md = md_res.unwrap();
    unsafe{ metadata_to_stat(&path, md, &mut *buf) };
    return 0;
}
#[no_mangle]
pub extern "C" fn sfs_statfs(buf: *mut statfs) -> i32{
    let ret = forward_get_chunk_stat();
    if ret.0 != 0{
        error_msg("sfs_statfs".to_string(), format!("error {} occurs wile fetch chunk stat", ret.0));
        return -1;
    }
    let blk_stat = ret.1;
    unsafe{
        (*buf).f_type = 0;
        (*buf).f_bsize = blk_stat.chunk_size as i64;
        (*buf).f_blocks = blk_stat.chunk_total;
        
        {*buf}.f_bfree = blk_stat.chunk_free;
        {*buf}.f_bavail = blk_stat.chunk_free;
        {*buf}.f_files = 0;
        {*buf}.f_ffree = 0;
        //{*buf}.f_fsid = fsid_t { __val: [0, 0]};
        {*buf}.f_namelen = global::path::max_length;
        {*buf}.f_frsize = 0;
        //{*buf}.flags = ST_NOATIME | ST_NODIRATIME | ST_NOSUID | ST_NODEV | ST_SYNCHRONOUS;
    }
    return 0;
}
#[no_mangle]
pub extern "C" fn sfs_statvfs(buf: *mut statvfs) -> i32{
    let ret = forward_get_chunk_stat();
    if ret.0 != 0{
        error_msg("sfs_statfs".to_string(), format!("error {} occurs wile fetch chunk stat", ret.0));
        return -1;
    }
    let blk_stat = ret.1;
    unsafe{
        (*buf).f_bsize = blk_stat.chunk_size;
        (*buf).f_blocks = blk_stat.chunk_total;
        {*buf}.f_bfree = blk_stat.chunk_free;
        {*buf}.f_bavail = blk_stat.chunk_free;
        {*buf}.f_files = 0;
        {*buf}.f_ffree = 0;
        {*buf}.f_favail = 0;
        {*buf}.f_fsid = 0;
        {*buf}.f_namemax = global::path::max_length as u64;
        {*buf}.f_frsize = 0;
        //{*buf}.flags = ST_NOATIME | ST_NODIRATIME | ST_NOSUID | ST_NODEV | ST_SYNCHRONOUS;
    }
    return 0;
}
#[no_mangle]
pub fn sfs_lseek(fd: i32, offset: i64, whence: i32) -> i64{
    let f_res = ClientContext::get_instance().get_ofm().lock().unwrap().get(fd);
    if let None = f_res{
        return -1;
    }
    internal_lseek(f_res.unwrap(), offset, whence)
}
pub fn internal_lseek(fd: Arc<Mutex<OpenFile>>, offset: i64, whence: i32) -> i64{
    match whence {
        SEEK_SET => {
            if offset < 0{
                error_msg("client::sfs_leek".to_string(), "offset must be positive".to_string());
                return -1;
            }
            fd.lock().unwrap().set_pos(offset);
        },
        SEEK_CUR => {
            let curr_pos = fd.lock().unwrap().get_pos();
            fd.lock().unwrap().set_pos(curr_pos + offset);
        },
        SEEK_END => {
            let ret = forward_get_metadentry_size(&fd.lock().unwrap().get_path());
            if ret.0 != 0 {
                return -1;
            }
            let file_size = ret.1;
            if offset < 0 && file_size < - offset {
                return -1;
            }
            fd.lock().unwrap().set_pos(file_size + offset);
        },
        SEEK_DATA => { return -1; },
        SEEK_HOLE => { return -1; },
        _ => { return -1; },
    }
    return fd.lock().unwrap().get_pos();
}

#[no_mangle]
pub extern "C" fn sfs_truncate(path: * const c_char, old_size: i64, new_size: i64) -> i32{
    if new_size < 0 || new_size > old_size {
        return -1;
    }
    if new_size == old_size{
        return 0;
    }
    let path = unsafe { CStr::from_ptr(path).to_string_lossy().into_owned() };
    if forward_get_decr_size(&path, new_size) != 0{
        return -1;
    }
    if forward_truncate(&path, old_size, new_size) != 0{
        return -1;
    }
    return 0;
}

#[no_mangle]
pub extern "C" fn sfs_dup(oldfd: i32) -> i32{
    return ClientContext::get_instance().get_ofm().lock().unwrap().dup(oldfd);
}

#[no_mangle]
pub extern "C" fn sfs_dup2(oldfd: i32, newfd: i32) -> i32{
    return ClientContext::get_instance().get_ofm().lock().unwrap().dup2(oldfd, newfd);
}

fn internal_pwrite(f: Arc<Mutex<OpenFile>>, buf: * const char, count: i64, offset: i64) -> i64{
    match f.lock().unwrap().get_type(){
        FileType::SFS_REGULAR => { error_msg("client::sfs_pwrite".to_string(), "can not write directory".to_string()); return -1 },
        FileType::SFS_DIRECTORY => {},
    }
    let path = f.lock().unwrap().get_path();
    let append_flag = f.lock().unwrap().get_flag(super::client_openfile::OpenFileFlags::Append);
    let ret_update_size = forward_update_metadentry_size(&path, count, offset, append_flag);
    if ret_update_size.0 != 0{
        error_msg("client::sfs_pwrite".to_string(), format!("update metadentry size with error {}", ret_update_size.0));
        return -1;
    }
    let updated_size = ret_update_size.1;
    let write_res = forward_write(&path, buf, append_flag, offset, count, updated_size);
    if write_res.0 != 0{
        error_msg("client::sfs_pwrite".to_string(), format!("write with error {}", write_res.0));
        return -1;
    }
    return write_res.1;
}

#[no_mangle]
fn sfs_pwrite(fd: i32, buf: * const char, count: i64, offset: i64) -> i64{
    let f = ClientContext::get_instance().get_ofm().lock().unwrap().get(fd);
    if let None = f{
        error_msg("client::sfs_pwrite".to_string(), "file not exist".to_string());
        return -1;
    }
    let f = f.unwrap();
    return internal_pwrite(f, buf, count, offset);
}

#[no_mangle]
pub extern "C" fn sfs_write(fd: i32, buf: * const char, count: i64) -> i64{
    let f = ClientContext::get_instance().get_ofm().lock().unwrap().get(fd);
    if let None = f{
        error_msg("client::sfs_write".to_string(), "file not exist".to_string());
        return -1;
    }
    let f = f.unwrap();
    let pos = f.lock().unwrap().get_pos();
    if f.lock().unwrap().get_flag(super::client_openfile::OpenFileFlags::Append) {
        internal_lseek(Arc::clone(&f), 0, SEEK_END);
    }
    let write_res = internal_pwrite(Arc::clone(&f), buf, count, pos);
    if write_res > 0{
        f.lock().unwrap().set_pos(pos + count);
    }
    return write_res;
}
fn internal_pread(f: Arc<Mutex<OpenFile>>, buf: * mut char, count: i64, offset: i64) -> i64{
    match f.lock().unwrap().get_type(){
        FileType::SFS_REGULAR => { error_msg("client::sfs_pread".to_string(), "can not read directory".to_string()); return -1 },
        FileType::SFS_DIRECTORY => {},
    }
    if client_config::ZERO_BUF_BEFORE_READ{
        unsafe { memset(buf as (* mut c_void), 0, count as usize); }
    }
    let path = f.lock().unwrap().get_path();
    let read_res = forward_read(&path, buf, offset, count);
    if read_res.0 != 0{
        error_msg("client::sfs_pread".to_string(), format!("read with error {}", read_res.0));
        return -1;
    }
    return read_res.1;
}

#[no_mangle]
pub extern "C" fn sfs_pread(fd: i32, buf: * mut char, count: i64, offset: i64) -> i64{
    let f = ClientContext::get_instance().get_ofm().lock().unwrap().get(fd);
    if let None = f{
        error_msg("client::sfs_pread".to_string(), "file not exist".to_string());
        return -1;
    }
    let f = f.unwrap();
    return internal_pread(Arc::clone(&f), buf, count, offset);
}

#[no_mangle]
pub extern "C" fn sfs_read(fd: i32, buf: * mut char, count: i64) -> i64{
    let f = ClientContext::get_instance().get_ofm().lock().unwrap().get(fd);
    if let None = f{
        error_msg("client::sfs_read".to_string(), "file not exist".to_string());
        return -1;
    }
    let f = f.unwrap();
    let pos = f.lock().unwrap().get_pos();
    let read_res = internal_pread(Arc::clone(&f), buf, count, pos);
    if read_res > 0{
        f.lock().unwrap().set_pos(pos + count);
    }
    return read_res;
}


#[no_mangle]
pub extern "C" fn sfs_rmdir(path: * const c_char) -> i32{
    
    let path = unsafe { CStr::from_ptr(path).to_string_lossy().into_owned() };
    let md_res = get_metadata(&path, false);
    if let Err(e) = md_res{
        error_msg("client::sfs_rmdir".to_string(), "file not exist".to_string());
        return -1;
    }
    let md = md_res.unwrap();
    if !S_ISDIR(md.get_mode()){
        error_msg("client::sfs_rmdir".to_string(), "path is not directory".to_string());
        return -1;
    }
    let dirent_res = forward_get_dirents(&path);
    if dirent_res.0 != 0{
        error_msg("client::sfs_rmdir".to_string(), format!("forward get dirents with error {}", dirent_res.0));
        return -1;
    }
    let opendir = dirent_res.1;
    if opendir.lock().unwrap().get_size() != 0{
        error_msg("client::sfs_rmdir".to_string(), "directory not empty".to_string());
        return -1;
    }
    let rm_res = forward_remove(&path, true, 0);
    if let Err(e) = rm_res{
        error_msg("client::sfs_rmdir".to_string(), format!("forward remove directory with error {}", dirent_res.0));
        return -1;
    }
    return 0;
}

#[no_mangle]
pub extern "C" fn sfs_opendir(path: * const c_char) -> i32{
    let path = unsafe { CStr::from_ptr(path).to_string_lossy().into_owned() };
    let md_res = get_metadata(&path, false);
    if let Err(e) = md_res{
        error_msg("client::sfs_opendir".to_string(), "file not exist".to_string());
        return -1;
    }
    let md = md_res.unwrap();
    if !S_ISDIR(md.get_mode()){
        error_msg("client::sfs_opendir".to_string(), "path is not directory".to_string());
        return -1;
    }
    let dirent_res = forward_get_dirents(&path);
    if dirent_res.0 != 0{
        error_msg("client::sfs_opendir".to_string(), format!("forward get dirents with error {}", dirent_res.0));
        return -1;
    }
    return ClientContext::get_instance().get_ofm().lock().unwrap().add(dirent_res.1);
}

fn align(size: usize, step: usize) -> usize{
    (size + step) & (!step + 1)
}
#[no_mangle]
pub extern "C" fn sfs_getdents(fd: i32, dirp: * mut dirent, count: i64) -> i32{
    let opendir = ClientContext::get_instance().get_ofm().lock().unwrap().get_dir(fd);
    if let None = opendir{
        error_msg("client::sfs_getdirents".to_string(), "directory not opned".to_string());
        return -1;
    }
    let opendir = opendir.unwrap();
    let mut pos = opendir.lock().unwrap().get_pos();
    if pos >= opendir.lock().unwrap().get_size() as i64{
        return 0;
    }
    let mut written = 0;
    let size = opendir.lock().unwrap().get_size() as i64;
    while pos < size{
        let de = opendir.lock().unwrap().getdent(pos);
        let total_size = align(19 + de.get_name().len() + 3, 8);
        if total_size as i64 > count - written{
            break;
        }
        let current_dirp = unsafe {(dirp as *mut c_char).offset(written as isize) as *mut dirent};
        let mut s = DefaultHasher::new();
        let p = opendir.lock().unwrap().get_path() + "/" + &de.get_name();
        p.hash(&mut s);
        unsafe{ 
            (*current_dirp).d_ino = s.finish();
            (*current_dirp).d_reclen = total_size as u16;
            let mut c = DT_REG;
            match de.get_type() {
                FileType::SFS_REGULAR => { c = DT_REG },
                FileType::SFS_DIRECTORY => { c = DT_DIR },
            }
            (*current_dirp).d_type = c;
            strcpy((*current_dirp).d_name.as_ptr() as *mut i8, de.get_name().as_ptr() as *mut i8);
            pos += 1;
            (*current_dirp).d_off = pos;
            written += total_size as i64;
        }
    }
    if written == 0{
        return -1;
    }
    opendir.lock().unwrap().set_pos(pos);
    return 0;
}

#[no_mangle]
pub extern "C" fn sfs_getdents64(fd: i32, dirp: * mut dirent64, count: i64) -> i32{
    let opendir = ClientContext::get_instance().get_ofm().lock().unwrap().get_dir(fd);
    if let None = opendir{
        error_msg("client::sfs_getdirents".to_string(), "directory not opned".to_string());
        return -1;
    }
    let opendir = opendir.unwrap();
    let mut pos = opendir.lock().unwrap().get_pos();
    if pos >= opendir.lock().unwrap().get_size() as i64{
        return 0;
    }
    let mut written = 0;
    let size = opendir.lock().unwrap().get_size() as i64;
    while pos < size{
        let de = opendir.lock().unwrap().getdent(pos);
        let total_size = align(19 + de.get_name().len() + 1, 8);
        if total_size as i64 > count - written{
            break;
        }
        let current_dirp = unsafe {(dirp as *mut c_char).offset(written as isize) as *mut dirent};
        let mut s = DefaultHasher::new();
        let p = opendir.lock().unwrap().get_path() + "/" + &de.get_name();
        p.hash(&mut s);
        unsafe{ 
            (*current_dirp).d_ino = s.finish();
            (*current_dirp).d_reclen = total_size as u16;
            let mut c = DT_REG;
            match de.get_type() {
                FileType::SFS_REGULAR => { c = DT_REG },
                FileType::SFS_DIRECTORY => { c = DT_DIR },
            }
            (*current_dirp).d_type = c;
            strcpy((*current_dirp).d_name.as_ptr() as *mut i8, de.get_name().as_ptr() as *mut i8);
            pos += 1;
            (*current_dirp).d_off = pos;
            written += total_size as i64;
        }
    }
    if written == 0{
        return -1;
    }
    opendir.lock().unwrap().set_pos(pos);
    return 0;
}

//#[no_mangle]
//pub extern "C" fn sfs_getdents(fd: i32, dirp: *linux_dirent, count: u32) -> i32{
//    return -1;
//}
