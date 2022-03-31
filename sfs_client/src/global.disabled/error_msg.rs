use crate::client::context::interception_enabled;

use super::fsconfig::ENABLE_OUTPUT;

#[allow(unused_variables, unreachable_code)]
pub fn error_msg(func: String, msg: String) {
    if ENABLE_OUTPUT && interception_enabled() {
        print!("error::{} - {}\n", func, msg);
    }
}
