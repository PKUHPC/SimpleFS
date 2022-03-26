use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
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
}
impl Clone for PostOption{
    fn clone(&self) -> Self {
        match self {
            Self::Lookup => Self::Lookup,
            Self::Stat => Self::Stat,
            Self::Create => Self::Create,
            Self::Remove => Self::Remove,
            Self::RemoveMeta => Self::RemoveMeta,
            Self::Write => Self::Write,
            Self::FsConfig => Self::FsConfig,
            Self::Read => Self::Read,
            Self::UpdateMetadentry => Self::UpdateMetadentry,
            Self::GetMetadentry => Self::GetMetadentry,
            Self::ChunkStat => Self::ChunkStat,
            Self::DecrSize => Self::DecrSize,
            Self::Trunc => Self::Trunc,
            Self::GetDirents => Self::GetDirents,
            Self::Unknown => Self::Unknown,
        }
    }
}

pub fn i2option(n: i32) -> PostOption{
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
        _ => PostOption::Unknown,
    }
}
pub fn option2i(option: PostOption) -> i32{
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
        PostOption::Unknown => -1,
    }
}
/*
#[derive(Serialize, Deserialize, Debug)]
pub struct Post{
    pub option: PostOption,
    pub data: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PostResult{
    pub err: bool,
    pub data: String
}
*/
