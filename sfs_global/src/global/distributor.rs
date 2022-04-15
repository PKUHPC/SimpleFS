use xxhash_rust::xxh3::xxh3_64;
use std::{collections::HashMap, sync::Arc};

pub trait Distributor {
    fn localhost(&self) -> u64;
    fn locate_data(&self, path: &String, chunk_id: u64) -> u64;
    fn locate_file_metadata(&self, path: &String) -> u64;
    fn locate_dir_metadata(&self, path: &String) -> Arc<Vec<u64>>;
}

#[derive(Debug)]
pub struct SimpleHashDistributor {
    pub localhost_: u64,
    pub hosts_size_: u64,
    pub all_hosts_: Arc<Vec<u64>>,
    //pub str_hash_: DefaultHasher
}
#[allow(unused_variables)]
impl Distributor for SimpleHashDistributor {
    fn localhost(&self) -> u64 {
        self.localhost_
    }

    fn locate_data(&self, path: &String, chunk_id: u64) -> u64 {
        if chunk_id == 0{
            return self.locate_file_metadata(path);
        }
        let s = path.clone() + &chunk_id.to_string();
        xxh3_64(s.as_bytes()) % self.hosts_size_
    }

    fn locate_file_metadata(&self, path: &String) -> u64 {
        xxh3_64(path.as_bytes()) % self.hosts_size_
    }

    fn locate_dir_metadata(&self, path: &String) -> Arc<Vec<u64>> {
        Arc::clone(&self.all_hosts_)
    }
}
impl SimpleHashDistributor {
    pub fn init() -> SimpleHashDistributor {
        SimpleHashDistributor {
            localhost_: 0,
            hosts_size_: 0,
            all_hosts_: Arc::new(Vec::new()),
            //str_hash_: DefaultHasher::new()
        }
    }
    pub fn new(host_id: u64, host_size: u64) -> SimpleHashDistributor {
        SimpleHashDistributor {
            localhost_: host_id,
            hosts_size_: host_size,
            all_hosts_: Arc::new((0..host_size).collect()),
            //str_hash_: DefaultHasher::new()
        }
    }
}

pub struct LocalOnlyDistributor {
    pub localhost_: u64,
}
impl LocalOnlyDistributor {
    pub fn new(host_id: u64) -> LocalOnlyDistributor {
        LocalOnlyDistributor {
            localhost_: host_id,
        }
    }
}
#[allow(unused_variables)]
impl Distributor for LocalOnlyDistributor {
    fn localhost(&self) -> u64 {
        self.localhost_
    }

    fn locate_data(&self, path: &String, chunk_id: u64) -> u64 {
        self.localhost_
    }

    fn locate_file_metadata(&self, path: &String) -> u64 {
        self.localhost_
    }

    fn locate_dir_metadata(&self, path: &String) -> Arc<Vec<u64>> {
        Arc::new(vec![self.localhost_])
    }
}

pub struct ForwardDistributor {
    pub fwd_host_: u64,
    pub hosts_size_: u64,
    pub all_hosts_: Arc<Vec<u64>>,
    pub str_hash_: HashMap<String, u64>,
}
#[allow(unused_variables)]
impl Distributor for ForwardDistributor {
    fn localhost(&self) -> u64 {
        self.fwd_host_
    }

    fn locate_data(&self, path: &String, chunk_id: u64) -> u64 {
        self.fwd_host_
    }

    fn locate_file_metadata(&self, path: &String) -> u64 {
        self.str_hash_.get(path).unwrap() % self.hosts_size_
    }

    fn locate_dir_metadata(&self, path: &String) -> Arc<Vec<u64>> {
        Arc::clone(&self.all_hosts_)
    }
}
