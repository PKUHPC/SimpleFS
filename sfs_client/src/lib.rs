use std::ffi::CStr;

use client::{
    context::{interception_enabled, DynamicContext, StaticContext},
    openfile::OpenFileFlags,
    util::get_metadata,
};
use libc::{c_char, strcpy};

pub mod client;

#[no_mangle]
pub extern "C" fn relativize_fd_path(
    dirfd: i32,
    cpath: *const c_char,
    resolved: *mut c_char,
    _follow_links: bool,
) -> i32 {
    let path = unsafe { CStr::from_ptr(cpath).to_string_lossy().into_owned() };
    let ret = DynamicContext::get_instance().relativize_fd_path(dirfd, &path, false);
    let resolved_str = ret.1 + "\0";
    unsafe {
        strcpy(resolved, resolved_str.as_ptr() as *const i8);
    }
    match ret.0 {
        client::context::RelativizeStatus::Internal => 0,
        client::context::RelativizeStatus::External => 1,
        client::context::RelativizeStatus::FdUnknown => 2,
        client::context::RelativizeStatus::FdNotADir => 3,
        client::context::RelativizeStatus::Error => -1,
    }
}
#[no_mangle]
pub extern "C" fn relativize_path(
    path: *const c_char,
    rel_path: *mut c_char,
    _follow_links: bool,
) -> bool {
    let path = unsafe { CStr::from_ptr(path).to_string_lossy().into_owned() };
    let ret = DynamicContext::get_instance().relativize_path(&path, false);
    let rel_path_str = ret.1 + "\0";
    unsafe {
        strcpy(rel_path, rel_path_str.as_ptr() as *const i8);
    }
    return ret.0;
}
#[no_mangle]
pub extern "C" fn fd_exist(fd: i32) -> bool {
    DynamicContext::get_instance()
        .get_ofm()
        .lock()
        .unwrap()
        .exist(fd)
}
#[no_mangle]
pub extern "C" fn fd_remove(fd: i32) {
    DynamicContext::get_instance()
        .get_ofm()
        .lock()
        .unwrap()
        .remove(fd);
}
#[no_mangle]
pub extern "C" fn fd_is_internal(fd: i32) -> bool {
    DynamicContext::get_instance().is_internal_fd(fd)
}
#[no_mangle]
pub extern "C" fn fd_get_path(fd: i32, path: *mut c_char) {
    let cpath = DynamicContext::get_instance()
        .get_ofm()
        .lock()
        .unwrap()
        .get(fd)
        .unwrap()
        .lock()
        .unwrap()
        .get_path()
        .clone()
        + "\0";
    unsafe {
        strcpy(path, cpath.as_ptr() as *const i8);
    }
}
#[no_mangle]
pub extern "C" fn fd_get_dir_path(fd: i32, path: *mut c_char) {
    let cpath = DynamicContext::get_instance()
        .get_ofm()
        .lock()
        .unwrap()
        .get_dir(fd)
        .unwrap()
        .lock()
        .unwrap()
        .get_path()
        .clone()
        + "\0";
    unsafe {
        strcpy(path, cpath.as_ptr() as *const i8);
    }
}
fn i2flag(flag: i32) -> OpenFileFlags {
    match flag {
        0 => OpenFileFlags::Append,
        1 => OpenFileFlags::Creat,
        2 => OpenFileFlags::Trunc,
        3 => OpenFileFlags::Rdonly,
        4 => OpenFileFlags::Wronly,
        5 => OpenFileFlags::Rdwr,
        6 => OpenFileFlags::Cloexec,
        7 => OpenFileFlags::FlagCount,
        _ => OpenFileFlags::Unknown,
    }
}
#[no_mangle]
pub extern "C" fn set_flag(fd: i32, flag: i32, val: bool) {
    DynamicContext::get_instance()
        .get_ofm()
        .lock()
        .unwrap()
        .get(fd)
        .unwrap()
        .lock()
        .unwrap()
        .set_flag(i2flag(flag), val);
}
#[no_mangle]
pub extern "C" fn get_flag(fd: i32, flag: i32) -> bool {
    DynamicContext::get_instance()
        .get_ofm()
        .lock()
        .unwrap()
        .get(fd)
        .unwrap()
        .lock()
        .unwrap()
        .get_flag(i2flag(flag))
}
#[no_mangle]
pub extern "C" fn get_mountdir(path: *mut c_char) {
    let mountdir_str = StaticContext::get_instance().get_mountdir().clone() + "\0";
    unsafe {
        strcpy(path, mountdir_str.as_ptr() as *const i8);
    }
}
#[no_mangle]
pub extern "C" fn get_ctx_cwd(cwd: *mut c_char) {
    let cwd_str = DynamicContext::get_instance().get_cwd().clone() + "\0";
    unsafe {
        strcpy(cwd, cwd_str.as_ptr() as *const i8);
    }
}
#[no_mangle]
pub extern "C" fn set_ctx_cwd(cwd: *const c_char) {
    let cwd = unsafe { CStr::from_ptr(cwd).to_string_lossy().into_owned() };
    DynamicContext::get_instance().set_cwd(cwd);
}
#[no_mangle]
pub extern "C" fn set_cwd(cwd: *const c_char, internal: bool) {
    let cwd = unsafe { CStr::from_ptr(cwd).to_string_lossy().into_owned() };
    crate::client::path::set_cwd(&cwd, internal);
}
#[no_mangle]
pub extern "C" fn unset_env_cwd() {
    crate::client::path::unset_env_cwd();
}
#[no_mangle]
pub extern "C" fn get_sys_cwd(cwd: *mut c_char) {
    let cwd_str = crate::client::path::get_sys_cwd() + "\0";
    unsafe {
        strcpy(cwd, cwd_str.as_ptr() as *const i8);
    }
}
#[no_mangle]
pub extern "C" fn get_md_mode(path: *const c_char) -> i32 {
    let path = unsafe { CStr::from_ptr(path).to_string_lossy().into_owned() };
    let md_res = get_metadata(&path, false);
    if let Err(_e) = md_res {
        return -1;
    }
    return md_res.unwrap().get_mode() as i32;
}
#[no_mangle]
pub extern "C" fn enable_interception() {
    interception_enabled();
}
#[cfg(test)]
mod tests {
    use std::{thread, time};

