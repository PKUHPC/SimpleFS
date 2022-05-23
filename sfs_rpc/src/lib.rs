use proto::server::{Post, PostResult};

pub mod proto;

pub fn post(option: i32, data: Vec<u8>, extra: Vec<u8>) -> Post{
    let mut res = Post::default();
    res.set_option(option);
    res.set_data(data);
    res.set_extra(extra);
    res
}
pub fn post_result(err: i32, data: Vec<u8>, extra: Vec<u8>) -> PostResult{
    let mut res = PostResult::default();
    res.set_err(err);
    res.set_data(data);
    res.set_extra(extra);
    res
}