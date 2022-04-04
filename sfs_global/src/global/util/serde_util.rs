use serde::{Serialize, Deserialize};


pub fn serialize<T: Serialize>(data: T) -> Vec<u8>{
    let mut buf = flexbuffers::FlexbufferSerializer::new();
    data.serialize(&mut buf).unwrap();
    return buf.view().to_vec();
}

pub fn deserialize<'a, T: Deserialize<'a>>(data: &'a Vec<u8>) -> T{
    let reader = flexbuffers::Reader::get_root(&data as &[u8]).unwrap();
    let data = T::deserialize(reader).unwrap();
    return data;
}