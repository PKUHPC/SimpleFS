use bit_vec::BitVec;
use std::{
    collections::HashMap,
    sync::{atomic::AtomicBool, Arc, Mutex},
};

use sfs_global::global::error_msg::error_msg;

pub static O_RDONLY: i32 = 0x0000; // open for reading only
pub static O_WRONLY: i32 = 0x0001; // open for writing only
pub static O_RDWR: i32 = 0x0002; // open for reading and writing
pub static O_APPEND: i32 = 0x0008; // writes done at eof

pub static O_CREAT: i32 = 0x0100; // create and open file
pub static O_TRUNC: i32 = 0x0200; // open and truncate
pub static O_EXCL: i32 = 0x0400; // open only if file doesn't already exist

pub static MAX_FD: i32 = 0x7fffffff;
pub static MIN_FD: i32 = 100000;

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum FileType {
    SFS_REGULAR,
    SFS_DIRECTORY,
}
impl Clone for FileType {
    fn clone(&self) -> Self {
        match self {
            Self::SFS_REGULAR => Self::SFS_REGULAR,
            Self::SFS_DIRECTORY => Self::SFS_DIRECTORY,
        }
    }
}
pub enum OpenFileFlags {
    Append = 0,
    Creat,
    Trunc,
    Rdonly,
    Wronly,
    Rdwr,
    Cloexec,
    FlagCount,
    Unknown,
}

#[derive(Debug)]
pub struct SFSDirEntry {
    name_: String,
    type_: FileType,
}
impl SFSDirEntry {
    pub fn new() -> SFSDirEntry {
        SFSDirEntry {
            name_: "".to_string(),
            type_: FileType::SFS_REGULAR,
        }
    }
    pub fn get_name(&self) -> String {
        self.name_.clone()
    }
    pub fn get_type(&self) -> FileType {
        self.type_.clone()
    }
}

fn to_index(flag: OpenFileFlags) -> usize {
    match flag {
        OpenFileFlags::Append => 0,
        OpenFileFlags::Creat => 1,
        OpenFileFlags::Trunc => 2,
        OpenFileFlags::Rdonly => 3,
        OpenFileFlags::Wronly => 4,
        OpenFileFlags::Rdwr => 5,
        OpenFileFlags::Cloexec => 6,
        OpenFileFlags::FlagCount => 7,
        OpenFileFlags::Unknown => 8,
    }
}
pub struct OpenFile {
    type_: FileType,
    path_: String,
    flags_: Arc<Mutex<BitVec>>,
    pos_: Arc<Mutex<i64>>,
    pub entries_: Vec<Arc<SFSDirEntry>>, // for directory
                                         //pos_mutex_: Mutex<i32>,
                                         //flag_mutex_: Mutex<i32>
}
impl OpenFile {
    pub fn new(_path: &String, _flags: i32, _type: FileType) -> OpenFile {
        let mut flag_vec = BitVec::from_elem(9, false);

        if _flags & O_CREAT != 0 {
            flag_vec.set(to_index(OpenFileFlags::Creat), true);
        }
        if _flags & O_APPEND != 0 {
            flag_vec.set(to_index(OpenFileFlags::Append), true);
        }
        if _flags & O_TRUNC != 0 {
            flag_vec.set(to_index(OpenFileFlags::Trunc), true);
        }
        if _flags & O_RDONLY != 0 {
            flag_vec.set(to_index(OpenFileFlags::Rdonly), true);
        }
        if _flags & O_WRONLY != 0 {
            flag_vec.set(to_index(OpenFileFlags::Wronly), true);
        }
        if _flags & O_RDWR != 0 {
            flag_vec.set(to_index(OpenFileFlags::Rdwr), true);
        }
        OpenFile {
            type_: _type,
            path_: _path.clone(),
            flags_: Arc::new(Mutex::new(flag_vec)),
            pos_: Arc::new(Mutex::new(0)),
            entries_: Vec::new(), //pos_mutex_: Mutex::new(0),
                                  //flag_mutex_: Mutex::new(0)
        }
    }
    pub fn get_path(&self) -> &String {
        &self.path_
    }
    pub fn set_path(&mut self, new_path: String) {
        self.path_ = new_path;
    }
    pub fn get_pos(&self) -> i64 {
        *self.pos_.lock().unwrap()
    }
    pub fn set_pos(&mut self, new_pos: i64) {
        *self.pos_.lock().unwrap() = new_pos;
    }
    pub fn get_flag(&self, flag: OpenFileFlags) -> bool {
        let res = self.flags_.lock().unwrap().get(to_index(flag));
        if let Some(b) = res {
            return b;
        } else {
            print!("error::client::openfile::flag - invlaid flag detected");
            return false;
        }
    }
    pub fn set_flag(&mut self, flag: OpenFileFlags, new_value: bool) {
        self.flags_.lock().unwrap().set(to_index(flag), new_value);
    }
    pub fn get_type(&self) -> FileType {
        self.type_.clone()
    }
    pub fn add(&mut self, name: String, file_type: FileType) {
        match self.type_ {
            FileType::SFS_REGULAR => {
                return;
            }
            FileType::SFS_DIRECTORY => {
                self.entries_.push(Arc::new(SFSDirEntry {
                    name_: name,
                    type_: file_type,
                }));
            }
        }
    }
    pub fn getdent(&self, pos: i64) -> Arc<SFSDirEntry> {
        match self.type_ {
            FileType::SFS_REGULAR => Arc::new(SFSDirEntry::new()),
            FileType::SFS_DIRECTORY => Arc::clone(&self.entries_[pos as usize]),
        }
    }
    pub fn get_size(&self) -> usize {
        match self.type_ {
            FileType::SFS_REGULAR => 0,
            FileType::SFS_DIRECTORY => self.entries_.len(),
        }
    }
}

