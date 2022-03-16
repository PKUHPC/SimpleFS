use std::{hash::{Hash, Hasher}, collections::hash_map::DefaultHasher, io::Error};

use libc::{stat, makedev, gethostname};

use crate::{global::{metadata::Metadata, network::config::CHUNK_SIZE}, client::client_context::ClientContext };

use super::network::{forward_msg, self};

pub fn get_metadata(path: &String, follow_link: bool) -> Result<Metadata, Error>{
    let md_res = forward_msg::forward_stat(path);
    if let Err(e) = md_res{
        return Err(e);
    }
    return Ok(Metadata::deserialize(&md_res.unwrap()).unwrap());
}
pub fn get_hostname(short_hostname: bool) -> String{
    let hostname: [u8; 1024] = [0; 1024];
    let ret = unsafe {gethostname(hostname.as_ptr() as *mut i8, 1024)};
    if ret == 0{
        let mut hostname = String::from_utf8(hostname.to_vec()).unwrap();
        if !short_hostname{
            return hostname;
        }
        if let Some(pos) = hostname.find(&".".to_string()){
            hostname = hostname[0..pos].to_string();
        }
        return hostname;
    }
    else{
        return "".to_string();
    }
}
pub fn metadata_to_stat(path: &String, md: Metadata, attr: &mut stat) -> i32{
    unsafe{ attr.st_dev = makedev(0, 0) };
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    attr.st_ino = hasher.finish();
    attr.st_nlink = 1;
    attr.st_uid = ClientContext::get_instance().get_fsconfig().uid;
    attr.st_gid = ClientContext::get_instance().get_fsconfig().gid;
    attr.st_rdev = 0;
    attr.st_blksize = CHUNK_SIZE as i64;
    attr.st_blocks = 0;

    attr.st_atime = 0;
    attr.st_atime_nsec = 0;
    attr.st_ctime = 0;
    attr.st_ctime_nsec = 0;
    attr.st_mtime = 0;
    attr.st_mtime_nsec = 0;

    attr.st_mode = md.get_mode();
    attr.st_size = md.get_size();
    if ClientContext::get_instance().get_fsconfig().atime_state{
        attr.st_atime = md.get_access_time();
    }
    if ClientContext::get_instance().get_fsconfig().ctime_state{
        attr.st_ctime = md.get_change_time();
    }
    if ClientContext::get_instance().get_fsconfig().ctime_state{
        attr.st_ctime = md.get_modify_time();
    }
    if ClientContext::get_instance().get_fsconfig().link_cnt_state{
        attr.st_nlink = md.get_link_count();
    }
    if ClientContext::get_instance().get_fsconfig().blocks_state{
        attr.st_blocks = md.get_blocks();
    }
    return 0;
}