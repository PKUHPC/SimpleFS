#![allow(dead_code)]
pub mod global;
pub mod client;

#[no_mangle]
pub extern "C" fn hello_c(n: i32) -> i32{
    println!("hello c, here is rust");
    n + 1
}
#[cfg(test)]
mod tests {

    use libc::{c_char, O_RDWR, O_CREAT, S_IFREG, S_IFDIR, SEEK_SET, dirent};

    use crate::{global::{distributor::SimpleHashDistributor}, client::{endpoint::SFSEndpoint, context::ClientContext, syscall::{sfs_open, sfs_create, sfs_read, sfs_lseek, sfs_opendir, sfs_write, sfs_remove, sfs_truncate, sfs_stat, stat, sfs_dup, sfs_pwrite, sfs_pread, sfs_rmdir, sfs_getdents, sfs_dup2, internel_truncate}, init::init_environment}};

    #[test]
    pub fn test0(){
        init_environment();
    }
    #[test]
    pub fn test1(){
        let s = "hello, here is the test data of sfs small-data local-host open/read/write test";

        init_environment();

        let path = "/sfs/test/create_dir/file1\0".to_string();
        let path1 = "/sfs\0".to_string();
        let path2 = "/sfs/test\0".to_string();
        let path3 = "/sfs/test/create_dir\0".to_string();
        let res1 = sfs_create(path1.as_ptr() as * const i8, S_IFDIR);
        let res2 = sfs_create(path2.as_ptr() as * const i8, S_IFDIR);
        let res3 = sfs_create(path3.as_ptr() as * const i8, S_IFDIR);
        if res1 != 0 || res2 != 0 || res3 != 0{
            println!("create dir error ...");
            return;
        } 
        //let res = sfs_create(path.as_str().as_ptr() as * const c_char, S_IFDIR);
        let fd = sfs_open(path.as_str().as_ptr() as * const c_char, S_IFREG, O_CREAT | O_RDWR);
        if fd <= 0{
            println!("open error ...");
            return;
        }
        println!("open result: {}", fd);
        println!("ofm length: {}", ClientContext::get_instance().get_ofm().lock().unwrap().get_length());

        //let path = "/sfs/test/async_write/a".to_string();
        //let res = sfs_open(path.as_str().as_ptr() as * const c_char, S_IFREG, O_CREAT | O_RDONLY);
        //println!("open result: {}", res);
        //println!("ofm length: {}", ClientContext::get_instance().get_ofm().lock().unwrap().get_length());
        let res = sfs_write(fd, s.as_ptr() as * mut i8, s.len() as i64);
        if res <= 0{
            println!("write error ...");
            return;
        }
        else{
            println!("{} bytes written ...", res);
        }

        sfs_lseek(fd, 13, SEEK_SET);
        let mut buf = [0 as u8; 100];
        let res = sfs_read(fd, buf.as_mut_ptr() as * mut i8, 100);
        if res <= 0{
            println!("read error ...");
            return;
        }
        else{
            println!("read: {}", String::from_utf8(buf.to_vec()).unwrap());
        }
    }
    #[test]
    pub fn test2(){
        let s = "hello, here is the test data of sfs small-data local-host create/opendir test".to_string();

        init_environment();

        let path = "/sfs/test/create_dir/file1\0".to_string();
        let path1 = "/sfs\0".to_string();
        let path2 = "/sfs/test\0".to_string();
        let path3 = "/sfs/test/create_dir\0".to_string();
        let res1 = sfs_create(path1.as_ptr() as * const i8, S_IFDIR);
        let res2 = sfs_create(path2.as_ptr() as * const i8, S_IFDIR);
        let res3 = sfs_create(path3.as_ptr() as * const i8, S_IFDIR);

        let fd = sfs_open(path.as_str().as_ptr() as * const c_char, S_IFREG, O_CREAT | O_RDWR);
        let file_path1 = "/sfs/test/create_dir/file2\0".to_string();
        let fd = sfs_open(file_path1.as_str().as_ptr() as * const c_char, S_IFREG, O_CREAT | O_RDWR);

        let dir_path1 = "/sfs/test/create_dir\0".to_string();
        let dir_path2 = "/sfs/test/open_dir\0".to_string();
        let dir_path3 = "/sfs/test\0".to_string();

        let fd = sfs_opendir(dir_path1.as_str().as_ptr() as * const c_char);
        println!("open dir result: {}", fd);
        println!("ofm length: {}", ClientContext::get_instance().get_ofm().lock().unwrap().get_length());
        println!("dirents: {:?}\n", (*ClientContext::get_instance().get_ofm().lock().unwrap().get(fd).unwrap().lock().unwrap()).entries_);

        let res = sfs_create(dir_path2.as_ptr() as * const i8, S_IFDIR);
        let file_path2 = "/sfs/test/file1\0".to_string();
        let fd = sfs_open(file_path2.as_str().as_ptr() as * const c_char, S_IFREG, O_CREAT | O_RDWR);

        let fd = sfs_opendir(dir_path3.as_str().as_ptr() as * const c_char);
        println!("open dir result: {}", fd);
        println!("ofm length: {}", ClientContext::get_instance().get_ofm().lock().unwrap().get_length());
        println!("dirents: {:?}", (*ClientContext::get_instance().get_ofm().lock().unwrap().get(fd).unwrap().lock().unwrap()).entries_);
    }
    #[test]
    pub fn test3(){
        let data = "hello, here is the test data of sfs small-data local-host remove test";

        init_environment();


        let dpath_sfs = "/sfs\0".to_string();
        let cres = sfs_create(dpath_sfs.as_ptr() as * const i8, S_IFDIR);

        let fpath_file1 = "/sfs/file1\0".to_string();
        let fd = sfs_open(fpath_file1.as_str().as_ptr() as * const c_char, S_IFREG, O_CREAT | O_RDWR);

        let len = data.len() as i64; 
        let wres = sfs_write(fd, data.as_ptr() as * mut i8, len);

        sfs_lseek(fd, 0, SEEK_SET);
        let mut buf = vec![0 as u8; len as usize];
        let res = sfs_read(fd, buf.as_ptr() as * mut i8, len);
        if res <= 0{
            println!("read error ...");
            return;
        }
        else{
            println!("read: {}", String::from_utf8(buf).unwrap());
        }

        let fd = sfs_opendir(dpath_sfs.as_str().as_ptr() as * const c_char);
        println!("dirents of {}: {:?}", dpath_sfs, (*ClientContext::get_instance().get_ofm().lock().unwrap().get(fd).unwrap().lock().unwrap()).entries_);
        
        sfs_remove(fpath_file1.as_str().as_ptr() as * const c_char);
        let fd = sfs_opendir(dpath_sfs.as_str().as_ptr() as * const c_char);
        println!("dirents of {}: {:?}", dpath_sfs, (*ClientContext::get_instance().get_ofm().lock().unwrap().get(fd).unwrap().lock().unwrap()).entries_);
        
        sfs_remove(dpath_sfs.as_str().as_ptr() as * const c_char);
        let fd = sfs_opendir(dpath_sfs.as_str().as_ptr() as * const c_char);
        if fd != -1{
            println!("error remove dir ...");
        }
    }
    #[test]
    pub fn test4(){
        let data = "hello, here is the test data of sfs small-data local-host truncate test";

        init_environment();


        let dpath_sfs = "/sfs\0".to_string();
        let cres = sfs_create(dpath_sfs.as_ptr() as * const i8, S_IFDIR);

        let fpath_file1 = "/sfs/file1\0".to_string();
        let fd = sfs_open(fpath_file1.as_str().as_ptr() as * const c_char, S_IFREG, O_CREAT | O_RDWR);

        let len = data.len() as i64; 
        let wres = sfs_write(fd, data.as_ptr() as * mut i8, len);

        sfs_lseek(fd, 0, SEEK_SET);
        let mut buf = vec![0 as u8; len as usize];
        let res = sfs_read(fd, buf.as_ptr() as * mut i8, len);
        if res <= 0{
            println!("read error ...");
            return;
        }
        else{
            println!("read: {}", String::from_utf8(buf).unwrap());
        }

        let tres = internel_truncate(fpath_file1.as_str().as_ptr() as * const c_char, len, 13);
        if tres != 0{
            println!("truncate error ...");
            return;
        }

        sfs_lseek(fd, 0, SEEK_SET);
        let mut buf = vec![0 as u8; len as usize];
        let res = sfs_read(fd, buf.as_ptr() as * mut i8, len);
        if res <= 0{
            println!("read error ...");
            return;
        }
        else{
            println!("read: {}", String::from_utf8(buf).unwrap());
        }
    }
    #[test]
    pub fn test5(){
        let data = "hello, here is the test data of sfs small-data local-host stat test";

        init_environment();


        let dpath_sfs = "/sfs\0".to_string();
        let cres = sfs_create(dpath_sfs.as_ptr() as * const i8, S_IFDIR);

        let fpath_file1 = "/sfs/file1\0".to_string();
        let fd = sfs_open(fpath_file1.as_str().as_ptr() as * const c_char, S_IFREG, O_CREAT | O_RDWR);

        let len = data.len() as i64; 
        let wres = sfs_write(fd, data.as_ptr() as * mut i8, len);


        let mut stat = stat{
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
            st_atime: 0,
            st_atime_nsec: 0,
            st_mtime: 0,
            st_mtime_nsec: 0,
            st_ctime: 0,
            st_ctime_nsec: 0,
            __unused: [0; 3],
        };
        let res = sfs_stat(fpath_file1.as_str().as_ptr() as * const c_char, &mut stat as *mut stat, false);
        if res < 0{
            println!("stat error ...");
            return;
        }
        else{
            println!("stat: {:?}", stat);
        }

        let tres = internel_truncate(fpath_file1.as_str().as_ptr() as * const c_char, len, 13);
        if tres != 0{
            println!("truncate error ...");
            return;
        }

        let mut stat = stat{
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
            st_atime: 0,
            st_atime_nsec: 0,
            st_mtime: 0,
            st_mtime_nsec: 0,
            st_ctime: 0,
            st_ctime_nsec: 0,
            __unused: [0; 3],
        };
        let res = sfs_stat(fpath_file1.as_str().as_ptr() as * const c_char, &mut stat as *mut stat, false);
        if res < 0{
            println!("stat error ...");
            return;
        }
        else{
            println!("stat: {:?}", stat);
        }
    }

