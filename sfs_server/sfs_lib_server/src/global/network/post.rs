use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug)]
pub enum PostOption {
    Lookup,
    Stat,
    Create,
    Remove,
    Write,
    FsConfig
}
impl Clone for PostOption{
    fn clone(&self) -> Self {
        match self {
            Self::Lookup => Self::Lookup,
            Self::Stat => Self::Stat,
            Self::Create => Self::Create,
            Self::Remove => Self::Remove,
            Self::Write => Self::Write,
            Self::FsConfig => Self::FsConfig,
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
