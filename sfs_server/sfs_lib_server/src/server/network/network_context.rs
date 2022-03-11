use std::sync::{Mutex, MutexGuard};

use lazy_static::*;
pub struct NetworkContext{
    self_addr: String
}
lazy_static!{
    static ref NTC: Mutex<NetworkContext> = Mutex::new(
        NetworkContext{
            self_addr: "".to_string()
        }
    );
}
impl NetworkContext{
    pub fn get_instance() -> MutexGuard<'static, NetworkContext>{
        NTC.lock().unwrap()
    }
    pub fn get_self_addr(&self) -> &String{
        &self.self_addr
    }
    pub fn set_self_addr(&mut self, addr: String){
        self.self_addr = addr;
    }
}