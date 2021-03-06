use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PostOption {
    Lookup,
    Stat,
    Create,
    RemoveMeta,
    Remove,
    Write,
    FsConfig,
    Read,
    UpdateMetadentry,
    GetMetadentry,
    ChunkStat,
    DecrSize,
    Trunc,
    GetDirents,
    Unknown,
    WriteData,
    ReadData,
    PreCreate
}
pub fn i2option(n: i32) -> PostOption {
    match n {
        0 => PostOption::Lookup,
        1 => PostOption::Stat,
        2 => PostOption::Create,
        3 => PostOption::RemoveMeta,
        4 => PostOption::Remove,
        5 => PostOption::Write,
        6 => PostOption::FsConfig,
        7 => PostOption::Read,
        8 => PostOption::UpdateMetadentry,
        9 => PostOption::GetMetadentry,
        10 => PostOption::ChunkStat,
        11 => PostOption::DecrSize,
        12 => PostOption::Trunc,
        13 => PostOption::GetDirents,
        14 => PostOption::WriteData,
        15 => PostOption::ReadData,
        16 => PostOption::PreCreate,
        _ => PostOption::Unknown,
    }
}
pub fn option2i(option: &PostOption) -> i32 {
    match option {
        PostOption::Lookup => 0,
        PostOption::Stat => 1,
        PostOption::Create => 2,
        PostOption::RemoveMeta => 3,
        PostOption::Remove => 4,
        PostOption::Write => 5,
        PostOption::FsConfig => 6,
        PostOption::Read => 7,
        PostOption::UpdateMetadentry => 8,
        PostOption::GetMetadentry => 9,
        PostOption::ChunkStat => 10,
        PostOption::DecrSize => 11,
        PostOption::Trunc => 12,
        PostOption::GetDirents => 13,
        PostOption::WriteData => 14,
        PostOption::ReadData => 15,
        PostOption::PreCreate => 16,
        PostOption::Unknown => -1,
    }
}