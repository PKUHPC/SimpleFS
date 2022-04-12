use futures::{SinkExt, TryStreamExt};
use grpcio::{ChannelBuilder, Environment, Error, WriteFlags};
use serde::Serialize;
use sfs_global::global::endpoint::SFSEndpoint;
use sfs_global::global::util::serde_util::serialize;
use sfs_rpc::proto::{
    server::{Post, PostResult},
    server_grpc::SfsHandleClient,
};
use std::sync::Arc;

use sfs_global::global::network::post::{option2i, post, PostOption};

pub struct NetworkService {}
impl NetworkService {
    #[tokio::main]
    pub async fn post<T: Serialize>(
        endp: &SFSEndpoint,
        data: T,
        opt: PostOption,
    ) -> Result<PostResult, Error> {
        let serialized_data = serialize(&data);
        let post = post(option2i(&opt), serialized_data, vec![0; 0]);
        let env = Arc::new(Environment::new(12));
        let channel = ChannelBuilder::new(env).connect(&format!("{}:{}", endp.addr, 8082));
        let client = SfsHandleClient::new(channel);

        let post_result = client.handle(&post)?;
        return Ok(post_result);
    }

    #[tokio::main]
    pub async fn group_post(posts: Vec<(SFSEndpoint, Post)>) -> Result<Vec<PostResult>, Error> {
        let mut post_results: Vec<PostResult> = Vec::new();
        for (endp, post) in posts {
            let env = Arc::new(Environment::new(12));
            let channel = ChannelBuilder::new(env).connect(&format!("{}:{}", endp.addr, 8082));
            let client = SfsHandleClient::new(channel);

            let post_result = client.handle(&post)?;
            post_results.push(post_result);
        }
        return Ok(post_results);
    }

    pub async fn post_stream(
        endp: &SFSEndpoint,
        posts: Vec<Post>,
    ) -> Result<Vec<PostResult>, Error> {
        let mut post_results = Vec::new();
        let env = Arc::new(Environment::new(12));
        let channel = ChannelBuilder::new(env).connect(&format!("{}:{}", endp.addr, 8082));
        let client = SfsHandleClient::new(channel);

        let (mut sink, mut receiver) = client.handle_stream()?;
        for post in posts {
            sink.send((post, WriteFlags::default())).await?;
        }
        sink.close().await?;
        while let Some(res) = receiver.try_next().await? {
            post_results.push(res);
        }
        return Ok(post_results);
    }
}
