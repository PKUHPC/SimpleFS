use std::time::{SystemTime, UNIX_EPOCH, self};
use std::fmt;

use libc::{S_IFDIR, S_IFREG};

pub fn S_ISREG(mode: u32) -> bool{
    mode & S_IFREG != 0
}
pub fn S_ISDIR(mode: u32) -> bool{
    mode & S_IFDIR != 0
}
#[derive(Debug)]
pub struct Metadata{
    access_time_: i64,
    modify_time_: i64,
    change_time_: i64,
    mode_: u32,
    link_count_: u64,
    size_: i64,
    blocks_: i64
}
impl fmt::Display for Metadata{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        write!(f, "c|{0}|{1}|{2}|{3}|{4}|{5}|{6}", self.get_mode(), self.get_size(), self.get_access_time(), self.get_modify_time(), self.get_change_time(), self.get_link_count(), self.get_blocks())
    }
}
impl Metadata{
    pub fn new() -> Metadata{
        Metadata{
            access_time_: time::SystemTime::now().elapsed().unwrap().as_secs() as i64,
            modify_time_: time::SystemTime::now().elapsed().unwrap().as_secs() as i64,
            change_time_: time::SystemTime::now().elapsed().unwrap().as_secs() as i64,
            mode_: 0,
            link_count_: 1,
            size_: 0,
            blocks_: 0
        }
    }
    pub fn deserialize(binary_str: &String) -> Result<Metadata, i32>{
        let s = binary_str.split('|');
        let vec = s.collect::<Vec<&str>>();
        if vec.len() != 8{
            print!("error::global::metadata::init_acm_time - invalid serialized metadata detected: {}", binary_str);
            Err(0)
        }
        else{
            let access_time = vec[3].parse::<i64>().unwrap();
            let modify_time = vec[4].parse::<i64>().unwrap();
            let change_time = vec[5].parse::<i64>().unwrap();
            let mode = vec[1].parse::<u32>().unwrap();
            let link_count = vec[6].parse::<u64>().unwrap();
            let size = vec[2].parse::<i64>().unwrap();
            let blocks = vec[7].parse::<i64>().unwrap();
            Ok(
                Metadata{
                    access_time_: access_time,
                    modify_time_: modify_time,
                    change_time_: change_time,
                    mode_: mode,
                    link_count_: link_count,
                    size_: size,
                    blocks_: blocks
                }
            )
        }
    }
    pub fn serialize(&self) -> String{
        self.to_string()
    }
    pub fn init_acm_time(&mut self){
        if let Ok(n) = SystemTime::now().duration_since(UNIX_EPOCH){
            self.access_time_ = n.as_micros() as i64;
            self.change_time_ = n.as_micros() as i64;
            self.modify_time_ = n.as_micros() as i64;
        } else{
            print!("error::global::metadata::init_acm_time - invalid time detected");
        }
    }
    pub fn update_acm_time(&mut self, a: bool, c: bool, m:bool){
        if let Ok(n) = SystemTime::now().duration_since(UNIX_EPOCH){
            if a {
                self.access_time_ = n.as_micros() as i64;
            }
            if c {
                self.change_time_ = n.as_micros() as i64;
            }
            if m {
                self.modify_time_ = n.as_micros() as i64;
            }
        } else{
            print!("error::global::metadata::update_acm_time - invalid time detected");
        }
    }
    pub fn get_access_time(&self) -> i64{
        self.access_time_
    }
    pub fn set_access_time(&mut self, atime: i64){
        self.access_time_ = atime;
    }
    pub fn get_modify_time(&self) -> i64{
        self.modify_time_
    }
    pub fn set_modify_time(&mut self, mtime: i64){
        self.modify_time_ = mtime;
    }
    pub fn get_change_time(&self) -> i64{
        self.change_time_
    }
    pub fn set_change_time(&mut self, ctime: i64){
        self.change_time_ = ctime;
    }
    pub fn get_mode(&self) -> u32{
        self.mode_
    }
    pub fn set_mode(&mut self, mode:u32){
        self.mode_ = mode;
    }
    pub fn get_link_count(&self) -> u64{
        self.link_count_
    }
    pub fn set_link_count(&mut self, link_count: u64){
        self.link_count_ = link_count;
    }
    pub fn get_size(&self) -> i64{
        self.size_
    }
    pub fn set_size(&mut self, size: i64){
        self.size_ = size;
    }
    pub fn get_blocks(&self) -> i64{
        self.blocks_
    }
    pub fn set_blocks(&mut self, blocks: i64){
        self.blocks_ = blocks;
    }
}
    
    
