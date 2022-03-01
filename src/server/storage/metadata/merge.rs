use rocksdb::MergeOperands;

use crate::{global::{error_msg::error_msg, metadata::Metadata}};
enum OperandID{
    Create,
    IncreaseSize,
    DecreaseSize,
    Unknown
}
fn get_id_from_ch(ch: &char) -> OperandID{
    match ch {
        'c' => OperandID::Create,
        'i' => OperandID::IncreaseSize,
        'd' => OperandID::DecreaseSize,
        _ => OperandID::Unknown
    }
}
fn get_id_from_op(op_s: &String) -> OperandID{
    get_id_from_ch(&(op_s.as_bytes()[0] as char))
}
fn get_param_for_increase_size(op_s: &String) -> Option<(i64, bool)>{
    let s = op_s.split('|');
    let vec = s.collect::<Vec<&str>>();
    if vec.len() != 3{
        error_msg("server::merge::get_param_for_increase_size".to_string(), "invalid string format".to_string());
        return None;
    }
    Some((vec[1].parse::<i64>().unwrap(), vec[2].parse::<bool>().unwrap()))
}
fn get_param_for_decrease_size(op_s: &String) -> Option<i64>{
    let s = op_s.split('|');
    let vec = s.collect::<Vec<&str>>();
    if vec.len() != 2{
        error_msg("server::merge::get_param_for_decrease_size".to_string(), "invalid string format".to_string());
        return None;
    }
    Some(vec[1].parse::<i64>().unwrap())
}
pub fn full_merge(new_key: &[u8],
    existing_val: Option<&[u8]>,
    operands: &MergeOperands)
    -> Option<Vec<u8>> {
    let mut md = Metadata::new();
    if let Some(val) = existing_val{
        if let Ok(data) = Metadata::deserialize(&String::from_utf8((&val).to_vec()).unwrap()){
            md = data;
        }
        else{
            error_msg("server::merge::full_merge".to_string(), "given key has invalid existing value".to_string()); 
            return None;
        }
    }
    let mut iter = operands.into_iter();
    if md.get_size() == 0{
        if iter.size_hint().0 == 0 {
            error_msg("server::merge::full_merge".to_string(), "given key has no existing value and no operands available".to_string()); 
            return None;
        }
        let op_s = String::from_utf8(iter.next().unwrap().to_vec()).unwrap();
        match get_id_from_op(&op_s){
            OperandID::Create => {
                if let Ok(data) = Metadata::deserialize(&op_s){
                    md = data;
                }
                else{
                    error_msg("server::merge::full_merge".to_string(), "given key has no existing value and the first operand is not a valid create".to_string()); 
                    return None;
                }
            }
            _ => {
                error_msg("server::merge::full_merge".to_string(), "given key has no existing value and the first operand is not create".to_string()); 
                return None;
            }
        }
    }
    let mut fsize = md.get_size();
    let mut op;
    while {op = iter.next(); op != None}{
        let op_s = String::from_utf8(op.unwrap().to_vec()).unwrap();
        match get_id_from_op(&op_s) {
            OperandID::Create => {
                continue;
            },
            OperandID::IncreaseSize => {
                if let Some(params )= get_param_for_increase_size(&op_s){
                    let op_size = params.0;
                    let append = params.1;
                    if append{
                        fsize += op_size;
                    }
                    else{
                        fsize = std::cmp::max(fsize, op_size);
                    }
                }
                else{
                    return None;
                }
            },
            OperandID::DecreaseSize => {
                if let Some(op_size )= get_param_for_decrease_size(&op_s){
                    if op_size > fsize{
                        error_msg("server::merge::full_merge".to_string(), "you can't decrease file size to a bigger one".to_string()); 
                        return None
                    }
                    fsize = op_size
                }
                else{
                    return None;
                }
            },
            _ => {
                error_msg("server::merge::full_merge".to_string(), "unknown operands detected".to_string()); 
                return None
            },
        }
    } 
    md.set_size(fsize);
    Some(md.serialize().as_bytes().to_vec())
}
pub fn partial_merge(new_key: &[u8],
    existing_val: Option<&[u8]>,
    operands: &MergeOperands)
    -> Option<Vec<u8>> {
        None
}