    #[test]
    pub fn test6(){
        let data = "hello, here is the test data of sfs small-data local-host dup test";

        init_environment();


        let dpath_sfs = "/sfs\0".to_string();
        let cres = sfs_create(dpath_sfs.as_ptr() as * const i8, S_IFDIR);

        let fpath_file1 = "/sfs/file1\0".to_string();
        let fd = sfs_open(fpath_file1.as_str().as_ptr() as * const c_char, S_IFREG, O_CREAT | O_RDWR);

        let fd2 = sfs_dup(fd);
        if fd2 <= 0 {
            println!("dup error ...");
            return;
        }
        println!("dup {} to {}", fd, fd2);

        let wres = sfs_write(fd2, data.as_ptr() as * mut i8, data.len() as i64);
        sfs_lseek(fd, 0, SEEK_SET);
        let mut buf = vec![0 as u8; data.len()];
        let res = sfs_read(fd, buf.as_ptr() as * mut i8, data.len() as i64);
        if res <= 0{
            println!("read error ...");
            return;
        }
        else{
            println!("read from origin fd: {}", String::from_utf8(buf).unwrap());
        }

        sfs_lseek(fd2, 0, SEEK_SET);
        let mut buf = vec![0 as u8; data.len()];
        let res = sfs_read(fd2, buf.as_ptr() as * mut i8, data.len() as i64);
        if res <= 0{
            println!("read error ...");
            return;
        }
        else{
            println!("read from dupped fd: {}", String::from_utf8(buf).unwrap());
        }
    }
    

