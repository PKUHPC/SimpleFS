use core::time;
use std::{io::{Error, BufReader, BufRead}, fs::OpenOptions, path::Path, net::TcpStream, thread};

use rand::{thread_rng, Rng};
use rand::seq::SliceRandom;
use regex::Regex;

use crate::{global::{util::env_util::get_var, error_msg::error_msg, distributor::SimpleHashDistributor, fsconfig::hostfile_path}, client::{context::ClientContext}};

use super::{util::get_hostname, endpoint::SFSEndpoint, network::{network_service::NetworkService, forward_msg::forward_get_fs_config}};

fn extract_protocol(uri: &String){
    
}
fn load_host_file(path: &String) -> Result<Vec<(String, String)>, Error>{
    let mut hosts: Vec<(String, String)> = Vec::new();

    let f = OpenOptions::new().read(true).open(Path::new(&path))?;
    let reader = BufReader::new(f);
    for res in reader.lines(){
        let line = res?;
        let pattern = Regex::new("^(\\S+)\\s+(\\S+)$").unwrap();
        if !pattern.is_match(line.as_str()){
            error_msg("client::load_host_file".to_string(), format!("invalid host line '{}' detected", line));
            continue;
        }
        for matched in pattern.captures_iter(line.as_str()){
            hosts.push((matched[1].to_string(), matched[2].to_string()));
        }
    }
    if hosts.len() == 0{
        error_msg("client::load_host_file".to_string(), "no valid host".to_string());
        return Err(Error::new(std::io::ErrorKind::NotFound, "no valid host"));
    }
    extract_protocol(&hosts[0].1);
    return Ok(hosts);
}
fn lookup_endpoint(uri: &String, max_retries: i32, host_id: u64) -> Result<SFSEndpoint, Error>{
    let endp = SFSEndpoint{ addr: uri.clone() };
    for i in 0..max_retries{
        if let Ok(post_res) = NetworkService::post::<u64>(&endp, host_id, crate::global::network::post::PostOption::Lookup){
            println!("connected: '{}'", uri);
            return Ok(endp);
        }
        else{
            error_msg("client::init::lookup_endpoint".to_string(), format!("fail to connect '{}', trying {}/{}", uri, i + 1, max_retries));
            thread::sleep(time::Duration::from_millis(5));
        }
    }
    Err(Error::new(std::io::ErrorKind::ConnectionAborted, "fail to connect to target host"))
}
fn connect_hosts(mut hosts: &mut Vec<(String, String)>) -> bool{
    let local_hostname = get_hostname(true);
    println!("localhost name: {}", local_hostname);
    let mut local_host_found = false;
    let mut addrs = vec![SFSEndpoint::new(); hosts.len()];
    let mut host_id: Vec<u64> = (0..(hosts.len() as u64)).collect();

    let mut rng = rand::thread_rng();
    host_id.shuffle(&mut rng);

    for id in host_id{
        let hostname = &hosts.get(id as usize).unwrap().0;
        let uri = &hosts.get(id as usize).unwrap().1;

        let lookup = lookup_endpoint(uri, 3, id);
        if let Err(e) = lookup{
            error_msg("client::init::connect_hosts".to_string(), format!("can not reach host '{}' with '{}'", hostname, uri));
            return false;
        }
        else{
            let endp = lookup.unwrap();
            addrs[id as usize] = endp;
        }
        if !local_host_found && hostname.eq(&local_hostname){
            ClientContext::get_instance().set_local_host_id(id);
            local_host_found = true;
        }
    }

    if !local_host_found{
        ClientContext::get_instance().set_local_host_id(0);
    }
    
    ClientContext::get_instance().set_hosts(addrs);
    return true;
}
fn read_host_file() -> Vec<(String, String)>{
    let hostfile = get_var("HOST_FILE".to_string(), hostfile_path.to_string().clone());
    let mut hosts: Vec<(String, String)> = Vec::new();
    let load_res = load_host_file(&hostfile);
    if let Err(e) = load_res{
        error_msg("client::read_host_file".to_string(), "fail to load host file".to_string());
        return Vec::new();
    }
    hosts = load_res.unwrap();
    return hosts;
}
pub fn init_environment(){
    let mut hosts = read_host_file();

    println!("found hosts: {:?}", hosts);
    if !connect_hosts(&mut hosts){
        return;
    }

    let host_id = ClientContext::get_instance().get_local_host_id();
    let host_len =ClientContext::get_instance().get_hosts().len() as u64; 
    let distributor = SimpleHashDistributor::new(host_id, host_len);
    ClientContext::get_instance().set_distributor(distributor);

    if !forward_get_fs_config(){
        error_msg("client::client_init".to_string(), "fail to fetch fs config".to_string());
    }
}