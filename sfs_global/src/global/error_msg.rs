use super::fsconfig::ENABLE_OUTPUT;

#[allow(unused_variables, unreachable_code)]
pub fn error_msg(func: String, msg: String) {
    if ENABLE_OUTPUT{
        print!("error::{} - {}\n", func, msg);
    }
}
