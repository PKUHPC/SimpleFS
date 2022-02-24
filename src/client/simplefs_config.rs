pub struct SFSConfig{
    atime_state: bool,
    ctime_state: bool,
    mtime_state: bool,
    link_cnt_state: bool,
    blocks_state: bool,
    uid: u32,
    gid: u32,
    rootdir: String
}
impl SFSConfig{
    pub fn init() -> SFSConfig{
        SFSConfig{
            atime_state: true,
            ctime_state: true,
            mtime_state: true,
            link_cnt_state: true,
            blocks_state: true,
            uid: 0,
            gid: 0,
            rootdir: "".to_string(),
        }
    }
}