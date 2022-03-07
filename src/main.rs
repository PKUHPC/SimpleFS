pub mod handle;
pub mod task;
use std::{fs::{self}, io::{Read, Error}, path::Path, net::{TcpListener, TcpStream}, thread};
use sfs_lib::{global::network::post::PostOption::*, global::network::{forward_data::WriteData, config::CHUNK_SIZE}};
use sfs_lib::{server::{filesystem::storage_context::StorageContext, storage::metadata::db::MetadataDB, storage::data::chunk_storage::*}, global::network::post::Post};
use sha2::{Sha256, Digest};

fn handle_client(mut stream: TcpStream) -> Result<(), Error>{
    let mut buf = [0; 2048];
    loop {
        let bytes_read = stream.read(&mut buf)?;
        if bytes_read == 0 {
            break;
        }
    }
    let post: Post = serde_json::from_str(String::from_utf8(buf.to_vec()).unwrap().as_str()).expect("JSON was not well-formatted");
    match post.option {
        Stat => todo!(),
        Create => todo!(),
        Remove => todo!(),
        Write => {
            let write_data: WriteData = serde_json::from_str(&post.data).unwrap();
            //handle_write(stream, write_data);
        },
        Lookup => {
            let id: u64 = serde_json::from_str(&post.data).unwrap();
            StorageContext::get_instance().set_host_id(id);
        },
    }
    Ok(())
}

fn init_server(addr: &String){
    let bind_res = TcpListener::bind(addr);
    if let Err(e) = bind_res{
        return ;
    }
    let listener = bind_res.unwrap();
    let mut thread_vec: Vec<thread::JoinHandle<()>> = Vec::new();
    for stream in listener.incoming() {
        let stream = stream.expect("failed!");
        let handle = thread::spawn(move || {
            handle_client(stream)
        .unwrap_or_else(|error| eprintln!("{:?}", error));
        });

        thread_vec.push(handle);
    }

    for handle in thread_vec {
        handle.join().unwrap();
    }
}

fn init_environment(){
    let metadata_path = StorageContext::get_instance().get_metadir().clone() + &"/rocksdb".to_string();
    StorageContext::get_instance().set_mdb(MetadataDB::new(metadata_path).expect("fail to create metadata data base"));

    let chunk_storage_path = StorageContext::get_instance().get_rootdir().clone() + &"/data/chunks".to_string();
    fs::create_dir_all(Path::new(&chunk_storage_path)).expect("fail to create chunk storage");
    StorageContext::get_instance().set_storage(ChunkStorage::new(&chunk_storage_path, CHUNK_SIZE).expect("fail to create chunk storage"));

    init_server(StorageContext::get_instance().get_bind_addr());

    if !StorageContext::get_instance().get_hosts_file().len() == 0{
        todo!()
    }
}
pub fn main(){
    /* 
    let RPC_PROTOCOL: String = String::from("tcp");
    
    let mut json: Vec<u8> = Vec::new();
    let open_res =  fs::OpenOptions::new().read(true).open("config.json".to_string());
    if let Err(e) = open_res{
        error_msg("server_main".to_string(), "fail to open config file".to_string());
        return;
    }
    let mut f = open_res.unwrap();
    f.read_to_end(&mut json).expect("fail to read config file");
    let s = String::from_utf8(json.clone()).unwrap();
    let config: ServerConfig = serde_json::from_str(s.as_str()).expect("JSON was not well-formatted");

    fs::create_dir_all(Path::new(&config.mountdir)).expect("fail to create mount directory");
    StorageContext::get_instance().set_mountdir(fs::canonicalize(&config.mountdir).unwrap().to_str().unwrap().to_string());
    let root_dir = config.rootdir;
    let root_dirpath = root_dir + &std::process::id().to_string();
    fs::create_dir_all(Path::new(&root_dirpath)).expect("fail to create root directory");
    StorageContext::get_instance().set_rootdir(root_dirpath);
    StorageContext::get_instance().set_metadir(fs::canonicalize(&config.metadir).unwrap().to_str().unwrap().to_string());
    StorageContext::get_instance().set_hosts_file(config.hosts_file);
    StorageContext::get_instance().set_bind_addr(format!("{}://{}", RPC_PROTOCOL, config.listen));

    init_environment();
    */
    //init_server(&"192.168.230.137:8082".to_string());
    /*
    let s = "hello everyone, this is a sfs write test and will write a small data".to_string();
    let mut hosts = ClientContext::get_instance().get_hosts();
    hosts.lock().unwrap().push(SFSEndpoint{
        addr: "192.168.230.137:8082".to_string(),
    });
    println!("trying to write");
    forward_write(&"/sfs/test/write_chunk/a".to_string(), s.as_ptr() as * const i8, true, 0, s.len() as i64, s.len() as i64);
    */
    /*
    StorageContext::get_instance().set_storage(ChunkStorage{
        root_path_: "/home/dev/Desktop/storage".to_string(),
        chunk_size_: CHUNK_SIZE,
    });

    let data = String::from("{\"option\":\"Write\",\"data\":\"{\\\"path\\\":\\\"/sfs/test/write_chunk/a\\\",\\\"offset\\\":0,\\\"host_id\\\":0,\\\"host_size\\\":1,\\\"chunk_n\\\":1,\\\"chunk_start\\\":0,\\\"chunk_end\\\":0,\\\"total_chunk_size\\\":68,\\\"buffers\\\":\\\"hello everyone, this is a sfs write test and will write a small data\\\"}\"}");
    let post: Post = serde_json::from_str(&data).unwrap();
    let write_data: WriteData = serde_json::from_str(&post.data).unwrap();
    handle_write(write_data);
    */
    let mut hasher = Sha256::new();
    let s1 = "sjfbcakjbsca".to_string();
    let s2 = "sjfbcakjbsca".to_string();
    let s3 = "sjfbcakjbsca".to_string();
    let s4 = "sjfbcakjbsca".to_string();
    hasher.update(s1);
    println!("{}", hasher.finalize()[0]);

    let mut hasher = Sha256::new();
    hasher.update(s2);
    println!("{}", hasher.finalize()[0]);
}