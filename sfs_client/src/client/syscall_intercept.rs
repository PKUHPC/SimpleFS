use std::{os::raw::c_char, ffi::CStr};

use libc::{EINVAL, ENOTDIR, SYS_openat, SYS_close, SYS_stat, SYS_lstat, SYS_fstat, SYS_read, SYS_pread64, SYS_pwrite64, SYS_write, SYS_unlinkat, AT_REMOVEDIR, SYS_access, SYS_faccessat, SYS_lseek, EOVERFLOW, SYS_truncate, SYS_ftruncate, SYS_dup, SYS_dup2, ENOTSUP, SYS_dup3, SYS_symlinkat, dirent, SYS_getdents, SYS_getdents64, dirent64, SYS_mkdir, S_IFDIR, SYS_mkdirat, SYS_fchmodat, SYS_fchmod, EBADF, SYS_fchdir};

use crate::global::{metadata::S_ISDIR, util::path_util::has_trailing_slash, error_msg::error_msg};

use super::{context::{DynamicContext, RelativizeStatus, StaticContext}, syscall::{sfs_open, sfs_stat, stat, sfs_read, sfs_pread, sfs_write, sfs_pwrite, sfs_rmdir, sfs_remove, sfs_access, sfs_lseek, sfs_truncate, sfs_dup, sfs_dup2, sfs_getdents, sfs_getdents64, sfs_create}, util::get_metadata, path::{set_cwd, unset_env_cwd, get_sys_cwd}};

