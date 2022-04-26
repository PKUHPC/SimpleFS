use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub mountdir: String,
    pub rootdir: String,
    pub metadir: String,
    pub hosts_file: String,
    pub listen: String,
    pub output: bool
}
pub const STUFF_WITH_ROCKSDB: bool = true;
pub const IGNORE_IF_EXISTS: bool = true;
pub const TRUNCATE_DIRECTORY: bool = true;
