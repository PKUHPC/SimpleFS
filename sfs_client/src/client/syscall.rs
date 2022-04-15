use std::collections::hash_map::DefaultHasher;
use std::ffi::CStr;
use std::hash::{Hash, Hasher};
use std::os::raw::c_char;
use std::slice;
use std::sync::{Arc, Mutex};

use errno::{set_errno, Errno};
#[allow(unused_imports)]
use libc::{
    blkcnt_t, blksize_t, c_int, c_void, dev_t, dirent, dirent64, gid_t, ino_t, memcpy, memset,
    mode_t, nlink_t, off_t, statfs, statvfs, strcpy, time_t, uid_t, DT_DIR, DT_REG, O_APPEND,
    O_CREAT, O_DIRECTORY, O_EXCL, O_PATH, O_RDONLY, O_TRUNC, O_WRONLY, SEEK_CUR, SEEK_DATA,
    SEEK_END, SEEK_HOLE, SEEK_SET, S_IFBLK, S_IFCHR, S_IFDIR, S_IFIFO, S_IFMT, S_IFREG, S_IFSOCK,
};
use libc::{statx, EBADF, EBUSY, EINVAL, EISDIR, ENOENT, ENOTDIR, ENOTEMPTY, ENOTSUP};

use sfs_global::global;
use sfs_global::global::error_msg::error_msg;
use sfs_global::global::fsconfig::{ENABLE_STUFFING, ZERO_BUF_BEFORE_READ};
use sfs_global::global::metadata::{S_ISDIR, S_ISREG};
use sfs_global::global::network::config::CHUNK_SIZE;
use sfs_global::global::util::path_util::dirname;

use super::config::CHECK_PARENT_DIR;
#[allow(unused_imports)]
use super::context::{interception_enabled, DynamicContext};
use super::network::forward_msg::{
    forward_create, forward_decr_size, forward_get_chunk_stat, forward_get_dirents,
    forward_get_metadentry_size, forward_read, forward_remove, forward_truncate,
    forward_update_metadentry_size, forward_write,
};
use super::openfile::{FileType, OpenFile};
use super::util::{get_metadata, metadata_to_stat};

