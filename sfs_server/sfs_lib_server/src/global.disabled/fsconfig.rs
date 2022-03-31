use serde::{Deserialize, Serialize};

pub static HOSTFILE_PATH: &str = "hostfile";
#[derive(Serialize, Deserialize, Debug)]
pub struct SFSConfig {
    pub atime_state: bool,
    pub ctime_state: bool,
    pub mtime_state: bool,
    pub link_cnt_state: bool,
    pub blocks_state: bool,
    pub uid: u32,
    pub gid: u32,
    pub rootdir: String,
    pub mountdir: String,
}
impl SFSConfig {
    pub fn new() -> SFSConfig {
        SFSConfig {
            atime_state: true,
            ctime_state: true,
            mtime_state: true,
            link_cnt_state: true,
            blocks_state: true,
            uid: 0,
            gid: 0,
            rootdir: "".to_string(),
            mountdir: "".to_string(),
        }
    }
}
impl Clone for SFSConfig {
    fn clone(&self) -> Self {
        Self {
            atime_state: self.atime_state.clone(),
            ctime_state: self.ctime_state.clone(),
            mtime_state: self.mtime_state.clone(),
            link_cnt_state: self.link_cnt_state.clone(),
            blocks_state: self.blocks_state.clone(),
            uid: self.uid.clone(),
            gid: self.gid.clone(),
            rootdir: self.rootdir.clone(),
            mountdir: self.mountdir.clone(),
        }
    }
}
pub static ZERO_BUF_BEFORE_READ: bool = false;
pub static CWD: &str = "SFS_CWD";
pub static ENABLE_OUTPUT: bool = true;