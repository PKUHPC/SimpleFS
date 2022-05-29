use core::time;
use std::{
    fs::OpenOptions,
    io::{BufRead, BufReader, Error, Read},
    path::Path,
    sync::Arc,
    thread,
};

use grpcio::{ChannelBuilder, Environment};
use rdma_sys::rdma_create_event_channel;
use regex::Regex;

use sfs_global::global::{
    distributor::SimpleHashDistributor,
    endpoint::SFSEndpoint,
    error_msg::error_msg,
    fsconfig::{ENABLE_OUTPUT, HOSTFILE_PATH},
    network::{
        config::RDMAConfig,
        post::{option2i, PostOption},
    },
    util::{
        env_util::{get_hostname, get_var},
        serde_util::serialize,
    },
};
use sfs_rpc::{post, proto::server_grpc::SfsHandleClient};

use super::{
    context::{DynamicContext, StaticContext},
    network::{forward_msg::forward_get_fs_config, rdmacm::process_cm_event},
};

fn extract_protocol(_uri: &String) {}
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
    extract_protocol(&hosts[0].1);
    return Ok(hosts);
}
fn lookup_endpoint(
    uri: &String,
    max_retries: i32,
    host_id: u64,
) -> Result<(SFSEndpoint, SfsHandleClient), Error> {
    let endp = SFSEndpoint { addr: uri.clone() };
    for i in 0..max_retries {
        let serialized_data = serialize(&host_id);
        let post = post(option2i(&PostOption::Lookup), serialized_data, vec![0; 0]);
        let env = Arc::new(Environment::new(4));
        let channel = ChannelBuilder::new(env)
            .max_receive_message_len(128 * 1024 * 1024)
            .max_send_message_len(128 * 1024 * 1024)
            .connect(&format!("{}:{}", endp.addr, 8082));
        let client = SfsHandleClient::new(channel);
        if let Ok(_post_res) = client.handle(&post) {
            if ENABLE_OUTPUT {
                println!("connected: '{}'", uri);
            }
            return Ok((endp, client));
        } else {
            error_msg(
                "client::init::lookup_endpoint".to_string(),
                format!(
                    "fail to connect '{}', trying {}/{}",
                    uri,
                    i + 1,
                    max_retries
                ),
            );
            thread::sleep(time::Duration::from_millis(5));
        }
    }
    Err(Error::new(
        std::io::ErrorKind::ConnectionAborted,
        "fail to connect to target host",
    ))
}
fn connect_hosts(hosts: &mut Vec<(String, String)>, context: &mut StaticContext) -> u64 {
    let local_hostname = get_hostname(true);
    if ENABLE_OUTPUT {
        println!("localhost name: {}", local_hostname);
    }
    let mut local_host_found = false;
    let mut addrs = Vec::new();
    let mut clients = Vec::new();
    let host_id: Vec<u64> = (0..(hosts.len() as u64)).collect();

    for id in host_id {
        let hostname = &hosts.get(id as usize).unwrap().0;
        let uri = &hosts.get(id as usize).unwrap().1;

        let lookup = lookup_endpoint(uri, 1, id);
        if let Err(_e) = lookup {
            error_msg(
                "client::init::connect_hosts".to_string(),
                format!("can not reach host '{}' with '{}'", hostname, uri),
            );
            return 0;
        } else {
            let res = lookup.unwrap();
            addrs.push(res.0);
            clients.push(res.1);
        }
        if !local_host_found && hostname.eq(&local_hostname) {
            context.set_local_host_id(id);
            local_host_found = true;
        }
    }

    if !local_host_found {
        context.set_local_host_id(0);
    }
    let len = addrs.len() as u64;
    context.set_hosts(addrs);
    context.set_clients(clients);
    return len;
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
pub fn init_environment() -> StaticContext {
    DynamicContext::get_instance();
    let mut hosts = read_host_file();

    let mut context = StaticContext::new();
    if ENABLE_OUTPUT {
        println!("found hosts: {:?}", hosts);
    }
    let host_len = connect_hosts(&mut hosts, &mut context);
    if host_len == 0 {
        return context;
    }
    let host_id = context.get_local_host_id();
    let distributor = SimpleHashDistributor::new(host_id, host_len);
    context.set_distributor(distributor);

    if !forward_get_fs_config(&mut context) {
        error_msg(
            "client::client_init".to_string(),
            "fail to fetch fs config".to_string(),
        );
    }
    let mut json: Vec<u8> = Vec::new();
    let mut f = std::fs::OpenOptions::new()
        .read(true)
        .open("config.json".to_string())
        .unwrap();

    f.read_to_end(&mut json).expect("fail to read config file");
    let s = String::from_utf8(json.clone()).unwrap();
    let config: RDMAConfig = serde_json::from_str(s.as_str()).expect("JSON was not well-formatted");
    context.rdma_addr = config.addr;

    context.init_flag = true;
    context.event_channel = unsafe{rdma_create_event_channel() as u64};
    let ec = context.event_channel;
    context.handle = Some(std::thread::spawn(move || {process_cm_event(ec)}));

    return context;
}