    #[allow(unused_imports)]
    use libc::{c_char, dirent, stat, O_CREAT, O_RDWR, SEEK_SET, S_IFDIR, S_IFREG};

    #[allow(unused_imports)]
    use crate::client::{
        context::{DynamicContext, StaticContext},
        path::resolve,
        syscall::{
            internal_truncate, sfs_create, sfs_dup, sfs_dup2, sfs_getdents, sfs_lseek, sfs_open,
            sfs_opendir, sfs_pread, sfs_pwrite, sfs_read, sfs_remove, sfs_rmdir, sfs_stat,
            sfs_write,
        },
    };
    use sfs_global::global::network::config::CHUNK_SIZE;
    use std::time::UNIX_EPOCH;

    #[test]
    fn test0() {
        println!(
            "{}",
            time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );
    }
    #[test]
    pub fn test1() {
        let s = "hello, here is the test data of sfs small-data local-host open/read/write test";

        let path = "/sfs/test/create_dir/file1\0".to_string();
        let path1 = "/sfs\0".to_string();
        let path2 = "/sfs/test\0".to_string();
        let path3 = "/sfs/test/create_dir\0".to_string();
        sfs_create(path1.as_ptr() as *const i8, S_IFDIR);
        sfs_create(path2.as_ptr() as *const i8, S_IFDIR);
        sfs_create(path3.as_ptr() as *const i8, S_IFDIR);

        //let res = sfs_create(path.as_str().as_ptr() as * const c_char, S_IFDIR);
        let fd = sfs_open(
            path.as_str().as_ptr() as *const c_char,
            S_IFREG,
            O_CREAT | O_RDWR,
        );
        if fd <= 0 {
            println!("open error ...");
            return;
        }
        println!("open result: {}", fd);
        println!(
            "ofm length: {}",
            DynamicContext::get_instance()
                .get_ofm()
                .lock()
                .unwrap()
                .get_length()
        );

        //let path = "/sfs/test/async_write/a".to_string();
        //let res = sfs_open(path.as_str().as_ptr() as * const c_char, S_IFREG, O_CREAT | O_RDONLY);
        //println!("open result: {}", res);
        //println!("ofm length: {}", DynamicContext::get_instance().get_ofm().lock().unwrap().get_length());
        let res = sfs_write(fd, s.as_ptr() as *mut i8, s.len() as i64);
        if res <= 0 {
            println!("write error ...");
            return;
        } else {
            println!("{} bytes written ...", res);
        }
        sfs_lseek(fd, 13, SEEK_SET);
        let mut buf = vec![0 as u8; 100 as usize];
        let res = sfs_read(fd, buf.as_mut_ptr() as *mut i8, 100 as i64);
        if res <= 0 {
            println!("read error ...");
            return;
        } else {
            println!("read: {}", String::from_utf8(buf.to_vec()).unwrap());
        }
    }
    #[test]
    pub fn test2() {
        let _s = "hello, here is the test data of sfs small-data local-host create/opendir test"
            .to_string();

        let path = "/sfs/test/create_dir/file1\0".to_string();
        let path1 = "/sfs\0".to_string();
        let path2 = "/sfs/test\0".to_string();
        let path3 = "/sfs/test/create_dir\0".to_string();
        let _res1 = sfs_create(path1.as_ptr() as *const i8, S_IFDIR);
        let _res2 = sfs_create(path2.as_ptr() as *const i8, S_IFDIR);
        let _res3 = sfs_create(path3.as_ptr() as *const i8, S_IFDIR);

        let _fd = sfs_open(
            path.as_str().as_ptr() as *const c_char,
            S_IFREG,
            O_CREAT | O_RDWR,
        );
        let file_path1 = "/sfs/test/create_dir/file2\0".to_string();
        let _fd = sfs_open(
            file_path1.as_str().as_ptr() as *const c_char,
            S_IFREG,
            O_CREAT | O_RDWR,
        );

        let dir_path1 = "/sfs/test/create_dir\0".to_string();
        let dir_path2 = "/sfs/test/open_dir\0".to_string();
        let dir_path3 = "/sfs/test\0".to_string();

        let fd = sfs_opendir(dir_path1.as_str().as_ptr() as *const c_char);
        println!("open dir result: {}", fd);
        println!(
            "ofm length: {}",
            DynamicContext::get_instance()
                .get_ofm()
                .lock()
                .unwrap()
                .get_length()
        );
        println!(
            "dirents: {:?}\n",
            (*DynamicContext::get_instance()
                .get_ofm()
                .lock()
                .unwrap()
                .get(fd)
                .unwrap()
                .lock()
                .unwrap())
            .entries_
        );

        let _res = sfs_create(dir_path2.as_ptr() as *const i8, S_IFDIR);
        let file_path2 = "/sfs/test/file1\0".to_string();
        let _fd = sfs_open(
            file_path2.as_str().as_ptr() as *const c_char,
            S_IFREG,
            O_CREAT | O_RDWR,
        );

        let fd = sfs_opendir(dir_path3.as_str().as_ptr() as *const c_char);
        println!("open dir result: {}", fd);
        println!(
            "ofm length: {}",
            DynamicContext::get_instance()
                .get_ofm()
                .lock()
                .unwrap()
                .get_length()
        );
        println!(
            "dirents: {:?}",
            (*DynamicContext::get_instance()
                .get_ofm()
                .lock()
                .unwrap()
                .get(fd)
                .unwrap()
                .lock()
                .unwrap())
            .entries_
        );
    }
    #[test]
    pub fn test3() {
        let data = "hello, here is the test data of sfs small-data local-host remove test";

        let dpath_sfs = "/sfs\0".to_string();
        let _cres = sfs_create(dpath_sfs.as_ptr() as *const i8, S_IFDIR);

        let fpath_file1 = "/sfs/file1\0".to_string();
        let fd = sfs_open(
            fpath_file1.as_str().as_ptr() as *const c_char,
            S_IFREG,
            O_CREAT | O_RDWR,
        );

        let len = data.len() as i64;
        let _wres = sfs_write(fd, data.as_ptr() as *mut i8, len);

        sfs_lseek(fd, 0, SEEK_SET);
        let buf = vec![0 as u8; len as usize];
        let res = sfs_read(fd, buf.as_ptr() as *mut i8, len);
        if res <= 0 {
            println!("read error ...");
            return;
        } else {
            println!("read: {}", String::from_utf8(buf).unwrap());
        }

        let fd = sfs_opendir(dpath_sfs.as_str().as_ptr() as *const c_char);
        println!(
            "dirents of {}: {:?}",
            dpath_sfs,
            (*DynamicContext::get_instance()
                .get_ofm()
                .lock()
                .unwrap()
                .get(fd)
                .unwrap()
                .lock()
                .unwrap())
            .entries_
        );

        sfs_remove(fpath_file1.as_str().as_ptr() as *const c_char);
        let fd = sfs_opendir(dpath_sfs.as_str().as_ptr() as *const c_char);
        println!(
            "dirents of {}: {:?}",
            dpath_sfs,
            (*DynamicContext::get_instance()
                .get_ofm()
                .lock()
                .unwrap()
                .get(fd)
                .unwrap()
                .lock()
                .unwrap())
            .entries_
        );

        sfs_remove(dpath_sfs.as_str().as_ptr() as *const c_char);
        let fd = sfs_opendir(dpath_sfs.as_str().as_ptr() as *const c_char);
        if fd != -1 {
            println!("error remove dir ...");
        }
    }
    #[test]
    pub fn test4() {
        let data = "hello, here is the test data of sfs small-data local-host truncate test";

        let dpath_sfs = "/sfs\0".to_string();
        let _cres = sfs_create(dpath_sfs.as_ptr() as *const i8, S_IFDIR);

        let fpath_file1 = "/sfs/file1\0".to_string();
        let fd = sfs_open(
            fpath_file1.as_str().as_ptr() as *const c_char,
            S_IFREG,
            O_CREAT | O_RDWR,
        );

        let len = data.len() as i64;
        let _wres = sfs_write(fd, data.as_ptr() as *mut i8, len);

        sfs_lseek(fd, 0, SEEK_SET);
        let buf = vec![0 as u8; len as usize];
        let res = sfs_read(fd, buf.as_ptr() as *mut i8, len);
        if res <= 0 {
            println!("read error ...");
            return;
        } else {
            println!("read: {}", String::from_utf8(buf).unwrap());
        }

        let tres = internal_truncate(fpath_file1.as_str().as_ptr() as *const c_char, len, 13);
        if tres != 0 {
            println!("truncate error ...");
            return;
        }

        sfs_lseek(fd, 0, SEEK_SET);
        let buf = vec![0 as u8; len as usize];
        let res = sfs_read(fd, buf.as_ptr() as *mut i8, len);
        if res <= 0 {
            println!("read error ...");
            return;
        } else {
            println!("read: {}", String::from_utf8(buf).unwrap());
        }
    }
    #[test]
    pub fn test6() {
        let data = "hello, here is the test data of sfs small-data local-host dup test";

        let dpath_sfs = "/sfs\0".to_string();
        let _cres = sfs_create(dpath_sfs.as_ptr() as *const i8, S_IFDIR);

        let fpath_file1 = "/sfs/file1\0".to_string();
        let fd = sfs_open(
            fpath_file1.as_str().as_ptr() as *const c_char,
            S_IFREG,
            O_CREAT | O_RDWR,
        );

        let fd2 = sfs_dup(fd);
        if fd2 <= 0 {
            println!("dup error ...");
            return;
        }
        println!("dup {} to {}", fd, fd2);

        let _wres = sfs_write(fd2, data.as_ptr() as *mut i8, data.len() as i64);
        sfs_lseek(fd, 0, SEEK_SET);
        let buf = vec![0 as u8; data.len()];
        let res = sfs_read(fd, buf.as_ptr() as *mut i8, data.len() as i64);
        if res <= 0 {
            println!("read error ...");
            return;
        } else {
            println!("read from origin fd: {}", String::from_utf8(buf).unwrap());
        }

        sfs_lseek(fd2, 0, SEEK_SET);
        let buf = vec![0 as u8; data.len()];
        let res = sfs_read(fd2, buf.as_ptr() as *mut i8, data.len() as i64);
        if res <= 0 {
            println!("read error ...");
            return;
        } else {
            println!("read from dupped fd: {}", String::from_utf8(buf).unwrap());
        }
    }
    #[test]
    pub fn test7() {
        let data1 = "hello, there is the test data of sfs small-data local-host pwrite/pread test";
        let data2 = "hello, here is the test data of sfs small-data local-host pwrite/pread test";
        let dpath_sfs = "/sfs\0".to_string();
        let _cres = sfs_create(dpath_sfs.as_ptr() as *const i8, S_IFDIR);

        let fpath_file1 = "/sfs/file1\0".to_string();
        let fd = sfs_open(
            fpath_file1.as_str().as_ptr() as *const c_char,
            S_IFREG,
            O_CREAT | O_RDWR,
        );

        let _wres = sfs_write(fd, data1.as_ptr() as *mut i8, data1.len() as i64);
        let _wres = sfs_pwrite(fd, data2.as_ptr() as *mut i8, 10, 9);
        let buf = vec![0 as u8; 100];
        let res = sfs_pread(fd, buf.as_ptr() as *mut i8, 200, 3);
        if res <= 0 {
            println!("read error ...");
            return;
        } else {
            println!("read: {}", String::from_utf8(buf).unwrap());
        }
    }
    #[test]
    pub fn test8() {
        let _data = "hello, here is the test data of sfs small-data local-host rmdir test";

        let dpath_sfs = "/sfs\0".to_string();
        let _cres = sfs_create(dpath_sfs.as_ptr() as *const i8, S_IFDIR);

        let fpath_file1 = "/sfs/file1\0".to_string();
        let _fd = sfs_open(
            fpath_file1.as_str().as_ptr() as *const c_char,
            S_IFREG,
            O_CREAT | O_RDWR,
        );

        sfs_rmdir(dpath_sfs.as_ptr() as *const i8);
        sfs_remove(fpath_file1.as_ptr() as *const i8);
        sfs_rmdir(dpath_sfs.as_ptr() as *const i8);
    }
    /*
    #[test]
    pub fn test9() {
        let data = "hello, here is the test data of sfs small-data local-host getdents test";

        let dpath_sfs = "/sfs\0".to_string();
        sfs_create(dpath_sfs.as_ptr() as *const i8, S_IFDIR);

        let fpath_file1 = "/sfs/file1\0".to_string();
        let fpath_file2 = "/sfs/file2\0".to_string();
        let fpath_file3 = "/sfs/file3\0".to_string();
        let dpath_dir1 = "/sfs/dir1\0".to_string();
        let dpath_dir2 = "/sfs/dir2\0".to_string();
        sfs_open(
            fpath_file1.as_str().as_ptr() as *const c_char,
            S_IFREG,
            O_CREAT | O_RDWR,
        );
        sfs_open(
            fpath_file2.as_str().as_ptr() as *const c_char,
            S_IFREG,
            O_CREAT | O_RDWR,
        );
        sfs_open(
            fpath_file3.as_str().as_ptr() as *const c_char,
            S_IFREG,
            O_CREAT | O_RDWR,
        );
        sfs_create(dpath_dir1.as_ptr() as *const i8, S_IFDIR);
        sfs_create(dpath_dir2.as_ptr() as *const i8, S_IFDIR);

        let new_dirent = dirent {
            d_ino: 0,
            d_off: 0,
            d_reclen: 0,
            d_type: 0,
            d_name: [0; 256],
        };
        let dirents = [new_dirent; 20];
        let fd = sfs_opendir(dpath_sfs.as_str().as_ptr() as *const c_char);
        println!(
            "dirents of {}: {:?}",
            dpath_sfs,
            (*DynamicContext::get_instance()
                .get_ofm()
                .lock()
                .unwrap()
                .get(fd)
                .unwrap()
                .lock()
                .unwrap())
            .entries_
        );
        sfs_lseek(fd, 0, SEEK_SET);
        sfs_getdents(fd, dirents.as_ptr() as *mut dirent, 200);

        let mut dirent_ptr = dirents.as_ptr() as *const c_char;
        for i in 0..5 {
            let dirent = unsafe { *(dirent_ptr as *const dirent) };
            let total = dirent.d_reclen;
            dirent_ptr = unsafe { dirent_ptr.offset(total as isize) };

            let name_size = total - 19;
            let mut c_vec: Vec<u8> = Vec::new();

            let mut len = 0;
            for c in dirent.d_name {
                c_vec.push(c as u8);
                len += 1;
                if c == 0 {
                    break;
                }
            }
            c_vec = c_vec[0..len].to_vec();
            println!("{:?}: {}", dirent, String::from_utf8(c_vec).unwrap());
        }
    }
    */
    #[test]
    pub fn test_10() {
        let data = "hello, here is the test data of sfs small-data local-host dup2 test";

        let dpath_sfs = "/sfs\0".to_string();
        let _cres = sfs_create(dpath_sfs.as_ptr() as *const i8, S_IFDIR);

        let fpath_file1 = "/sfs/file1\0".to_string();
        let fd = sfs_open(
            fpath_file1.as_str().as_ptr() as *const c_char,
            S_IFREG,
            O_CREAT | O_RDWR,
        );
        let fd2 = 100010;
        let fd3 = sfs_dup2(fd, fd2);
        if fd2 != fd3 {
            println!("dup2 error ...");
            return;
        }
        println!("dup2 {} to {}", fd, fd3);

        let _wres = sfs_write(fd2, data.as_ptr() as *mut i8, data.len() as i64);
        sfs_lseek(fd, 0, SEEK_SET);
        let buf = vec![0 as u8; data.len()];
        let res = sfs_read(fd, buf.as_ptr() as *mut i8, data.len() as i64);
        if res <= 0 {
            println!("read error ...");
            return;
        } else {
            println!("read from origin fd: {}", String::from_utf8(buf).unwrap());
        }

        sfs_lseek(fd2, 0, SEEK_SET);
        let buf = vec![0 as u8; data.len()];
        let res = sfs_read(fd2, buf.as_ptr() as *mut i8, data.len() as i64);
        if res <= 0 {
            println!("read error ...");
            return;
        } else {
            println!("read from dupped fd: {}", String::from_utf8(buf).unwrap());
        }
    }
    #[test]
    pub fn test_bigdata() {
        let cnt = 1200;

        let path = "/file1\0".to_string();
        let fd = sfs_open(
            path.as_str().as_ptr() as *const c_char,
            S_IFREG,
            O_CREAT | O_RDWR,
        );
        if fd <= 0 {
            println!("open error ...");
            return;
        }
        println!("open result: {}", fd);
        println!(
            "ofm length: {}",
            DynamicContext::get_instance()
                .get_ofm()
                .lock()
                .unwrap()
                .get_length()
        );

        let s = vec!['a' as i8; cnt * CHUNK_SIZE as usize];
        let res = sfs_write(
            fd,
            s.as_ptr() as *mut i8,
            (cnt * CHUNK_SIZE as usize) as i64,
        );
        if res <= 0 {
            println!("write error ...");
            return;
        } else {
            println!("{} bytes written ...", res);
        }
        drop(s);
        /*
        sfs_lseek(fd, 13, SEEK_SET);
        let mut buf = vec![0 as u8; cnt * CHUNK_SIZE as usize];
        let res = sfs_read(fd, buf.as_mut_ptr() as *mut i8, cnt as i64 * CHUNK_SIZE as i64);
        if res <= 0 {
            println!("read error ...");
            return;
        } else {
            println!("{} bytes read", res);
        }
         */
    }
    #[test]
    #[allow(unused_must_use)]
    pub fn test_parallel() {
        let mut handles = Vec::new();
        let path = "/file".to_string() + (1 as i32).to_string().as_str() + "\0";
        let fd = sfs_open(
            path.as_str().as_ptr() as *const c_char,
            S_IFREG,
            O_CREAT | O_RDWR,
        );
        if fd <= 0 {
            println!("open error on thread {} ...", 1);
            return;
        }
        let thread = 100;
        let cnt = 3;
        for i in 0..thread {
            handles.push(thread::spawn(move || {
                //println!("file {} opened on thread {} ...", fd, i);
                let data = vec!['a' as i8; cnt * CHUNK_SIZE as usize];
                let res = sfs_write(fd, data.as_ptr() as *mut i8, data.len() as i64);
                if res <= 0 {
                    println!("write error on thread {} ...", i);
                    return;
                } else {
                    println!("{} bytes written on thread {} ...", res, i);
                }
            }))
        }
        for handle in handles {
            handle.join();
        }
        sfs_lseek(fd, 0, SEEK_SET);
        let mut buf = vec![0 as u8; cnt * thread * CHUNK_SIZE as usize];
        let res = sfs_read(
            fd,
            buf.as_mut_ptr() as *mut i8,
            cnt as i64 * thread as i64 * CHUNK_SIZE as i64,
        );
        if res <= 0 {
            println!("read error ...");
            return;
        } else {
            println!("{} bytes read", res);
        }
    }
    #[test]
    #[allow(unused_must_use)]
    pub fn test_continue() {
        let path = "/file1\0".to_string();
        let fd = sfs_open(
            path.as_str().as_ptr() as *const c_char,
            S_IFREG,
            O_CREAT | O_RDWR,
        );
        for i in 0..3 {
            //println!("file {} opened on thread {} ...", fd, i);
            let cnt = 500;
            let data = vec!['a' as i8; cnt * CHUNK_SIZE as usize];
            let res = sfs_write(fd, data.as_ptr() as *mut i8, data.len() as i64);
            if res <= 0 {
                println!("write error on task {} ...", i);
                return;
            } else {
                println!("{} bytes written on task {} ...", res, i);
            }
        }
    }
}
