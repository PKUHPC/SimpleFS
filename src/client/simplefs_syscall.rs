use std::ffi::CStr;
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn sfs_open(path: * const c_char, mode: i32, flag: i32) -> i32{
    let s = unsafe { CStr::from_ptr(path).to_string_lossy().into_owned() };
    return -1;
}

#[no_mangle]
pub extern "C" fn sfs_create(path: * const c_char, mode: i32) -> i32{
    return -1;
}

#[no_mangle]
pub extern "C" fn sfs_remove(path: * const c_char) -> i32{
    return -1;
}

#[no_mangle]
pub extern "C" fn sfs_access(path: * const c_char, mask: i32, follow_links: bool) -> i32{
    return -1;
}

//#[no_mangle]
//pub extern "C" fn sfs_stat(path: * const c_char, buf: * stat, follow_links: bool) -> i32{
//    return -1;
//}

#[no_mangle]
pub extern "C" fn sfs_truncate(path: * const c_char, offset: i32) -> i32{
    return -1;
}

#[no_mangle]
pub extern "C" fn sfs_dup(oldfd: i32) -> i32{
    return -1;
}

#[no_mangle]
pub extern "C" fn sfs_read(fd: i32, buf: * const char, count: u32) -> i32{
    return -1;
}

#[no_mangle]
pub extern "C" fn sfs_write(fd: i32, buf: * const char, count: u32) -> i32{
    return -1;
}

#[no_mangle]
pub extern "C" fn sfs_rmdir(path: * const c_char) -> i32{
    return -1;
}

//#[no_mangle]
//pub extern "C" fn sfs_getdents(fd: i32, dirp: *linux_dirent, count: u32) -> i32{
//    return -1;
//}
