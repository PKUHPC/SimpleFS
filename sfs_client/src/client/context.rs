use lazy_static::*;
use rdma_sys::{rdma_event_channel, rdma_cm_id, rdma_disconnect, rdma_destroy_event_channel};
use sfs_global::global::endpoint::SFSEndpoint;
use sfs_global::global::network::config::CLIENT_CM_IDS;
use sfs_rpc::proto::server_grpc::SfsHandleClient;
use tokio::runtime::{Builder, Runtime};

use std::collections::HashMap;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::JoinHandle;

use bit_vec::*;

use crate::client::init::init_environment;
use crate::client::openfile::OpenFileMap;
use sfs_global::global::distributor::SimpleHashDistributor;
use sfs_global::global::error_msg::error_msg;
use sfs_global::global::fsconfig::SFSConfig;
use sfs_global::global::util::path_util::{
    has_trailing_slash, is_absolute, is_relative, split_path,
};

use super::network::rdmacm::RDMACMContext;
use super::path::resolve;

/*
#[link(name = "syscall_no_intercept", kind = "static")]
extern "C" {
    pub fn syscall_no_intercept(syscall_number: i64, ...) -> i32;
}
*/

#[allow(dead_code)]
pub static MAX_OPEN_FDS: u32 = 10000000;
pub static MIN_INTERNAL_FD: i32 = 1000000;
#[allow(dead_code)]
pub static MAX_USER_FDS: i32 = MIN_INTERNAL_FD;
pub static MAX_INTERNAL_FDS: i32 = 15000000;

static AT_FDCWD: i32 = -100;
static SEPERATOR: char = '/';

enum InterceptionStat {
    Disabled,
    Initialize,
    Enabled,
}
impl Clone for InterceptionStat {
    fn clone(&self) -> Self {
        match self {
            Self::Disabled => Self::Disabled,
            Self::Initialize => Self::Initialize,
            Self::Enabled => Self::Enabled,
        }
    }
}
lazy_static! {
    static ref INTERCEPTION_ENABLE: Mutex<InterceptionStat> =
        Mutex::new(InterceptionStat::Disabled);
}
pub fn start_interception() {
    *INTERCEPTION_ENABLE.lock().unwrap() = InterceptionStat::Initialize;
}
pub fn enable_interception() {
    *INTERCEPTION_ENABLE.lock().unwrap() = InterceptionStat::Enabled;
}
pub fn disable_interception() {
    *INTERCEPTION_ENABLE.lock().unwrap() = InterceptionStat::Disabled;
}
pub fn interception_enabled() -> bool {
    let stat = (*INTERCEPTION_ENABLE.lock().unwrap()).clone();
    match stat {
        InterceptionStat::Disabled => {
            start_interception();
            if StaticContext::get_instance().init_flag {
                enable_interception();
                true
            } else {
                false
            }
        }
        InterceptionStat::Initialize => false,
        InterceptionStat::Enabled => true,
    }
}

pub enum RelativizeStatus {
    Internal,
    External,
    FdUnknown,
    FdNotADir,
    Error,
}
// Context that will change in multi-thread
#[allow(dead_code)]
pub struct DynamicContext {
    open_file_map_: Arc<Mutex<OpenFileMap>>,
    internal_fds_: Mutex<BitVec>,
    protected_fds_: BitVec,

