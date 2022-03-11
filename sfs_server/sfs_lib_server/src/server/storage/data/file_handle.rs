
pub static INIT_VALUE: i32 = -1;
pub struct FileHandle{
    fd_: i32,
    path_: String
}
impl FileHandle{
    pub fn new(fd_: i32, path_: String) -> FileHandle{
        FileHandle{
            fd_: fd_,
            path_: path_
        }
    }
    pub fn valid(&self) -> bool{
        self.fd_ != INIT_VALUE 
    }
    pub fn native(&self) -> i32{
        self.fd_.clone()
    }
    pub fn close(&mut self) -> bool{
        if self.valid(){
        }
        self.fd_ = INIT_VALUE;
        true
    }
}