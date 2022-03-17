#![allow(dead_code)]
pub mod global;
pub mod client;

#[cfg(test)]
mod tests {

    use libc::{c_char, O_RDWR, O_CREAT, S_IFREG, S_IFDIR, SEEK_SET};

    use crate::{global::{distributor::SimpleHashDistributor}, client::{client_endpoint::SFSEndpoint, client_context::ClientContext, client_syscall::{sfs_open, sfs_create, sfs_read, sfs_lseek, sfs_opendir, sfs_write}}};

    #[test]
    pub fn test1(){
        let s = "bybchuicbahbcashbadhasuhdadioada";

        let distributor = SimpleHashDistributor::new(1, 1);
        ClientContext::get_instance().set_distributor(distributor);
        
        let endp = SFSEndpoint{
            addr: "127.0.0.1".to_string(),
        };
        ClientContext::get_instance().set_hosts(vec![endp; 1]);


        let path = "/sfs/test/create_dir/file1".to_string();
        let path1 = "/sfs".to_string();
        let path2 = "/sfs/test".to_string();
        let path3 = "/sfs/test/create_dir".to_string();
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
        let mut buf = [0 as u8; 50];
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
        let s = "bybchuicbahbcashbadhasuhdadioada".to_string();

        let distributor = SimpleHashDistributor::new(1, 1);
        ClientContext::get_instance().set_distributor(distributor);
        
        let endp = SFSEndpoint{
            addr: "127.0.0.1".to_string(),
        };
        ClientContext::get_instance().set_hosts(vec![endp; 1]);

        let dir_path = "/sfs/test/create_dir".to_string();
        let fd = sfs_opendir(dir_path.as_str().as_ptr() as * const c_char);
        println!("open result: {}", fd);
        println!("ofm length: {}", ClientContext::get_instance().get_ofm().lock().unwrap().get_length());
        println!("dirents: {:?}", ClientContext::get_instance().get_ofm().lock().unwrap().get(fd).unwrap().lock().unwrap().getdent(0));
    }
}