    cwd_: Mutex<String>,
    pub debug_counter: Mutex<i32>,
}
lazy_static! {
    static ref DCTX: DynamicContext = DynamicContext {
        open_file_map_: Arc::new(Mutex::new(OpenFileMap::new())),
        internal_fds_: Mutex::new(BitVec::from_elem(MAX_INTERNAL_FDS as usize, true)),
        protected_fds_: BitVec::from_elem(MAX_INTERNAL_FDS as usize, true),

        cwd_: Mutex::new("".to_string()),
        debug_counter: Mutex::new(0)
    };
}
impl DynamicContext {
    pub fn get_instance() -> &'static DynamicContext {
        &DCTX
    }
    pub fn relativize_fd_path(
        &self,
        dirfd: i32,
        raw_path: &String,
        resolve_last_link: bool,
    ) -> (RelativizeStatus, String) {
        if !interception_enabled() {
            error_msg(
                "client::simplefs_context::ClientContext::relativize_fd_path".to_string(),
                "interception need to be enabled".to_string(),
            );
            return (RelativizeStatus::Error, raw_path.clone());
        }
        if SCTX.mountdir_.len() == 0 {
            error_msg(
                "client::simplefs_context::ClientContext::relativize_fd_path".to_string(),
                "file system not mounted".to_string(),
            );
            return (RelativizeStatus::Error, raw_path.clone());
        }
        if raw_path.len() == 0 {
            error_msg(
                "client::simplefs_context::ClientContext::relativize_fd_path".to_string(),
                "raw path is empty".to_string(),
            );
            return (RelativizeStatus::Error, raw_path.clone());
        }
        let mut path: String;
        if is_relative(raw_path) {
            if dirfd == AT_FDCWD {
                path = self.get_cwd().clone() + raw_path;
            } else {
                if !self.open_file_map_.lock().unwrap().exist(dirfd) {
                    return (RelativizeStatus::FdUnknown, raw_path.to_string());
                }
                if let Some(dir) = self.open_file_map_.lock().unwrap().get_dir(dirfd) {
                    path = SCTX.get_mountdir().clone() + dir.lock().unwrap().get_path();
                    path.push(SEPERATOR);
                    path.push_str(raw_path);
                } else {
                    return (RelativizeStatus::FdNotADir, raw_path.to_string());
                }
            }
        } else {
            path = raw_path.clone();
        }
        let resolve_res = resolve(&path, resolve_last_link);
        if resolve_res.0 {
            return (RelativizeStatus::Internal, resolve_res.1);
        }
        (RelativizeStatus::External, raw_path.to_string())
    }
    pub fn relativize_path(&self, raw_path: &String, resolve_last_link: bool) -> (bool, String) {
        if !interception_enabled() {
            error_msg(
                "client::simplefs_context::ClientContext::relativize_path".to_string(),
                "interception need to be enabled".to_string(),
            );
            return (false, raw_path.clone());
        }
        if SCTX.mountdir_.len() == 0 {
            error_msg(
                "client::simplefs_context::ClientContext::relativize_path".to_string(),
                "file system not mounted".to_string(),
            );
            return (false, raw_path.clone());
        }
        if raw_path.len() == 0 {
            error_msg(
                "client::simplefs_context::ClientContext::relativize_path".to_string(),
                "raw path is empty".to_string(),
            );
            return (false, raw_path.clone());
        }
        let path: String;
        if is_relative(&raw_path) {
            path = self.get_cwd().clone() + &raw_path.clone();
        } else {
            path = raw_path.clone();
        }
        let resolve_res = resolve(&path, resolve_last_link);
        if resolve_res.0 {
            resolve_res
        } else {
            (resolve_res.0, raw_path.clone())
        }
    }
    pub fn get_ofm(&self) -> Arc<Mutex<OpenFileMap>> {
        Arc::clone(&self.open_file_map_)
    }
    // internal fd is not implmented currently
    pub fn register_internal_fd(&mut self, fd: i32) -> i32 {
        if fd < 0 {
            error_msg(
                "client:client_context:register_INTERNAL_fd".to_string(),
                "file descriptor must be positive".to_string(),
            );
            return fd;
        }
        if fd < MIN_INTERNAL_FD as i32 {
            error_msg(
                "client:client_context:register_INTERNAL_fd".to_string(),
                "file descriptor must be larger than MIN_INTERNAL_FD".to_string(),
            );
            return fd;
        }
        if !SCTX.internal_fds_must_relocate_ {
            (*self.internal_fds_.lock().unwrap()).set(fd as usize, false);
            return fd;
        }
        let mut pos: usize = MAX_INTERNAL_FDS as usize + 1;
        for (index, value) in (*self.internal_fds_.lock().unwrap()).iter().enumerate() {
            if value {
                pos = index;
                break;
            }
        }
        if pos == MAX_INTERNAL_FDS as usize + 1 {
            error_msg(
                "client:client_context:register_INTERNAL_fd".to_string(),
                "no available INTERNAL fd slot".to_string(),
            );
            return fd;
        }
        (*self.internal_fds_.lock().unwrap()).set(pos, false);
        let ifd = 0;
        //unsafe {
        //ifd = syscall_no_intercept(SYS_dup3, fd.clone(), pos + MIN_INTERNAL_FD as usize, O_CLOEXEC);
        //syscall_no_intercept(SYS_close, fd.clone());
        //}
        ifd
    }
    pub fn unregister_internal_fd(&mut self, fd: i32) {
        if fd < MIN_INTERNAL_FD as i32 {
            error_msg(
                "client:client_context:unregister_INTERNAL_fd".to_string(),
                "file descriptor must be larger than MIN_INTERNAL_FD".to_string(),
            );
            return;
        }
        let pos: usize = fd as usize - MIN_INTERNAL_FD as usize;
        (*self.internal_fds_.lock().unwrap()).set(pos, true);
    }
    #[allow(unreachable_code, unused)]
    pub fn is_internal_fd(&self, fd: i32) -> bool {
        if fd < MIN_INTERNAL_FD as i32 {
            //error_msg("client:client_context:is_INTERNAL_fd".to_string(), "file descriptor must be larger than MIN_INTERNAL_FD".to_string());
            return false;
        }
        let pos: usize = fd as usize - MIN_INTERNAL_FD as usize;
        return !(*self.internal_fds_.lock().unwrap()).get(pos).unwrap();
    }
    pub fn set_cwd(&self, path: String) {
        *self.cwd_.lock().unwrap() = path;
    }
    pub fn get_cwd(&self) -> MutexGuard<'_, String> {
        self.cwd_.lock().unwrap()
    }
    pub fn incr_counter() {
        *DCTX.debug_counter.lock().unwrap() += 1;
    }
    pub fn decr_counter() {
        *DCTX.debug_counter.lock().unwrap() -= 1;
    }
    pub fn show_counter() {
        println!("current counter: {}", *DCTX.debug_counter.lock().unwrap());
    }
}

