use libc::gethostname;

pub fn get_var(name: String, default: String) -> String {
    if let Ok(var) = std::env::var(name) {
        var
    } else {
        default
    }
}
pub fn get_hostname(short_hostname: bool) -> String {
    let hostname: [u8; 1024] = [0; 1024];
    let ret = unsafe { gethostname(hostname.as_ptr() as *mut i8, 1024) };
    if ret == 0 {
        let mut hostname = String::from_utf8(hostname.to_vec()).unwrap();
        if !short_hostname {
            return hostname;
        }
        if let Some(pos) = hostname.find(&".".to_string()) {
            hostname = hostname[0..pos].to_string();
        }
        if let Some(pos) = hostname.find(&"\0".to_string()) {
            hostname = hostname[0..pos].to_string();
        }
        return hostname;
    } else {
        return "".to_string();
    }
}
