use rocksdb::MergeOperands;
use serde::{Deserialize, Serialize};
use sfs_global::global::{
    metadata::Metadata, network::config::CHUNK_SIZE, util::serde_util::deserialize,
};

use crate::error_msg::error_msg;
#[derive(Debug, Deserialize, Serialize)]
pub enum Operand {
    Create {
        md: Vec<u8>,
    },
    IncreaseSize {
        size: usize,
        append: bool,
        time: i64,
    },
    DecreaseSize {
        size: usize,
        time: i64,
    },
}
#[allow(unused_variables)]
pub fn full_merge(
    new_key: &[u8],
    existing_val: Option<&[u8]>,
    operands: &MergeOperands,
) -> Option<Vec<u8>> {
    //println!("full merging on key {} ...", String::from_utf8(new_key.to_vec()).unwrap());
    let mut md;
    let mut iter = operands.into_iter();
    if let Some(val) = existing_val {
        md = Metadata::deserialize(&val.to_vec());
    } else {
        if iter.size_hint().0 == 0 {
            error_msg(
                "server::merge::full_merge".to_string(),
                "given key has no existing value and no operands available".to_string(),
            );
            return None;
        }
        let op_s = iter.next().unwrap().to_vec();
        match deserialize::<Operand>(&op_s) {
            Operand::Create { md: data } => {
                md = Metadata::deserialize(&data);
            }
            _ => {
                error_msg(
                    "server::merge::full_merge".to_string(),
                    "given key has no existing value and the first operand is not create"
                        .to_string(),
                );
                return None;
            }
        }
    }
    let mut fsize = md.get_size();
    let mut mtime: i64 = md.get_modify_time();
    let mut ctime: i64 = md.get_change_time();
    let mut op;
    while {
        op = iter.next();
        op != None
    } {
        let op_s = op.unwrap().to_vec();
        match deserialize::<Operand>(&op_s) {
            Operand::Create { md: data } => {
                continue;
            }
            Operand::IncreaseSize { size, append, time } => {
                if append {
                    fsize += size as i64;
                } else {
                    fsize = std::cmp::max(fsize, size as i64);
                }
                mtime = std::cmp::max(mtime, time);
            }
            Operand::DecreaseSize { size, time } => {
                if size as i64 > fsize {
                    error_msg(
                        "server::merge::full_merge".to_string(),
                        "you can't decrease file size to a bigger one".to_string(),
                    );
                    return None;
                }
                fsize = size as i64;
                mtime = std::cmp::max(mtime, time);
                ctime = std::cmp::max(ctime, time);
            }
        }
    }
    if fsize as u64 > CHUNK_SIZE {
        md.unstuff();
    }
    md.set_size(fsize);
    md.set_modify_time(mtime);
    md.set_change_time(ctime);
    Some(md.serialize())
}
#[allow(unused_variables)]
pub fn partial_merge(
    new_key: &[u8],
    existing_val: Option<&[u8]>,
    operands: &MergeOperands,
) -> Option<Vec<u8>> {
    //println!("partial merging ...");
    None
}