// Context that is read only after initialize
pub struct StaticContext {
    distributor_: Arc<SimpleHashDistributor>,
    fs_config_: Arc<SFSConfig>,

    mountdir_components_: Arc<Vec<String>>,
    mountdir_: String,

    hosts_: Vec<SFSEndpoint>,
    clients_: Vec<SfsHandleClient>,
    local_host_id: u64,
    fwd_host_id: u64,
    rpc_protocol_: String,
    auto_sm_: bool,

    internal_fds_must_relocate_: bool,
    runtime_: Arc<Runtime>,

    pub rdma_addr: String,

    pub event_channel: u64,
    pub handle: Option<JoinHandle<()>>,

    pub write_cm_ids: HashMap<u64, Vec<Mutex<u64>>>,
    pub wait_write_idx: Mutex<usize>,

    pub read_cm_ids: HashMap<u64, Vec<Mutex<u64>>>,
    pub wait_read_idx: Mutex<usize>,

    pub init_flag: bool,
}
lazy_static! {
    static ref SCTX: StaticContext = init_environment();
}
impl StaticContext {
    pub fn get_instance() -> &'static StaticContext {
        &SCTX
    }
    pub fn init_logging() {}
    pub fn new() -> StaticContext {
        StaticContext {
            distributor_: Arc::new(SimpleHashDistributor::init()),
            fs_config_: Arc::new(SFSConfig::new()),
            mountdir_components_: Arc::new(Vec::new()),
            mountdir_: "".to_string(),
            hosts_: Vec::new(),
            clients_: Vec::new(),
            local_host_id: 0,
            fwd_host_id: 0,
            rpc_protocol_: "tcp".to_string(),
            auto_sm_: false,
            internal_fds_must_relocate_: true,
            init_flag: false,
            rdma_addr: "127.0.0.1".to_string(),
            runtime_: Arc::new(
                Builder::new_current_thread()
                    .enable_all()
                    .thread_stack_size(12 * 1024 * 1024)
                    .build()
                    .unwrap(),
            ),
            event_channel: null_mut() as *mut rdma_event_channel as u64,
            write_cm_ids: HashMap::new(),
            wait_write_idx: Mutex::new(0),
            read_cm_ids: HashMap::new(),
            wait_read_idx: Mutex::new(0),
            handle: None
            
        }
    }
    pub fn set_mountdir(&mut self, mut path: String) {
        if !is_absolute(&path) {
            error_msg(
                "client::simplefs_context::mountdir".to_string(),
                "must be mounted at an absolute path".to_string(),
            );
        }
        if has_trailing_slash(&path) {
            path = path[0..path.len() - 1].to_string();
        }
        self.mountdir_components_ = Arc::new(split_path(path.clone()));
        self.mountdir_ = path;
    }
    pub fn get_mountdir(&self) -> &String {
        &self.mountdir_
    }
    pub fn get_mountdir_components(&self) -> Arc<Vec<String>> {
        Arc::clone(&self.mountdir_components_)
    }
    pub fn get_hosts(&self) -> &Vec<SFSEndpoint> {
        &self.hosts_
    }
    pub fn set_hosts(&mut self, hosts: Vec<SFSEndpoint>) {
        self.hosts_ = hosts;
    }
    pub fn get_clients(&self) -> &Vec<SfsHandleClient> {
        &self.clients_
    }
    pub fn set_clients(&mut self, clients: Vec<SfsHandleClient>) {
        self.clients_ = clients;
    }
    pub fn clear_hosts(&mut self) {
        self.hosts_ = Vec::new();
    }
    pub fn set_local_host_id(&mut self, host_id: u64) {
        self.local_host_id = host_id;
    }
    pub fn get_local_host_id(&self) -> u64 {
        self.local_host_id.clone()
    }
    pub fn set_fwd_host_id(&mut self, host_id: u64) {
        self.fwd_host_id = host_id;
    }
    pub fn get_fwd_host_id(&self) -> u64 {
        self.fwd_host_id.clone()
    }
    pub fn set_rpc_protocol(&mut self, protocol: String) {
        self.rpc_protocol_ = protocol;
    }
    pub fn get_rpc_protocol(&self) -> &String {
        &self.rpc_protocol_
    }
    pub fn set_auto_sm(&mut self, auto_sm: bool) {
        self.auto_sm_ = auto_sm;
    }
    pub fn get_suto_sm(&self) -> bool {
        self.auto_sm_.clone()
    }
    pub fn set_distributor(&mut self, d: SimpleHashDistributor) {
        self.distributor_ = Arc::new(d);
    }
    pub fn get_distributor(&self) -> Arc<SimpleHashDistributor> {
        Arc::clone(&self.distributor_)
    }
    pub fn get_fsconfig(&self) -> Arc<SFSConfig> {
        Arc::clone(&self.fs_config_)
    }
    pub fn set_fsconfig(&mut self, config: SFSConfig) {
        self.fs_config_ = Arc::new(config);
    }
    pub fn get_init_flag(&self) -> bool {
        self.init_flag
    }
    pub fn get_runtime(&self) -> Arc<Runtime> {
        Arc::clone(&self.runtime_)
    }
    pub fn get_rdma_addr(&self) -> &String {
        &self.rdma_addr
    }
    pub fn get_event_channel(&self) -> *mut rdma_event_channel{
        self.event_channel as *mut rdma_event_channel
    }
    pub fn get_write_cm_id(&'static self, host_id: u64) -> Option<MutexGuard<'static, u64>>{
        assert_eq!(self.write_cm_ids.get(&host_id).unwrap().len(), CLIENT_CM_IDS);
        for lock in self.write_cm_ids.get(&host_id).unwrap().iter(){
            if let Ok(guard) = lock.try_lock(){
                return Some(guard);
            }
        }
        let mut idx_guard = self.wait_write_idx.lock().unwrap();
        let idx = *idx_guard;
        (*idx_guard) = idx + 1;
        if (*idx_guard) >= CLIENT_CM_IDS{
            (*idx_guard) = 0;
        }
        drop(idx_guard);
        return Some(self.write_cm_ids.get(&host_id).unwrap().get(idx).unwrap().lock().unwrap());
    }
    pub fn get_read_cm_id(&'static self, host_id: u64) -> Option<MutexGuard<'static, u64>>{
        assert_eq!(self.read_cm_ids.get(&host_id).unwrap().len(), CLIENT_CM_IDS);
        for lock in self.read_cm_ids.get(&host_id).unwrap().iter(){
            if let Ok(guard) = lock.try_lock(){
                return Some(guard);
            }
        }
        let mut idx_guard = self.wait_read_idx.lock().unwrap();
        let idx = *idx_guard;
        (*idx_guard) = idx + 1;
        if (*idx_guard) >= CLIENT_CM_IDS{
            (*idx_guard) = 0;
        }
        drop(idx_guard);
        return Some(self.read_cm_ids.get(&host_id).unwrap().get(idx).unwrap().lock().unwrap());
    }
    pub fn protect_user_fds() {}
    pub fn unprotect_user_fds() {}
}
impl Drop for StaticContext{
    fn drop(&mut self) {
        unsafe{
            rdma_destroy_event_channel(self.event_channel as *mut rdma_event_channel);
        }
        let handle = self.handle.take().unwrap();
        handle.join().unwrap();
        for (_host_id, cm_ids) in self.write_cm_ids.iter(){
            for lock in cm_ids{
                let cm_id = *lock.lock().unwrap() as *mut rdma_cm_id;
                unsafe{
                    rdma_disconnect(cm_id);
                    
                    let ctx = (*cm_id).context as *mut RDMACMContext;
                    ((*ctx).on_disconnect)(cm_id);
                }
            }
        }
        for (_host_id, cm_ids) in self.read_cm_ids.iter(){
            for lock in cm_ids{
                let cm_id = *lock.lock().unwrap() as *mut rdma_cm_id;
                unsafe{
                    rdma_disconnect(cm_id);
                
                    let ctx = (*cm_id).context as *mut RDMACMContext;
                    ((*ctx).on_disconnect)(cm_id);
                }
            }
        }
    }
}
