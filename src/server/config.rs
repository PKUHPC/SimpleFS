use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct ServerConfig{
    pub mountdir: String,
    pub rootdir: String,
    pub metadir: String,
    pub hosts_file : String,
    pub listen: String
}

pub static chunksize: i64 = 524288;