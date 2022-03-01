pub struct SFSConfig{
    pub atime_state: bool,
    pub ctime_state: bool,
    pub mtime_state: bool,
    pub link_cnt_state: bool,
    pub blocks_state: bool,
    pub uid: u32,
    pub gid: u32,
    pub rootdir: String
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