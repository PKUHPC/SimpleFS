use lazy_static::*;
use sfs_global::global::endpoint::SFSEndpoint;

use std::sync::{Arc, Mutex, MutexGuard};

use bit_vec::*;

use crate::client::init::init_environment;
use crate::client::openfile::OpenFileMap;
use sfs_global::global::distributor::SimpleHashDistributor;
use sfs_global::global::error_msg::error_msg;
use sfs_global::global::fsconfig::SFSConfig;
use sfs_global::global::util::path_util::{
    has_trailing_slash, is_absolute, is_relative, split_path,
};

use super::path::resolve;

/*
#[link(name = "syscall_no_intercept", kind = "static")]
extern "C" {
    pub fn syscall_no_intercept(syscall_number: i64, ...) -> i32;
}
*/

static MAX_INTERNAL_FDS: u32 = 256;
static MAX_OPEN_FDS: u32 = 1024;
static MIN_INTERNEL_FD: u32 = MAX_OPEN_FDS - MAX_INTERNAL_FDS;
#[allow(dead_code)]
static MAX_USER_FDS: u32 = MIN_INTERNEL_FD;
static MAX_INTERNEL_FDS: u32 = 100000;

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

    cwd_: String,
}
lazy_static! {
    static ref DCTX: Mutex<DynamicContext> = Mutex::new(DynamicContext {
        open_file_map_: Arc::new(Mutex::new(OpenFileMap::new())),
        internal_fds_: Mutex::new(BitVec::new()),
        protected_fds_: BitVec::from_elem(MAX_INTERNEL_FDS as usize, true),

        cwd_: "".to_string()
    });
}
impl DynamicContext {
    pub fn get_instance() -> MutexGuard<'static, DynamicContext> {
        DCTX.lock().unwrap()
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
                path = self.cwd_.clone() + raw_path;
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
            path = self.cwd_.clone() + &raw_path.clone();
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
    pub fn register_internel_fd(&mut self, fd: i32) -> i32 {
        if fd < 0 {
            error_msg(
                "client:client_context:register_internel_fd".to_string(),
                "file descriptor must be positive".to_string(),
            );
            return fd;
        }
        if !SCTX.internal_fds_must_relocate_ {
            if fd < MIN_INTERNEL_FD as i32 {
                error_msg(
                    "client:client_context:register_internel_fd".to_string(),
                    "file descriptor must be larger than MIN_INTERNEL_FD".to_string(),
                );
                return fd;
            }
            (*self.internal_fds_.lock().unwrap()).set(fd.clone() as usize, false);
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
                "client:client_context:register_internel_fd".to_string(),
                "no available internel fd slot".to_string(),
            );
            return fd;
        }
        (*self.internal_fds_.lock().unwrap()).set(pos, false);
        let ifd = 0;
        //unsafe {
        //ifd = syscall_no_intercept(SYS_dup3, fd.clone(), pos + MIN_INTERNEL_FD as usize, O_CLOEXEC);
        //syscall_no_intercept(SYS_close, fd.clone());
        //}
        ifd
    }
    pub fn unregister_internel_fd(&mut self, fd: i32) {
        if fd < MIN_INTERNEL_FD as i32 {
            error_msg(
                "client:client_context:unregister_internel_fd".to_string(),
                "file descriptor must be larger than MIN_INTERNEL_FD".to_string(),
            );
            return;
        }
        let pos: usize = fd as usize - MIN_INTERNEL_FD as usize;
        (*self.internal_fds_.lock().unwrap()).set(pos, true);
    }
    pub fn is_internel_fd(&self, fd: i32) -> bool {
        if fd < MIN_INTERNEL_FD as i32 {
            //error_msg("client:client_context:is_internel_fd".to_string(), "file descriptor must be larger than MIN_INTERNEL_FD".to_string());
            return false;
        }
        let pos: usize = fd as usize - MIN_INTERNEL_FD as usize;
        return !(*self.internal_fds_.lock().unwrap()).get(pos).unwrap();
    }
    pub fn set_cwd(&mut self, path: String) {
        self.cwd_ = path;
    }
    pub fn get_cwd(&self) -> &String {
        &self.cwd_
    }
}

// Context that is read only after initialize
pub struct StaticContext {
    distributor_: Arc<SimpleHashDistributor>,
    fs_config_: Arc<SFSConfig>,

    mountdir_components_: Arc<Vec<String>>,
    mountdir_: String,

    hosts_: Vec<SFSEndpoint>,
    local_host_id: u64,
    fwd_host_id: u64,
    rpc_protocol_: String,
    auto_sm_: bool,

    internal_fds_must_relocate_: bool,

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
            local_host_id: 0,
            fwd_host_id: 0,
            rpc_protocol_: "tcp".to_string(),
            auto_sm_: false,
            internal_fds_must_relocate_: true,
            init_flag: false,
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
    pub fn protect_user_fds() {}
    pub fn unprotect_user_fds() {}
}