pub struct OpenFileMap {
    files_: Arc<Mutex<HashMap<i32, Arc<Mutex<OpenFile>>>>>,
    //files_mutex_: Mutex<i32>,
    fd_idx_: Arc<Mutex<i32>>,
    //fd_idx_mutex_: Mutex<i32>,
    fd_validation_needed_: AtomicBool,
}

impl OpenFileMap {
    pub fn new() -> OpenFileMap {
        OpenFileMap {
            files_: Arc::new(Mutex::new(HashMap::new())),
            fd_idx_: Arc::new(Mutex::new(MIN_FD)),
            fd_validation_needed_: AtomicBool::new(false),
        }
    }
    pub fn get(&self, fd: i32) -> Option<Arc<Mutex<OpenFile>>> {
        if let Some(f) = self.files_.lock().unwrap().get(&fd) {
            Some(Arc::clone(f))
        } else {
            error_msg(
                "client::open_file_map::get".to_string(),
                "file descriptor not found".to_string(),
            );
            None
        }
    }
    pub fn get_dir(&self, dirfd: i32) -> Option<Arc<Mutex<OpenFile>>> {
        if let Some(f) = self.get(dirfd) {
            match f.lock().unwrap().get_type() {
                FileType::SFS_REGULAR => None,
                FileType::SFS_DIRECTORY => Some(Arc::clone(&f)),
            }
        } else {
            error_msg(
                "client::open_file_map::get_dir".to_string(),
                "file descriptor not found".to_string(),
            );
            None
        }
    }
    pub fn exist(&self, fd: i32) -> bool {
        (*self.files_.lock().unwrap()).contains_key(&fd)
    }
    fn generate_fd_idx(&mut self) -> i32 {
        let mut fd_idx = self.fd_idx_.lock().unwrap();
        if *fd_idx == MAX_FD {
            *fd_idx = MIN_FD;
            self.fd_validation_needed_ = AtomicBool::new(true);
        }
        *fd_idx = *fd_idx + 1;
        drop(fd_idx);
        return *self.fd_idx_.lock().unwrap();
    }
    pub fn safe_generate_fd_idx(&mut self) -> i32 {
        let mut fd = self.generate_fd_idx();
        if self
            .fd_validation_needed_
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            while self.exist(fd) {
                fd = self.generate_fd_idx();
            }
        }
        return fd;
    }
    pub fn get_fd_idx(&self) -> i32 {
        return *self.fd_idx_.lock().unwrap();
    }
    pub fn add(&mut self, file: Arc<Mutex<OpenFile>>) -> i32 {
        let fd = self.safe_generate_fd_idx();
        (*self.files_.lock().unwrap()).insert(fd, file);
        return fd;
    }
    pub fn remove(&mut self, fd: i32) -> bool {
        if !self.exist(fd) {
            return false;
        } else {
            self.files_.lock().unwrap().remove(&fd);
            if self
                .fd_validation_needed_
                .load(std::sync::atomic::Ordering::Relaxed)
                && self.files_.lock().unwrap().capacity() == 0
            {
                self.fd_validation_needed_ = AtomicBool::new(false);
            }
            return true;
        }
    }
    // 'dup' need to be checked twice
    pub fn dup(&mut self, oldfd: i32) -> i32 {
        if let Some(open_file) = self.get(oldfd) {
            let newfd = self.safe_generate_fd_idx();
            (*self.files_.lock().unwrap()).insert(newfd, open_file);
            return newfd;
        } else {
            error_msg(
                "clent::simplefs_openfile::openfilemap::dup".to_string(),
                "no such file".to_string(),
            );
            return -1;
        }
    }
    pub fn dup2(&mut self, oldfd: i32, newfd: i32) -> i32 {
        if let Some(open_file) = self.get(oldfd) {
            if self.exist(newfd) {
                self.remove(newfd);
            }
            if self.get_fd_idx() < newfd && newfd != 0 && newfd != 1 && newfd != 2 {
                self.fd_validation_needed_ = AtomicBool::new(true);
            }
            (*self.files_.lock().unwrap()).insert(newfd, open_file);
            return newfd;
        } else {
            error_msg(
                "clent::simplefs_openfile::openfilemap::dup2".to_string(),
                "no such file".to_string(),
            );
            return -1;
        }
    }
    pub fn get_length(&self) -> usize {
        self.files_.lock().unwrap().len()
    }
}
