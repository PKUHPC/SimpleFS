use rocksdb::MergeOperands;
use serde::{Deserialize, Serialize};
use sfs_global::global::{util::serde_util::deserialize};
#[derive(Debug, Deserialize, Serialize)]
pub enum Operand {
    Write { offset: u64, size: u64, data: Vec<u8> },
    Truncate {offset: u64}
}
#[allow(unused_variables)]
pub fn full_merge(
    new_key: &[u8],
    existing_val: Option<&[u8]>,
    operands: &MergeOperands,
) -> Option<Vec<u8>> {
    //println!("full merging on key {} ...", String::from_utf8(new_key.to_vec()).unwrap());
    let mut result = vec![0; 0];
    if let Some(data) = existing_val{
        result = data.to_vec();
    }
    let mut iter = operands.into_iter();
    let mut op;
    while{op = iter.next(); op != None}{
        let op_s = op.unwrap();
        match deserialize::<Operand>(&op_s.to_vec()){
            Operand::Write { offset, size, data } => {
                let fsize = std::cmp::max(result.len(), (offset + size) as usize);
                if fsize > result.len(){
                    result.append(&mut vec![0; fsize - result.len()]);
                }
                result.splice(offset as usize..(offset + size) as usize, data[0..size as usize].iter().cloned());
            },
            Operand::Truncate { offset } => {
                result = result[0..offset as usize].to_vec();
            },
        }
    }
    return Some(result.to_vec());
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