#[no_mangle]
pub extern "C" fn hook_openat(dirfd: i32, path: * const c_char, mode: u32, flag: i32) -> i32{
    let raw_path = unsafe { CStr::from_ptr(path).to_string_lossy().into_owned() };
    let res = DynamicContext::get_instance().relativize_fd_path(dirfd, &raw_path, false);
    let rstatus = res.0;
    let resolved = res.1 + "\0";
    match rstatus {
        RelativizeStatus::Internal => {
            sfs_open(resolved.as_ptr() as * const c_char, mode, flag)
        },
        RelativizeStatus::External => {
            unsafe {syscall_no_intercept(SYS_openat, dirfd, resolved.as_ptr() as * const c_char, flag, mode) as i32}
        },
        RelativizeStatus::FdUnknown => {
            unsafe {syscall_no_intercept(SYS_openat, dirfd, path, flag, mode) as i32}
        },
        RelativizeStatus::FdNotADir => {-ENOTDIR},
        RelativizeStatus::Error => {-EINVAL}
    }
}
#[no_mangle]
pub extern "C" fn hook_close(fd: i32) -> i32{
    if DynamicContext::get_instance().get_ofm().lock().unwrap().exist(fd){
        DynamicContext::get_instance().get_ofm().lock().unwrap().remove(fd);
        return 0;
    }
    if DynamicContext::get_instance().is_internel_fd(fd){
        return 0;
    }
    return unsafe{syscall_no_intercept(SYS_close, fd) as i32};
}
#[no_mangle]
pub extern "C" fn hook_stat(path: * const c_char, buf: * mut stat) -> i32{
    let raw_path = unsafe { CStr::from_ptr(path).to_string_lossy().into_owned() };
    let res = DynamicContext::get_instance().relativize_path(&raw_path, false);
    let rel_path = res.1 + "\0";
    if res.0{
        return sfs_stat(rel_path.as_ptr() as *const c_char, buf, false);
    }
    return unsafe{syscall_no_intercept(SYS_stat, rel_path.as_ptr() as *const c_char, buf) as i32};
}
#[no_mangle]
pub extern "C" fn hook_lstat(path: * const c_char, buf: * mut stat) -> i32{
    let raw_path = unsafe { CStr::from_ptr(path).to_string_lossy().into_owned() };
    let res = DynamicContext::get_instance().relativize_path(&raw_path, false);
    let rel_path = res.1 + "\0";
    if res.0{
        return sfs_stat(rel_path.as_ptr() as *const c_char, buf, false);
    }
    return unsafe{syscall_no_intercept(SYS_lstat, rel_path.as_ptr() as *const c_char, buf) as i32};
}
#[no_mangle]
pub extern "C" fn hook_fstat(fd: i32, buf: * mut stat) -> i32{
    if DynamicContext::get_instance().get_ofm().lock().unwrap().exist(fd){
        let path = DynamicContext::get_instance().get_ofm().lock().unwrap().get(fd).unwrap().lock().unwrap().get_path().clone() + "\0";
        return sfs_stat(path.as_ptr() as *const c_char, buf, false);
    }
    return unsafe{syscall_no_intercept(SYS_fstat, fd, buf) as i32};
}
#[no_mangle]
pub extern "C" fn hook_fstatat(dirfd: i32, path: * const c_char, buf: * mut stat, flags: i32) -> i32{
    let raw_path = unsafe { CStr::from_ptr(path).to_string_lossy().into_owned() };
    let res = DynamicContext::get_instance().relativize_fd_path(dirfd, &raw_path, false);
    let rstatus = res.0;
    let resolved = res.1 + "\0";
    match rstatus {
        RelativizeStatus::Internal => {
            sfs_stat(resolved.as_ptr() as * const c_char, buf, false)
        },
        RelativizeStatus::External => {
            unsafe {syscall_no_intercept(SYS_fstat, dirfd, resolved.as_ptr() as * const c_char, buf, flags) as i32}
        },
        RelativizeStatus::FdUnknown => {
            unsafe {syscall_no_intercept(SYS_fstat, dirfd, path, buf, flags) as i32}
        },
        RelativizeStatus::FdNotADir => {-ENOTDIR},
        RelativizeStatus::Error => {-EINVAL}
    }
}
#[no_mangle]
pub extern "C" fn hook_read(fd: i32, buf: *mut c_char, count: i64) -> i32{
    if DynamicContext::get_instance().get_ofm().lock().unwrap().exist(fd){
        return sfs_read(fd, buf, count) as i32;
    }
    return unsafe{syscall_no_intercept(SYS_read, fd, buf, count) as i32};
}
#[no_mangle]
pub extern "C" fn hook_pread(fd: i32, buf: *mut c_char, count: i64, pos: i64) -> i32{
    if DynamicContext::get_instance().get_ofm().lock().unwrap().exist(fd){
        return sfs_pread(fd, buf, count, pos) as i32;
    }
    return unsafe{syscall_no_intercept(SYS_pread64, fd, buf, count, pos) as i32};
}
#[no_mangle]
pub extern "C" fn hook_write(fd: i32, buf: *const c_char, count: i64) -> i32{
    if DynamicContext::get_instance().get_ofm().lock().unwrap().exist(fd){
        return sfs_write(fd, buf, count) as i32;
    }
    return unsafe{syscall_no_intercept(SYS_write, fd, buf, count) as i32};
}
#[no_mangle]
pub extern "C" fn hook_pwrite(fd: i32, buf: *const c_char, count: i64, pos: i64) -> i32{
    if DynamicContext::get_instance().get_ofm().lock().unwrap().exist(fd){
        return sfs_pwrite(fd, buf, count, pos) as i32;
    }
    return unsafe{syscall_no_intercept(SYS_pwrite64, fd, buf, count, pos) as i32};
}
#[no_mangle]
pub extern "C" fn hook_unlinkat(dirfd: i32, path: * const c_char, flags: i32) -> i32{
    let raw_path = unsafe { CStr::from_ptr(path).to_string_lossy().into_owned() };
    let res = DynamicContext::get_instance().relativize_fd_path(dirfd, &raw_path, false);
    let rstatus = res.0;
    let resolved = res.1 + "\0";
    match rstatus {
        RelativizeStatus::Internal => {
            if flags & AT_REMOVEDIR != 0{
                sfs_rmdir(resolved.as_ptr() as * const c_char)
            }
            else{
                sfs_remove(resolved.as_ptr() as * const c_char)
            }
        },
        RelativizeStatus::External => {
            unsafe {syscall_no_intercept(SYS_unlinkat, dirfd, resolved.as_ptr() as * const c_char, flags) as i32}
        },
        RelativizeStatus::FdUnknown => {
            unsafe {syscall_no_intercept(SYS_unlinkat, dirfd, path, flags) as i32}
        },
        RelativizeStatus::FdNotADir => {-ENOTDIR},
        RelativizeStatus::Error => {-EINVAL}
    }
}
#[no_mangle]
pub extern "C" fn hook_access(path: * const c_char, mask: i32) -> i32{
    let raw_path = unsafe { CStr::from_ptr(path).to_string_lossy().into_owned() };
    let res = DynamicContext::get_instance().relativize_path(&raw_path, false);
    let rel_path = res.1 + "\0";
    if res.0{
        let ret = sfs_access(rel_path.as_ptr() as * const c_char, mask, false);
        if ret < 0{
            return -1;
        }
        return ret;
    }
    return unsafe{syscall_no_intercept(SYS_access, rel_path.as_ptr() as *const c_char, mask) as i32};
}
#[no_mangle]
pub extern "C" fn hook_faccessat(dirfd: i32, path: * const c_char, mode: i32) -> i32{
    let raw_path = unsafe { CStr::from_ptr(path).to_string_lossy().into_owned() };
    let res = DynamicContext::get_instance().relativize_fd_path(dirfd, &raw_path, false);
    let rstatus = res.0;
    let resolved = res.1 + "\0";
    match rstatus {
        RelativizeStatus::Internal => {
            sfs_access(resolved.as_ptr() as *const c_char, mode, false)
        },
        RelativizeStatus::External => {
            unsafe {syscall_no_intercept(SYS_faccessat, dirfd, resolved.as_ptr() as * const c_char, mode) as i32}
        },
        RelativizeStatus::FdUnknown => {
            unsafe {syscall_no_intercept(SYS_faccessat, dirfd, path, mode) as i32}
        },
        RelativizeStatus::FdNotADir => {-ENOTDIR},
        RelativizeStatus::Error => {-EINVAL}
    }
}
#[no_mangle]
pub extern "C" fn hook_lseek(fd: i32, offset: i64, whence: i32) -> i64{
    if DynamicContext::get_instance().get_ofm().lock().unwrap().exist(fd){
        let ret = sfs_lseek(fd, offset, whence);
        if ret > i64::MAX{
            return -EOVERFLOW as i64;
        }
        else if ret < 0{
            return -1;
        }
        return ret;
    }
    return unsafe{syscall_no_intercept(SYS_lseek, fd, offset, whence) as i64};
}
#[no_mangle]
pub extern "C" fn hook_truncate(path: * const c_char, length: i64) -> i32{
    let raw_path = unsafe { CStr::from_ptr(path).to_string_lossy().into_owned() };
    let res = DynamicContext::get_instance().relativize_path(&raw_path, false);
    let rel_path = res.1 + "\0";
    if res.0{
        return sfs_truncate(rel_path.as_ptr() as *const c_char, length);
    }
    return unsafe{syscall_no_intercept(SYS_truncate, rel_path.as_ptr() as *const c_char, length) as i32};
}
#[no_mangle]
pub extern "C" fn hook_ftruncate(fd: i32, length: i64) -> i32{
    if DynamicContext::get_instance().get_ofm().lock().unwrap().exist(fd){
        let path = DynamicContext::get_instance().get_ofm().lock().unwrap().get(fd).unwrap().lock().unwrap().get_path().clone();
        return sfs_truncate(path.as_ptr() as *const c_char, length);
    }
    return unsafe{syscall_no_intercept(SYS_ftruncate, fd, length) as i32};
}
#[no_mangle]
pub extern "C" fn hook_dup(fd: i32) -> i32{
    if DynamicContext::get_instance().get_ofm().lock().unwrap().exist(fd){
        return sfs_dup(fd);
    }
    return unsafe{syscall_no_intercept(SYS_dup, fd) as i32};
}
#[no_mangle]
pub extern "C" fn hook_dup2(oldfd: i32, newfd: i32) -> i32{
    if DynamicContext::get_instance().get_ofm().lock().unwrap().exist(oldfd){
        return sfs_dup2(oldfd, newfd);
    }
    return unsafe{syscall_no_intercept(SYS_dup2, oldfd, newfd) as i32};
}
#[no_mangle]
pub extern "C" fn hook_dup3(oldfd: i32, newfd: i32, flags: i32) -> i32{
    if DynamicContext::get_instance().get_ofm().lock().unwrap().exist(oldfd){
        return -ENOTSUP;
    }
    return unsafe{syscall_no_intercept(SYS_dup3, oldfd, newfd, flags) as i32};
}
#[no_mangle]
pub extern "C" fn hook_symlinkat(oldname: *const c_char, newdfd: i32, newname: *const c_char) -> i32{
    let old_raw_path = unsafe { CStr::from_ptr(oldname).to_string_lossy().into_owned() };
    let old_res = DynamicContext::get_instance().relativize_path(&old_raw_path, false);
    let old_rel_path = old_res.1 + "\0";
    if old_res.0{
        return -ENOTSUP;
    }

    let new_raw_path = unsafe { CStr::from_ptr(newname).to_string_lossy().into_owned() };
    let new_res = DynamicContext::get_instance().relativize_fd_path(newdfd, &new_raw_path, false);
    let rstatus = new_res.0;
    let new_resolved = new_res.1 + "\0";
    match rstatus {
        RelativizeStatus::Internal => {
            -ENOTSUP
        },
        RelativizeStatus::External => {
            unsafe {syscall_no_intercept(SYS_symlinkat, oldname, newdfd, new_resolved.as_ptr() as * const c_char) as i32}
        },
        RelativizeStatus::FdUnknown => {
            unsafe {syscall_no_intercept(SYS_symlinkat, oldname, newdfd, newname) as i32}
        },
        RelativizeStatus::FdNotADir => {-ENOTDIR},
        RelativizeStatus::Error => {-EINVAL}
    }
}
#[no_mangle]
pub extern "C" fn hook_getdents(fd: i32, dirp: *mut dirent, count: i64) -> i32{
    if DynamicContext::get_instance().get_ofm().lock().unwrap().exist(fd){
        return sfs_getdents(fd, dirp, count);
    }
    return unsafe{syscall_no_intercept(SYS_getdents, fd, dirp, count) as i32};
}
#[no_mangle]
pub extern "C" fn hook_getdents64(fd: i32, dirp: *mut dirent64, count: i64) -> i32{
    if DynamicContext::get_instance().get_ofm().lock().unwrap().exist(fd){
        return sfs_getdents64(fd, dirp, count);
    }
    return unsafe{syscall_no_intercept(SYS_getdents64, fd, dirp, count) as i32};
}
#[no_mangle]
pub extern "C" fn hook_mkdir(dirfd: i32, path: * const c_char, mode: u32) -> i32{
    let raw_path = unsafe { CStr::from_ptr(path).to_string_lossy().into_owned() };
    let res = DynamicContext::get_instance().relativize_fd_path(dirfd, &raw_path, false);
    let rstatus = res.0;
    let resolved = res.1 + "\0";
    match rstatus {
        RelativizeStatus::Internal => {
            sfs_create(resolved.as_ptr() as *const c_char, mode | S_IFDIR)
        },
        RelativizeStatus::External => {
            unsafe {syscall_no_intercept(SYS_mkdirat, dirfd, resolved.as_ptr() as * const c_char, mode) as i32}
        },
        RelativizeStatus::FdUnknown => {
            unsafe {syscall_no_intercept(SYS_mkdirat, dirfd, path, mode) as i32}
        },
        RelativizeStatus::FdNotADir => {-ENOTDIR},
        RelativizeStatus::Error => {-EINVAL}
    }
}
#[no_mangle]
pub extern "C" fn hook_fchmodat(dirfd: i32, path: * const c_char, mode: u32) -> i32{
    let raw_path = unsafe { CStr::from_ptr(path).to_string_lossy().into_owned() };
    let res = DynamicContext::get_instance().relativize_fd_path(dirfd, &raw_path, false);
    let rstatus = res.0;
    let resolved = res.1 + "\0";
    match rstatus {
        RelativizeStatus::Internal => {
            -ENOTSUP
        },
        RelativizeStatus::External => {
            unsafe {syscall_no_intercept(SYS_fchmodat, dirfd, resolved.as_ptr() as * const c_char, mode) as i32}
        },
        RelativizeStatus::FdUnknown => {
            unsafe {syscall_no_intercept(SYS_fchmodat, dirfd, path, mode) as i32}
        },
        RelativizeStatus::FdNotADir => {-ENOTDIR},
        RelativizeStatus::Error => {-EINVAL}
    }
}
#[no_mangle]
pub extern "C" fn hook_fchmod(fd: i32, mode: u32) -> i32{
    if DynamicContext::get_instance().get_ofm().lock().unwrap().exist(fd){
        return -ENOTSUP;
    }
    return unsafe{syscall_no_intercept(SYS_fchmod, mode) as i32};
}
#[no_mangle]
pub extern "C" fn hook_chdir(path: *const c_char) -> i32{
    let raw_path = unsafe { CStr::from_ptr(path).to_string_lossy().into_owned() };
    let res = DynamicContext::get_instance().relativize_path(&raw_path, false);
    let mut rel_path = res.1 + "\0";
    if res.0{
        let md_res = get_metadata(&rel_path, false);
        if let Err(e) = md_res{
            return -1;
        }
        let md = md_res.unwrap();
        if !S_ISDIR(md.get_mode()){
            return -ENOTDIR;
        }
        rel_path = StaticContext::get_instance().get_mountdir().clone() + &rel_path;
        if has_trailing_slash(&rel_path){
            rel_path.pop().unwrap();
        }
    }
    return set_cwd(&rel_path, res.0);
}
#[no_mangle]
pub extern "C" fn hook_fchdir(fd: i32) -> i32{
    if DynamicContext::get_instance().get_ofm().lock().unwrap().exist(fd){
        let opendir = DynamicContext::get_instance().get_ofm().lock().unwrap().get_dir(fd);
        if let None = opendir{
            error_msg("client::hook_fchdir".to_string(), format!("file descriptor {} is not a directory", fd));
            return -EBADF;
        }
        let opendir = opendir.unwrap();
        let mut new_path = StaticContext::get_instance().get_mountdir().clone() + opendir.lock().unwrap().get_path();
        if has_trailing_slash(&new_path){
            new_path.pop().unwrap();
            return set_cwd(&new_path, true)
        }
        else{
            let ret = unsafe{syscall_no_intercept(SYS_fchdir, fd) as i32};
            if ret < 0{
                return -1;
            }
            unset_env_cwd();
            DynamicContext::get_instance().set_cwd(get_sys_cwd());
        }
    }
    return 0;
}

#[link(name = "syscall_intercept", kind = "static")]
extern "C" {
    pub fn syscall_no_intercept(
        syscall_number: ::std::os::raw::c_long,
        ...
    ) -> ::std::os::raw::c_long;
}
