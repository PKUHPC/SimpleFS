use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug)]
pub enum PostOption {
    Stat,
    Create,
    Remove,
    Write
}
impl Clone for PostOption{
    fn clone(&self) -> Self {
        match self {
            Self::Stat => Self::Stat,
            Self::Create => Self::Create,
            Self::Remove => Self::Remove,
            Self::Write => Self::Write,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Post{
    pub option: PostOption,
    pub data: String
}