    #[test]
    pub fn test7(){
        let data = "hello, here is the test data of sfs small-data local-host pwrite/pread test";

        init_environment();


        let dpath_sfs = "/sfs\0".to_string();
        let cres = sfs_create(dpath_sfs.as_ptr() as * const i8, S_IFDIR);

        let fpath_file1 = "/sfs/file1\0".to_string();
        let fd = sfs_open(fpath_file1.as_str().as_ptr() as * const c_char, S_IFREG, O_CREAT | O_RDWR);

        let wres = sfs_write(fd, data.as_ptr() as * mut i8, data.len() as i64);
        sfs_lseek(fd, 0, SEEK_SET);
        let wres = sfs_pwrite(fd, data.as_ptr() as * mut i8, data.len() as i64, 9);
        sfs_lseek(fd, 0, SEEK_SET);
        let mut buf = vec![0 as u8; 100];
        let res = sfs_pread(fd, buf.as_ptr() as * mut i8, 200, 7);
        if res <= 0{
            println!("read error ...");
            return;
        }
        else{
            println!("read: {}", String::from_utf8(buf).unwrap());
        }
    }

    #[test]
    pub fn test8(){
        let data = "hello, here is the test data of sfs small-data local-host rmdir test";

        init_environment();


        let dpath_sfs = "/sfs\0".to_string();
        let cres = sfs_create(dpath_sfs.as_ptr() as * const i8, S_IFDIR);

        let fpath_file1 = "/sfs/file1\0".to_string();
        let fd = sfs_open(fpath_file1.as_str().as_ptr() as * const c_char, S_IFREG, O_CREAT | O_RDWR);

        sfs_rmdir(dpath_sfs.as_ptr() as * const i8);
        sfs_remove(fpath_file1.as_ptr() as * const i8);
        sfs_rmdir(dpath_sfs.as_ptr() as * const i8);
    }

