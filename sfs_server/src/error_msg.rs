use crate::server::filesystem::storage_context::StorageContext;

#[allow(unused_variables, unreachable_code)]
pub fn error_msg(func: String, msg: String) {
    if StorageContext::get_instance().output() {
        print!("error::{} - {}\n", func, msg);
    }
}
