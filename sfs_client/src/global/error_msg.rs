use crate::client::context::interception_enabled;

#[allow(unused_variables, unreachable_code)]
pub fn error_msg(func: String, msg: String) {
    return;
    if interception_enabled() {
        print!("error::{} - {}\n", func, msg);
    }
}