    #[test]
    pub fn test9(){
        let data = "hello, here is the test data of sfs small-data local-host getdents test";

        init_environment();


        let dpath_sfs = "/sfs\0".to_string();
        sfs_create(dpath_sfs.as_ptr() as * const i8, S_IFDIR);

        let fpath_file1 = "/sfs/file1\0".to_string();
        let fpath_file2 = "/sfs/file2\0".to_string();
        let fpath_file3 = "/sfs/file3\0".to_string();
        let dpath_dir1 = "/sfs/dir1\0".to_string();
        let dpath_dir2 = "/sfs/dir2\0".to_string();
        sfs_open(fpath_file1.as_str().as_ptr() as * const c_char, S_IFREG, O_CREAT | O_RDWR);
        sfs_open(fpath_file2.as_str().as_ptr() as * const c_char, S_IFREG, O_CREAT | O_RDWR);
        sfs_open(fpath_file3.as_str().as_ptr() as * const c_char, S_IFREG, O_CREAT | O_RDWR);
        sfs_create(dpath_dir1.as_ptr() as * const i8, S_IFDIR);
        sfs_create(dpath_dir2.as_ptr() as * const i8, S_IFDIR);

        let new_dirent = dirent{
            d_ino: 0,
            d_off: 0,
            d_reclen: 0,
            d_type: 0,
            d_name: [0; 256],
        };
        let dirents = [new_dirent; 20];
        let fd = sfs_opendir(dpath_sfs.as_str().as_ptr() as * const c_char);
        println!("dirents of {}: {:?}", dpath_sfs, (*ClientContext::get_instance().get_ofm().lock().unwrap().get(fd).unwrap().lock().unwrap()).entries_);
        sfs_lseek(fd, 0, SEEK_SET);
        sfs_getdents(fd, dirents.as_ptr() as *mut dirent, 200);

        let mut dirent_ptr = dirents.as_ptr() as *const c_char;
        for i in 0..5{
            let dirent = unsafe{*(dirent_ptr as *const dirent)};
            let total = dirent.d_reclen;
            dirent_ptr = unsafe{dirent_ptr.offset(total as isize)};

            let name_size = total - 19;
            let mut c_vec:Vec<u8> = Vec::new();

            let mut len = 0;
            for c in dirent.d_name{
                c_vec.push(c as u8);
                len += 1;
                if c == 0{
                    break;
                }
            }
            c_vec = c_vec[0..len].to_vec();
            println!("{:?}: {}", dirent, String::from_utf8(c_vec).unwrap());
        }
    }

    #[test]
    pub fn test10(){
        let data = "hello, here is the test data of sfs small-data local-host dup2 test";

        init_environment();


        let dpath_sfs = "/sfs\0".to_string();
        let cres = sfs_create(dpath_sfs.as_ptr() as * const i8, S_IFDIR);

        let fpath_file1 = "/sfs/file1\0".to_string();
        let fd = sfs_open(fpath_file1.as_str().as_ptr() as * const c_char, S_IFREG, O_CREAT | O_RDWR);
        let fd2 = 100010;
        let fd3 = sfs_dup2(fd, fd2);
        if fd2 != fd3 {
            println!("dup2 error ...");
            return;
        }
        println!("dup2 {} to {}", fd, fd3);

        let wres = sfs_write(fd2, data.as_ptr() as * mut i8, data.len() as i64);
        sfs_lseek(fd, 0, SEEK_SET);
        let mut buf = vec![0 as u8; data.len()];
        let res = sfs_read(fd, buf.as_ptr() as * mut i8, data.len() as i64);
        if res <= 0{
            println!("read error ...");
            return;
        }
        else{
            println!("read from origin fd: {}", String::from_utf8(buf).unwrap());
        }

        sfs_lseek(fd2, 0, SEEK_SET);
        let mut buf = vec![0 as u8; data.len()];
        let res = sfs_read(fd2, buf.as_ptr() as * mut i8, data.len() as i64);
        if res <= 0{
            println!("read error ...");
            return;
        }
        else{
            println!("read from dupped fd: {}", String::from_utf8(buf).unwrap());
        }
    }
}
