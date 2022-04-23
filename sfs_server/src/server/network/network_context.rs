use std::{
    fs::OpenOptions,
    io::{BufRead, BufReader, Error},
    path::Path,
    sync::Arc,
};

use grpcio::{ChannelBuilder, Environment};
use lazy_static::*;
use regex::Regex;
use sfs_global::global::{
    distributor::SimpleHashDistributor,
    endpoint::SFSEndpoint,
    fsconfig::{ENABLE_OUTPUT, HOSTFILE_PATH},
    util::env_util::{get_hostname, get_var},
};
use sfs_rpc::proto::server_grpc::SfsHandleClient;
use tokio::runtime::{Builder, Runtime};

use crate::{error_msg::error_msg, server::filesystem::storage_context::StorageContext};

fn load_host_file(path: &String) -> Result<Vec<(String, String)>, Error> {
    let mut hosts: Vec<(String, String)> = Vec::new();

    let f = OpenOptions::new().read(true).open(Path::new(&path))?;
    let reader = BufReader::new(f);
    for res in reader.lines() {
        let line = res?;
        let pattern = Regex::new("^(\\S+)\\s+(\\S+)$").unwrap();
        if !pattern.is_match(line.as_str()) {
            error_msg(
                "client::load_host_file".to_string(),
                format!("invalid host line '{}' detected", line),
            );
            continue;
        }
        for matched in pattern.captures_iter(line.as_str()) {
            hosts.push((matched[1].to_string(), matched[2].to_string()));
        }
    }
    if hosts.len() == 0 {
        error_msg(
            "client::load_host_file".to_string(),
            "no valid host".to_string(),
        );
        return Err(Error::new(std::io::ErrorKind::NotFound, "no valid host"));
    }
    return Ok(hosts);
}
// no connect actually
fn connect_hosts(hosts: &mut Vec<(String, String)>, context: &mut NetworkContext) -> bool {
    let local_hostname = get_hostname(true);
    if ENABLE_OUTPUT {
        println!("localhost name: {}", local_hostname);
    }
    let mut local_host_found = false;
    let mut addrs = vec![SFSEndpoint::new(); hosts.len()];
    let mut clients = Vec::new();
    let host_id: Vec<u64> = (0..(hosts.len() as u64)).collect();

    for id in host_id {
        let hostname = &hosts.get(id as usize).unwrap().0;
        let uri = &hosts.get(id as usize).unwrap().1;

        let endp = SFSEndpoint { addr: uri.clone() };
        let env = Arc::new(Environment::new(12));
        let channel = ChannelBuilder::new(env).connect(&format!("{}:{}", endp.addr, 8082));
        let client = SfsHandleClient::new(channel);
        addrs[id as usize] = endp;
        clients.push(client);
        if !local_host_found && hostname.eq(&local_hostname) {
            context.set_local_host_id(id);
            local_host_found = true;
        }
    }

    if !local_host_found {
        context.set_local_host_id(0);
    }

    context.set_hosts(addrs);
    context.set_clients(clients);
    return true;
}
fn read_host_file() -> Vec<(String, String)> {
    let hostfile = get_var("HOST_FILE".to_string(), HOSTFILE_PATH.to_string().clone());
    let load_res = load_host_file(&hostfile);
    if let Err(_e) = load_res {
        error_msg(
            "client::read_host_file".to_string(),
            "fail to load host file".to_string(),
        );
        return Vec::new();
    }
    let hosts = load_res.unwrap();
    return hosts;
}
pub struct NetworkContext {
    self_addr: String,
    hosts_: Vec<SFSEndpoint>,
    clients_: Vec<SfsHandleClient>,
    distributor_: Arc<SimpleHashDistributor>,
    local_host_id: u64,
    runtime_: Runtime,
}
lazy_static! {
    static ref NTC: NetworkContext = init_network();
}
fn init_network() -> NetworkContext {
    let mut hosts = read_host_file();

    let mut context = NetworkContext::new();
    if ENABLE_OUTPUT {
        println!("found hosts: {:?}", hosts);
    }
    if !connect_hosts(&mut hosts, &mut context) {
        return context;
    }

    let host_id = context.get_local_host_id();
    let host_len = context.get_hosts().len() as u64;
    let distributor = SimpleHashDistributor::new(host_id, host_len);
    context.set_distributor(distributor);
    context.set_self_addr(StorageContext::get_instance().get_bind_addr().clone());

    return context;
}
impl NetworkContext {
    pub fn get_instance() -> &'static NetworkContext {
        &NTC
    }
    pub fn new() -> NetworkContext {
        NetworkContext {
            self_addr: "".to_string(),
            hosts_: Vec::new(),
            clients_: Vec::new(),
            distributor_: Arc::new(SimpleHashDistributor::init()),
            local_host_id: 0,
            runtime_: Builder::new_multi_thread()
                .worker_threads(12)
                .thread_stack_size(3 * 1024 * 1024)
                .build()
                .unwrap(),
        }
    }
    pub fn get_self_addr(&self) -> &String {
        &self.self_addr
    }
    pub fn set_self_addr(&mut self, addr: String) {
        self.self_addr = addr;
    }
    pub fn set_distributor(&mut self, d: SimpleHashDistributor) {
        self.distributor_ = Arc::new(d);
    }
    pub fn get_distributor(&self) -> Arc<SimpleHashDistributor> {
        Arc::clone(&self.distributor_)
    }
    pub fn get_hosts(&self) -> &Vec<SFSEndpoint> {
        &self.hosts_
    }
    pub fn set_hosts(&mut self, hosts: Vec<SFSEndpoint>) {
        self.hosts_ = hosts;
    }
    pub fn clear_hosts(&mut self) {
        self.hosts_ = Vec::new();
    }
    pub fn get_clients(&self) -> &Vec<SfsHandleClient> {
        &self.clients_
    }
    pub fn set_clients(&mut self, clients: Vec<SfsHandleClient>) {
        self.clients_ = clients;
    }
    pub fn set_local_host_id(&mut self, host_id: u64) {
        self.local_host_id = host_id;
    }
    pub fn get_local_host_id(&self) -> u64 {
        self.local_host_id.clone()
    }
    pub fn get_runtime(&self) -> &Runtime {
        &self.runtime_
    }
}