#[no_mangle]
pub extern "C" fn sfs_open(path: *const c_char, mode: u32, flag: i32) -> i32 {
    let s = unsafe { CStr::from_ptr(path).to_string_lossy().into_owned() };
    if flag & O_PATH != 0 {
        error_msg(
            "client::sfs_open".to_string(),
            "'O_PATH' not supported".to_string(),
        );
        set_errno(Errno(ENOTSUP));
        return -1;
    }
    if flag & O_APPEND != 0 {
        error_msg(
            "client::sfs_open".to_string(),
            "'O_APPEND' not supported".to_string(),
        );
        set_errno(Errno(ENOTSUP));
        return -1;
    }
    let md_res = get_metadata(&s, false);
    if let Err(e) = md_res {
        if e == ENOENT {
            if flag & O_CREAT == 0 {
                error_msg(
                    "client::sfs_open".to_string(),
                    "file not exists and 'O_CREATE' is not set".to_string(),
                );
                set_errno(Errno(ENOENT));
                return -1;
            }
            if flag & O_DIRECTORY != 0 {
                error_msg(
                    "client::sfs_open".to_string(),
                    "'O_DIRECTORY' with 'O_CREATE' not supported".to_string(),
                );
                set_errno(Errno(ENOTSUP));
                return -1;
            }
            if sfs_create(path, mode | S_IFREG) != 0 {
                error_msg(
                    "client::sfs_open".to_string(),
                    "error occurs while creating non-existing file".to_string(),
                );
                return -1;
            }
        } else {
            error_msg(
                "client::sfs_open".to_string(),
                "error occurs while fetching metadata".to_string(),
            );
            return -1;
        }
    } else {
        let md = md_res.unwrap();
        if flag & O_EXCL != 0 {
            error_msg(
                "client::sfs_open".to_string(),
                "can't open exising file with 'O_EXCL'".to_string(),
            );
            return -1;
        }
        if S_ISDIR(md.get_mode()) {
            return sfs_opendir(path);
        }
        if flag & O_TRUNC != 0 && (flag & O_RDONLY != 0 || flag & O_WRONLY != 0) {
            if internal_truncate(path, md.get_size(), 0) != 0 {
                error_msg(
                    "client::sfs_open".to_string(),
                    "fail to truncate 'O_TRUNC' file".to_string(),
                );
                return -1;
            }
        }
    }
    return DynamicContext::get_instance()
        .get_ofm()
        .lock()
        .unwrap()
        .add(Arc::new(Mutex::new(OpenFile::new(
            &s,
            flag,
            FileType::SFS_REGULAR,
        ))));
}
fn check_parent_dir(path: &String) -> i32 {
    if !CHECK_PARENT_DIR {
        return 0;
    }
    let p_comp = dirname(path);
    let md_res = get_metadata(&p_comp, false);
    if let Err(e) = md_res {
        match e {
            ENOENT => {
                error_msg(
                    "client::check_parent_dir".to_string(),
                    format!("parent component '{}' doesn't exist", p_comp),
                );
            }
            _ => {
                error_msg(
                    "client::check_parent_dir".to_string(),
                    "fail to fetch parent dir metadata".to_string(),
                );
            }
        }
        return -1;
    }
    let md = md_res.unwrap();
    if md.get_mode() & S_IFDIR == 0 {
        error_msg(
            "client::check_parent_dir".to_string(),
            "parent is not directory".to_string(),
        );
        return -1;
    }
    return 0;
}
#[no_mangle]
pub extern "C" fn sfs_create(path: *const c_char, mut mode: u32) -> i32 {
    match mode & S_IFMT {
        0 => {
            mode |= S_IFREG;
        }
        S_IFREG => {}
        S_IFDIR => {}
        S_IFCHR => {
            error_msg(
                "client:sfs_create".to_string(),
                "unsupported node type".to_string(),
            );
            set_errno(Errno(ENOTSUP));
            return -1;
        }
        S_IFBLK => {
            error_msg(
                "client:sfs_create".to_string(),
                "unsupported node type".to_string(),
            );
            set_errno(Errno(ENOTSUP));
            return -1;
        }
        S_IFIFO => {
            error_msg(
                "client:sfs_create".to_string(),
                "unsupported node type".to_string(),
            );
            set_errno(Errno(ENOTSUP));
            return -1;
        }
        S_IFSOCK => {
            error_msg(
                "client:sfs_create".to_string(),
                "unsupported node type".to_string(),
            );
            set_errno(Errno(ENOTSUP));
            return -1;
        }
        _ => {
            error_msg(
                "client:sfs_create".to_string(),
                "unknown node type".to_string(),
            );
            set_errno(Errno(EINVAL));
            return -1;
        }
    }
    let path = unsafe { CStr::from_ptr(path).to_string_lossy().into_owned() };
    if check_parent_dir(&path) != 0 {
        error_msg(
            "client:sfs_create".to_string(),
            "check parent failed".to_string(),
        );
        return -1;
    }
    let create_res = forward_create(&path, mode);
    if let Err(_e) = create_res {
        error_msg(
            "client:sfs_create".to_string(),
            "error occurs while creating file".to_string(),
        );
        set_errno(Errno(EBUSY));
        return -1;
    } else {
        let err = create_res.unwrap();
        if err != 0 {
            set_errno(Errno(err));
            return -1;
        }
        return 0;
    }
}
#[no_mangle]
pub extern "C" fn sfs_remove(path: *const c_char) -> i32 {
    let path = unsafe { CStr::from_ptr(path).to_string_lossy().into_owned() };
    let md_res = get_metadata(&path, false);
    if let Err(e) = md_res {
        error_msg(
            "client::sfs_remove".to_string(),
            "fail to fetch metadata".to_string(),
        );
        set_errno(Errno(e));
        return -1;
    }
    let md = md_res.unwrap();
    let has_data = S_ISREG(md.get_mode()) && md.get_size() != 0;
    let rm_res = forward_remove(path.clone(), !has_data, md.get_size());
    if let Err(_e) = rm_res {
        error_msg(
            "client::sfs_remove".to_string(),
            "fail to remove file".to_string(),
        );
        return -1;
    }
    let err = rm_res.unwrap();
    if err != 0 {
        set_errno(Errno(err));
        return -1;
    }
    return 0;
}
#[no_mangle]
pub extern "C" fn sfs_access(path: *const c_char, _mask: i32, _follow_links: bool) -> i32 {
    let path = unsafe { CStr::from_ptr(path).to_string_lossy().into_owned() };
    let md_res = get_metadata(&path, false);
    if let Err(_e) = md_res {
        return -1;
    }
    return 0;
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct timespec {
    pub tv_sec: i64,
    pub tv_nsec: i64,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct stat {
    pub st_dev: dev_t,
    pub st_ino: ino_t,
    pub st_nlink: nlink_t,
    pub st_mode: mode_t,
    pub st_uid: uid_t,
    pub st_gid: gid_t,
    pub __pad0: c_int,
    pub st_rdev: dev_t,
    pub st_size: off_t,
    pub st_blksize: blksize_t,
    pub st_blocks: blkcnt_t,
    pub st_atim: timespec,
    pub st_mtim: timespec,
    pub st_ctim: timespec,
    pub __glibc_reserved: [i64; 3],
}
#[no_mangle]
pub extern "C" fn sfs_stat(path: *const c_char, buf: *mut stat, _follow_links: bool) -> i32 {
    let path = unsafe { CStr::from_ptr(path).to_string_lossy().into_owned() };
    let md_res = get_metadata(&path, false);
    if let Err(_e) = md_res {
        return -1;
    }
    let md = md_res.unwrap();
    metadata_to_stat(&path, md, buf);
    return 0;
}
#[no_mangle]
pub extern "C" fn sfs_statx(
    _dirfs: i32,
    path: *const c_char,
    _flags: i32,
    _mask: u32,
    buf: *mut statx,
    follow_links: bool,
) -> i32 {
    let path = unsafe { CStr::from_ptr(path).to_string_lossy().into_owned() };
    if interception_enabled() {
        println!("{}", path);
    }
    let md_res = get_metadata(&path, follow_links);
    if let Err(_e) = md_res {
        return -1;
    }
    let md = md_res.unwrap();
    let mut stat: stat = stat {
        st_dev: 0,
        st_ino: 0,
        st_nlink: 0,
        st_mode: 0,
        st_uid: 0,
        st_gid: 0,
        __pad0: 0,
        st_rdev: 0,
        st_size: 0,
        st_blksize: 0,
        st_blocks: 0,
        st_atim: timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        st_mtim: timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        st_ctim: timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        __glibc_reserved: [0; 3],
    };

    metadata_to_stat(&path, md, &mut stat);
    unsafe {
        (*buf).stx_mask = 0;
        (*buf).stx_blksize = stat.st_blksize as u32;
        (*buf).stx_attributes = 0;
        (*buf).stx_nlink = stat.st_nlink as u32;
        (*buf).stx_uid = stat.st_uid;
        (*buf).stx_gid = stat.st_gid;
        (*buf).stx_mode = stat.st_mode as u16;
        (*buf).stx_ino = stat.st_ino;
        (*buf).stx_size = stat.st_size as u64;
        (*buf).stx_blocks = stat.st_blocks as u64;
        (*buf).stx_attributes_mask = 0;

        (*buf).stx_atime.tv_sec = stat.st_atim.tv_sec;
        (*buf).stx_atime.tv_nsec = stat.st_atim.tv_nsec as u32;

        (*buf).stx_mtime.tv_sec = stat.st_mtim.tv_sec;
        (*buf).stx_mtime.tv_nsec = stat.st_mtim.tv_nsec as u32;

        (*buf).stx_ctime.tv_sec = stat.st_ctim.tv_sec;
        (*buf).stx_ctime.tv_nsec = stat.st_ctim.tv_nsec as u32;

        (*buf).stx_btime = (*buf).stx_atime;
    }
    return 0;
}
#[no_mangle]
pub extern "C" fn sfs_statfs(buf: *mut statfs) -> i32 {
    let ret = forward_get_chunk_stat();
    if ret.0 != 0 {
        error_msg(
            "sfs_statfs".to_string(),
            format!("error {} occurs wile fetch chunk stat", ret.0),
        );
        set_errno(Errno(ret.0));
        return -1;
    }
    let blk_stat = ret.1;
    unsafe {
        (*buf).f_type = 0;
        (*buf).f_bsize = blk_stat.chunk_size as i64;
        (*buf).f_blocks = blk_stat.chunk_total;

        { *buf }.f_bfree = blk_stat.chunk_free;
        { *buf }.f_bavail = blk_stat.chunk_free;
        { *buf }.f_files = 0;
        { *buf }.f_ffree = 0;
        //{*buf}.f_fsid = fsid_t { __val: [0, 0]};
        { *buf }.f_namelen = global::path::MAX_LENGTH;
        { *buf }.f_frsize = 0;
        //{*buf}.flags = ST_NOATIME | ST_NODIRATIME | ST_NOSUID | ST_NODEV | ST_SYNCHRONOUS;
    }
    return 0;
}
#[no_mangle]
pub extern "C" fn sfs_statvfs(buf: *mut statvfs) -> i32 {
    let ret = forward_get_chunk_stat();
    if ret.0 != 0 {
        error_msg(
            "sfs_statfs".to_string(),
            format!("error {} occurs wile fetch chunk stat", ret.0),
        );
        set_errno(Errno(ret.0));
        return -1;
    }
    let blk_stat = ret.1;
    unsafe {
        (*buf).f_bsize = blk_stat.chunk_size;
        (*buf).f_blocks = blk_stat.chunk_total;
        { *buf }.f_bfree = blk_stat.chunk_free;
        { *buf }.f_bavail = blk_stat.chunk_free;
        { *buf }.f_files = 0;
        { *buf }.f_ffree = 0;
        { *buf }.f_favail = 0;
        { *buf }.f_fsid = 0;
        { *buf }.f_namemax = global::path::MAX_LENGTH as u64;
        { *buf }.f_frsize = 0;
        //{*buf}.flags = ST_NOATIME | ST_NODIRATIME | ST_NOSUID | ST_NODEV | ST_SYNCHRONOUS;
    }
    return 0;
}
#[no_mangle]
pub extern "C" fn sfs_lseek(fd: i32, offset: i64, whence: i32) -> i64 {
    let f_res = DynamicContext::get_instance()
        .get_ofm()
        .lock()
        .unwrap()
        .get(fd);
    if let None = f_res {
        return -1;
    }
    internal_lseek(f_res.unwrap(), offset, whence)
}
pub fn internal_lseek(fd: Arc<Mutex<OpenFile>>, offset: i64, whence: i32) -> i64 {
    match whence {
        SEEK_SET => {
            if offset < 0 {
                error_msg(
                    "client::sfs_leek".to_string(),
                    "offset must be positive".to_string(),
                );
                set_errno(Errno(EINVAL));
                return -1;
            }
            fd.lock().unwrap().set_pos(offset);
        }
        SEEK_CUR => {
            let curr_pos = fd.lock().unwrap().get_pos();
            fd.lock().unwrap().set_pos(curr_pos + offset);
        }
        SEEK_END => {
            let ret = forward_get_metadentry_size(fd.lock().unwrap().get_path());
            if ret.0 != 0 {
                set_errno(Errno(ret.0));
                return -1;
            }
            let file_size = ret.1;
            if offset < 0 && file_size < -offset {
                set_errno(Errno(EINVAL));
                return -1;
            }
            fd.lock().unwrap().set_pos(file_size + offset);
        }
        SEEK_DATA => {
            set_errno(Errno(EINVAL));
            return -1;
        }
        SEEK_HOLE => {
            set_errno(Errno(EINVAL));
            return -1;
        }
        _ => {
            set_errno(Errno(EINVAL));
            return -1;
        }
    }
    return fd.lock().unwrap().get_pos();
}
#[no_mangle]
pub extern "C" fn internal_truncate(path: *const c_char, old_size: i64, new_size: i64) -> i32 {
    if new_size < 0 || new_size > old_size {
        return -1;
    }
    if new_size == old_size {
        return 0;
    }
    let path = unsafe { CStr::from_ptr(path).to_string_lossy().into_owned() };
    let err = forward_decr_size(&path, new_size);
    if err != 0 {
        set_errno(Errno(err));
        return -1;
    }
    let err = forward_truncate(&path, old_size, new_size);
    if err != 0 {
        set_errno(Errno(err));
        return -1;
    }
    return 0;
}
#[no_mangle]
pub extern "C" fn sfs_truncate(path: *const c_char, length: i64) -> i32 {
    let spath = unsafe { CStr::from_ptr(path).to_string_lossy().into_owned() };
    let md_res = get_metadata(&spath, false);
    if let Err(_e) = md_res {
        return -1;
    }
    let md = md_res.unwrap();
    return internal_truncate(path, md.get_size(), length);
}
#[no_mangle]
pub extern "C" fn sfs_dup(oldfd: i32) -> i32 {
    return DynamicContext::get_instance()
        .get_ofm()
        .lock()
        .unwrap()
        .dup(oldfd);
}
#[no_mangle]
pub extern "C" fn sfs_dup2(oldfd: i32, newfd: i32) -> i32 {
    return DynamicContext::get_instance()
        .get_ofm()
        .lock()
        .unwrap()
        .dup2(oldfd, newfd);
}
fn internal_pwrite(f: Arc<Mutex<OpenFile>>, buf: *const c_char, count: i64, offset: i64) -> i64 {
    match f.lock().unwrap().get_type() {
        FileType::SFS_DIRECTORY => {
            error_msg(
                "client::sfs_pwrite".to_string(),
                "can not write directory".to_string(),
            );
            set_errno(Errno(EISDIR));
            return -1;
        }
        FileType::SFS_REGULAR => {}
    }
    let append_flag = f
        .lock()
        .unwrap()
        .get_flag(super::openfile::OpenFileFlags::Append);
    let path = f.lock().unwrap().get_path().clone();
    let ret_update_size = if ENABLE_STUFFING && offset + count < CHUNK_SIZE as i64 {
        forward_update_metadentry_size(
            &path,
            count as u64,
            offset,
            append_flag,
            unsafe { slice::from_raw_parts(buf as *const u8, count as usize) }.to_vec(),
        )
    } else {
        forward_update_metadentry_size(&path, count as u64, offset, append_flag, vec![0; 0])
    };
    if ret_update_size.0 > 0 {
        error_msg(
            "client::sfs_pwrite".to_string(),
            format!("update metadentry size with error {}", ret_update_size.0),
        );
        set_errno(Errno(ret_update_size.0));
        return -1;
    }
    // stuffed file
    if ret_update_size.0 == -1 {
        return ret_update_size.1;
    }
    let updated_size = ret_update_size.1;
    let write_res = forward_write(&path, buf, append_flag, offset, count, updated_size);

    if write_res.0 != 0 {
        error_msg(
            "client::sfs_pwrite".to_string(),
            format!("write with error {}", write_res.0),
        );
        set_errno(Errno(write_res.0));
        return -1;
    }
    return write_res.1;
}
#[no_mangle]
pub extern "C" fn sfs_pwrite(fd: i32, buf: *const c_char, count: i64, offset: i64) -> i64 {
    let f = DynamicContext::get_instance()
        .get_ofm()
        .lock()
        .unwrap()
        .get(fd);
    if let None = f {
        error_msg(
            "client::sfs_pwrite".to_string(),
            "file not exist".to_string(),
        );
        set_errno(Errno(EBADF));
        return -1;
    }
    let f = f.unwrap();
    return internal_pwrite(f, buf, count, offset);
}
#[no_mangle]
pub extern "C" fn sfs_write(fd: i32, buf: *const c_char, count: i64) -> i64 {
    let f = DynamicContext::get_instance()
        .get_ofm()
        .lock()
        .unwrap()
        .get(fd);
    if let None = f {
        error_msg(
            "client::sfs_write".to_string(),
            "file not exist".to_string(),
        );
        set_errno(Errno(EBADF));
        return -1;
    }
    let f = f.unwrap();
    let pos = f.lock().unwrap().get_pos();
    if f.lock()
        .unwrap()
        .get_flag(super::openfile::OpenFileFlags::Append)
    {
        internal_lseek(Arc::clone(&f), 0, SEEK_END);
    }
    let write_res = internal_pwrite(Arc::clone(&f), buf, count, pos);
    if write_res > 0 {
        f.lock().unwrap().set_pos(pos + count);
    }
    return write_res;
}
fn internal_pread(f: Arc<Mutex<OpenFile>>, buf: *mut c_char, count: i64, offset: i64) -> i64 {
    match f.lock().unwrap().get_type() {
        FileType::SFS_DIRECTORY => {
            error_msg(
                "client::sfs_pread".to_string(),
                "can not read directory".to_string(),
            );
            set_errno(Errno(EISDIR));
            return -1;
        }
        FileType::SFS_REGULAR => {}
    }
    if ZERO_BUF_BEFORE_READ {
        unsafe {
            memset(buf as *mut c_void, 0, count as usize);
        }
    }
    let path = f.lock().unwrap().get_path().clone();
    let read_res = forward_read(&path, buf, offset, count);
    //println!("finish: {}", read_res.0);
    if read_res.0 != 0 {
        error_msg(
            "client::sfs_pread".to_string(),
            format!("read with error {}", read_res.0),
        );
        set_errno(Errno(read_res.0));
        return -1;
    }
    return read_res.1 as i64;
}
#[no_mangle]
pub extern "C" fn sfs_pread(fd: i32, buf: *mut c_char, count: i64, offset: i64) -> i64 {
    let f = DynamicContext::get_instance()
        .get_ofm()
        .lock()
        .unwrap()
        .get(fd);
    if let None = f {
        error_msg(
            "client::sfs_pread".to_string(),
            "file not exist".to_string(),
        );
        set_errno(Errno(EBADF));
        return -1;
    }
    let f = f.unwrap();
    return internal_pread(Arc::clone(&f), buf, count, offset);
}
#[no_mangle]
pub extern "C" fn sfs_read(fd: i32, buf: *mut c_char, count: i64) -> i64 {
    let f = DynamicContext::get_instance()
        .get_ofm()
        .lock()
        .unwrap()
        .get(fd);
    if let None = f {
        error_msg("client::sfs_read".to_string(), "file not exist".to_string());
        set_errno(Errno(EBADF));
        return -1;
    }
    let f = f.unwrap();
    let pos = f.lock().unwrap().get_pos();
    let read_res = internal_pread(Arc::clone(&f), buf, count, pos);
    if read_res > 0 {
        f.lock().unwrap().set_pos(pos + count);
    }
    return read_res;
}
#[no_mangle]
pub extern "C" fn sfs_rmdir(path: *const c_char) -> i32 {
    let path = unsafe { CStr::from_ptr(path).to_string_lossy().into_owned() };
    let md_res = get_metadata(&path, false);
    if let Err(_e) = md_res {
        error_msg(
            "client::sfs_rmdir".to_string(),
            "file not exist".to_string(),
        );
        return -1;
    }
    let md = md_res.unwrap();
    if !S_ISDIR(md.get_mode()) {
        error_msg(
            "client::sfs_rmdir".to_string(),
            "path is not directory".to_string(),
        );
        set_errno(Errno(ENOTDIR));
        return -1;
    }
    let dirent_res = forward_get_dirents(&path);
    if dirent_res.0 != 0 {
        error_msg(
            "client::sfs_rmdir".to_string(),
            format!("forward get dirents with error {}", dirent_res.0),
        );
        set_errno(Errno(dirent_res.0));
        return -1;
    }
    let opendir = dirent_res.1;
    if opendir.lock().unwrap().get_size() != 0 {
        error_msg(
            "client::sfs_rmdir".to_string(),
            "directory not empty".to_string(),
        );
        set_errno(Errno(ENOTEMPTY));
        return -1;
    }
    let rm_res = forward_remove(path.clone(), true, 0);
    if let Err(_e) = rm_res {
        error_msg(
            "client::sfs_rmdir".to_string(),
            format!("forward remove directory with error {}", dirent_res.0),
        );
        return -1;
    }
    let err = rm_res.unwrap();
    if err != 0 {
        set_errno(Errno(err));
        return -1;
    }
    return 0;
}
#[no_mangle]
pub extern "C" fn sfs_opendir(path: *const c_char) -> i32 {
    let path = unsafe { CStr::from_ptr(path).to_string_lossy().into_owned() };
    let md_res = get_metadata(&path, false);
    if let Err(e) = md_res {
        error_msg(
            "client::sfs_opendir".to_string(),
            "file not exist".to_string(),
        );
        set_errno(Errno(e));
        return -1;
    }
    let md = md_res.unwrap();
    if !S_ISDIR(md.get_mode()) {
        error_msg(
            "client::sfs_opendir".to_string(),
            "path is not directory".to_string(),
        );
        set_errno(Errno(ENOTDIR));
        return -1;
    }
    let dirent_res = forward_get_dirents(&path);
    if dirent_res.0 != 0 {
        error_msg(
            "client::sfs_opendir".to_string(),
            format!("forward get dirents with error {}", dirent_res.0),
        );
        set_errno(Errno(dirent_res.0));
        return -1;
    }
    return DynamicContext::get_instance()
        .get_ofm()
        .lock()
        .unwrap()
        .add(dirent_res.1);
}
fn align(size: usize, step: usize) -> usize {
    (size + step) & (!step + 1)
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct linux_dirent {
    pub d_ino: u64,
    pub d_off: u64,
    pub d_reclen: u16,
    pub d_name: [u8; 256],
}
#[no_mangle]
pub extern "C" fn sfs_getdents(fd: i32, dirp: *mut linux_dirent, count: i64) -> i32 {
    unsafe {
        memset(dirp as *mut c_void, 0, count as usize);
    }
    let opendir = DynamicContext::get_instance()
        .get_ofm()
        .lock()
        .unwrap()
        .get_dir(fd);
    if let None = opendir {
        error_msg(
            "client::sfs_getdirents".to_string(),
            "directory not opned".to_string(),
        );
        set_errno(Errno(EBADF));
        return -1;
    }
    let opendir = opendir.unwrap();
    let mut pos = opendir.lock().unwrap().get_pos();
    if pos >= opendir.lock().unwrap().get_size() as i64 {
        return 0;
    }
    let mut written = 0;
    let size = opendir.lock().unwrap().get_size() as i64;
    while pos < size {
        let de = opendir.lock().unwrap().getdent(pos);
        let total_size = align(18 + de.get_name().len() + 1 + 1, 8);
        if total_size as i64 > count - written {
            break;
        }
        let current_dirp =
            unsafe { (dirp as *mut c_char).offset(written as isize) as *mut linux_dirent };
        let mut s = DefaultHasher::new();
        let p = opendir.lock().unwrap().get_path().clone() + "/" + &de.get_name();
        p.hash(&mut s);
        let name = de.get_name() + "\0";
        unsafe {
            (*current_dirp).d_ino = s.finish() as u64;
            (*current_dirp).d_reclen = total_size as u16;
            let c: u8;
            match de.get_type() {
                FileType::SFS_REGULAR => c = DT_REG,
                FileType::SFS_DIRECTORY => c = DT_DIR,
            }
            /*
            strcpy(
                (*current_dirp).d_name.as_ptr() as *mut i8,
                name.as_ptr() as *const i8,
            );
            */
            memcpy(
                (current_dirp as *const c_char).offset(18) as *mut c_void,
                name.as_ptr() as *const c_void,
                name.len(),
            );
            *(current_dirp as *mut u8).offset(total_size as isize - 1) = c;
            (*current_dirp).d_off = pos as u64;
            pos += 1;
            written += total_size as i64;
        }
    }
    opendir.lock().unwrap().set_pos(pos);
    return written as i32;
}
#[no_mangle]
pub extern "C" fn sfs_getdents64(fd: i32, dirp: *mut dirent64, count: i64) -> i32 {
    unsafe {
        memset(dirp as *mut c_void, 0, count as usize);
    }
    let opendir = DynamicContext::get_instance()
        .get_ofm()
        .lock()
        .unwrap()
        .get_dir(fd);
    if let None = opendir {
        error_msg(
            "client::sfs_getdirents".to_string(),
            "directory not opned".to_string(),
        );
        set_errno(Errno(EBADF));
        return -1;
    }
    let opendir = opendir.unwrap();
    let mut pos = opendir.lock().unwrap().get_pos();
    if pos >= opendir.lock().unwrap().get_size() as i64 {
        return 0;
    }
    let mut written = 0;
    let size = opendir.lock().unwrap().get_size() as i64;
    while pos < size {
        let de = opendir.lock().unwrap().getdent(pos);
        let total_size = align(19 + de.get_name().len() + 1, 8);
        if total_size as i64 > count - written {
            break;
        }
        let current_dirp = unsafe { (dirp as *mut c_char).offset(written as isize) as *mut dirent };
        let mut s = DefaultHasher::new();
        let p = opendir.lock().unwrap().get_path().clone() + "/" + &de.get_name();
        p.hash(&mut s);
        let name = de.get_name() + "\0";
        unsafe {
            (*current_dirp).d_ino = s.finish() as u64;
            (*current_dirp).d_reclen = total_size as u16;
            let c: u8;
            match de.get_type() {
                FileType::SFS_REGULAR => c = DT_REG,
                FileType::SFS_DIRECTORY => c = DT_DIR,
            }
            (*current_dirp).d_type = c;
            strcpy(
                (*current_dirp).d_name.as_ptr() as *mut i8,
                name.as_ptr() as *const i8,
            );
            /*
            memcpy(
                (*current_dirp).d_name.as_ptr() as *mut c_void,
                name.as_ptr() as *const c_void,
                name.len(),
            );
            */
            (*current_dirp).d_off = pos;
            pos += 1;
            written += total_size as i64;
        }
    }
    if written == 0 {
        return -1;
    }
    opendir.lock().unwrap().set_pos(pos);
    return 0;
}
