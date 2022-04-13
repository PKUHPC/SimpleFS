use futures::{join, SinkExt, TryStreamExt};
use grpcio::{Error, WriteFlags};
use serde::Serialize;
use sfs_global::global::util::serde_util::serialize;
use sfs_rpc::proto::{
    server::{Post, PostResult},
    server_grpc::SfsHandleClient,
};

use sfs_global::global::network::post::{option2i, post, PostOption};

pub struct NetworkService {}
impl NetworkService {
    pub fn post<T: Serialize>(
        client: &SfsHandleClient,
        data: T,
        opt: PostOption,
    ) -> Result<PostResult, Error> {
        let serialized_data = serialize(&data);
        let post = post(option2i(&opt), serialized_data, vec![0; 0]);
        let post_result = client.handle(&post)?;
        return Ok(post_result);
    }

    pub fn group_post(posts: Vec<(&SfsHandleClient, Post)>) -> Result<Vec<PostResult>, Error> {
        let mut post_results: Vec<PostResult> = Vec::new();
        for (client, post) in posts {
            let post_result = client.handle(&post)?;
            post_results.push(post_result);
        }
        return Ok(post_results);
    }

    pub async fn post_stream(
        client: &SfsHandleClient,
        posts: Vec<Post>,
    ) -> Result<Vec<PostResult>, Error> {
        let (mut sink, mut receiver) = client.handle_stream()?;
        let send = async move {
            for post in posts {
                sink.send((post, WriteFlags::default())).await?;
            }
            sink.close().await?;
            Ok(()) as Result<_, Error>
        };
        let receive = async move {
            let mut post_results = Vec::new();
            while let Some(res) = receiver.try_next().await? {
                post_results.push(res);
            }
            Ok(post_results) as Result<_, Error>
        };
        let (sr, rr) = join!(send, receive);
        sr.and(rr)
    }
}
