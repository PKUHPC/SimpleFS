use futures::stream::iter;
use lazy_static::*;
use serde::Serialize;
use std::{
    collections::{hash_map::Entry, HashMap},
    io::Error,
};

use crate::{
    client::endpoint::SFSEndpoint,
    global::network::post::{option2i, PostOption},
};
use sfs_rpc::sfs_server::sfs_handle_client::SfsHandleClient;
use sfs_rpc::sfs_server::{Post, PostResult};

pub struct NetworkService {}
lazy_static! {
    static ref NTS: NetworkService = NetworkService {};
}
impl NetworkService {
    #[tokio::main]
    pub async fn post<T: Serialize>(
        endp: &SFSEndpoint,
        data: T,
        opt: PostOption,
    ) -> Result<PostResult, Error> {
        let serialized_data = serde_json::to_string(&data)?;
        let post = Post {
            option: option2i(opt),
            data: serialized_data,
        };
        let mut client = SfsHandleClient::connect(format!("http://{}:{}", endp.addr, 8082))
            .await
            .unwrap();
        let request = tonic::Request::new(iter(vec![post]));
        let post_result = client.handle(request).await;
        if let Err(e) = post_result {
            return Err(Error::new(std::io::ErrorKind::NotConnected, e.to_string()));
        }
        let mut response = post_result.unwrap().into_inner();
        return Ok(response.message().await.unwrap().unwrap());
    }

    #[tokio::main]
    pub async fn group_post(posts: Vec<(SFSEndpoint, Post)>) -> Result<Vec<PostResult>, Error> {
        let mut post_results: Vec<PostResult> = Vec::new();
        let mut post_map: HashMap<SFSEndpoint, Vec<Post>> = HashMap::new();
        for (endp, post) in posts {
            match post_map.entry(endp) {
                Entry::Occupied(mut e) => {
                    e.get_mut().push(post);
                }
                Entry::Vacant(e) => {
                    e.insert(vec![post]);
                }
            }
        }
        for (endp, posts) in post_map {
            let mut client = SfsHandleClient::connect(format!("http://{}:{}", endp.addr, 8082))
                .await
                .unwrap();
            let request = tonic::Request::new(iter(posts));
            let post_result = client.handle(request).await;
            if let Err(e) = post_result {
                return Err(Error::new(std::io::ErrorKind::NotConnected, e.to_string()));
            }
            let mut response = post_result.unwrap().into_inner();
            post_results.push(response.message().await.unwrap().unwrap());
        }
        return Ok(post_results);
    }
}
