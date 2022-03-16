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
            Self::GetDirents => Self::GetDirents
        }
    }
}

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
