use crate::client::context::StaticContext;
use errno::{set_errno, Errno};
use libc::{makedev, stat};
use sfs_global::global::distributor::Distributor;
#[allow(unused_imports)]
use sfs_global::global::{metadata::Metadata, network::config::CHUNK_SIZE};
use xxhash_rust::xxh3::xxh3_64;

use super::network::forward_msg;

pub fn get_metadata(path: &String, _follow_link: bool) -> Result<Metadata, i32> {
    let md_res = forward_msg::forward_stat(path);
    if let Err(e) = md_res {
        set_errno(Errno(e));
        return Err(e);
    }
    return Ok(Metadata::deserialize(&md_res.unwrap()));
}
// bias is used to make sure pass ctime check of pfind
// pfind ctime check will fail caused by non-syncing clock
pub static BIAS: i64 = 300;
pub fn metadata_to_stat(path: &String, md: Metadata, attr: *mut stat) -> i32 {
    unsafe { (*attr).st_dev = makedev(0, 0) };
    unsafe {
        (*attr).st_ino = xxh3_64(path.as_bytes());
        (*attr).st_nlink = 1;
        (*attr).st_uid = StaticContext::get_instance().get_fsconfig().uid;
        (*attr).st_gid = StaticContext::get_instance().get_fsconfig().gid;
        (*attr).st_rdev = StaticContext::get_instance()
            .get_distributor()
            .locate_file_metadata(path);
        (*attr).st_blksize = CHUNK_SIZE as i64;
        (*attr).st_blocks = md.get_size() / 512;

        (*attr).st_mode = md.get_mode();
        (*attr).st_size = md.get_size();
        (*attr).st_atime = md.get_access_time() + BIAS;
        (*attr).st_atime_nsec = md.get_access_time() + BIAS;
        (*attr).st_ctime = md.get_change_time() + BIAS;
        (*attr).st_ctime_nsec = md.get_change_time() + BIAS;
        (*attr).st_mtime = md.get_modify_time() + BIAS;
        (*attr).st_mtime_nsec = md.get_modify_time() + BIAS;
        if StaticContext::get_instance().get_fsconfig().link_cnt_state {
            (*attr).st_nlink = md.get_link_count();
        }
        if StaticContext::get_instance().get_fsconfig().blocks_state {
            (*attr).st_blocks = md.get_blocks();
        }
    }
    return 0;
}
