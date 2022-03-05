use std::{collections::HashMap, sync::Arc};

pub trait Distributor{
    fn localhost(&self, ) -> u64;
    fn locate_data(&self, path: &String, chunk_id: u64) -> u64;
    fn locate_file_metadata(&self, path: &String) -> u64;
    fn locate_dir_metadata(&self, path: &String) -> Arc<Vec<u64>>;
}

pub struct SimpleHashDistributor{
    localhost_: u64,
    hosts_size_: u64,
    all_hosts_: Arc<Vec<u64>>,
    str_hash_: HashMap<String, u64>
}
impl Distributor for SimpleHashDistributor{
    fn localhost(&self, ) -> u64 {
        self.localhost_
    }

    fn locate_data(&self, path: &String, chunk_id: u64) -> u64 {
        self.str_hash_.get(&(path.clone() + &chunk_id.to_string())).unwrap() % self.hosts_size_
    }

    fn locate_file_metadata(&self, path: &String) -> u64 {
        self.str_hash_.get(path).unwrap() % self.hosts_size_
    }

    fn locate_dir_metadata(&self, path: &String) -> Arc<Vec<u64>> {
        Arc::clone(&self.all_hosts_)
    }
}
impl SimpleHashDistributor{ 
    pub fn init() -> SimpleHashDistributor{
        SimpleHashDistributor{
            localhost_: 0,
            hosts_size_: 0,
            all_hosts_: Arc::new(Vec::new()),
            str_hash_: HashMap::new()
        }
    }
}

pub struct LocalOnlyDistributor{
    localhost_: u64
}
impl Distributor for LocalOnlyDistributor{
    fn localhost(&self, ) -> u64 {
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

pub struct ForwardDistributor{
    fwd_host_: u64, 
    hosts_size_: u64,
    all_hosts_: Arc<Vec<u64>>,
    str_hash_: HashMap<String, u64>
}
impl Distributor for ForwardDistributor{
    fn localhost(&self, ) -> u64 {
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