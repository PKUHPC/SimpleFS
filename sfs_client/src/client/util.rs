use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use errno::{set_errno, Errno};
use libc::makedev;

use crate::client::context::StaticContext;
#[allow(unused_imports)]
use sfs_global::global::{metadata::Metadata, network::config::CHUNK_SIZE};

use super::{network::forward_msg, syscall::stat};

pub fn get_metadata(path: &String, _follow_link: bool) -> Result<Metadata, i32> {
    let md_res = forward_msg::forward_stat(path);
    if let Err(e) = md_res {
        set_errno(Errno(e));
        return Err(e);
    }
    return Ok(Metadata::deserialize(&md_res.unwrap()));
}
pub fn metadata_to_stat(path: &String, md: Metadata, attr: *mut stat) -> i32 {
    unsafe { (*attr).st_dev = makedev(0, 0) };
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    unsafe {
        (*attr).st_ino = hasher.finish();
        (*attr).st_nlink = 1;
        (*attr).st_uid = StaticContext::get_instance().get_fsconfig().uid;
        (*attr).st_gid = StaticContext::get_instance().get_fsconfig().gid;
        (*attr).st_rdev = 0;
        (*attr).st_blksize = CHUNK_SIZE as i64;
        (*attr).st_blocks = 0;

        (*attr).st_atim.tv_sec = 0;
        (*attr).st_atim.tv_nsec = 0;
        (*attr).st_ctim.tv_sec = 0;
        (*attr).st_ctim.tv_nsec = 0;
        (*attr).st_mtim.tv_sec = 0;
        (*attr).st_mtim.tv_nsec = 0;

        (*attr).st_mode = md.get_mode();
        (*attr).st_size = md.get_size();
        if StaticContext::get_instance().get_fsconfig().atime_state {
            (*attr).st_atim.tv_sec = md.get_access_time();
        }
        if StaticContext::get_instance().get_fsconfig().ctime_state {
            (*attr).st_ctim.tv_sec = md.get_change_time();
        }
        if StaticContext::get_instance().get_fsconfig().ctime_state {
            (*attr).st_ctim.tv_sec = md.get_modify_time();
        }
        if StaticContext::get_instance().get_fsconfig().link_cnt_state {
            (*attr).st_nlink = md.get_link_count();
        }
        if StaticContext::get_instance().get_fsconfig().blocks_state {
            (*attr).st_blocks = md.get_blocks();
        }
    }
    return 0;
}
