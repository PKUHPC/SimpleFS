use futures::stream::iter;
use serde::Serialize;
use sfs_global::global::endpoint::SFSEndpoint;
use sfs_global::global::util::serde_util::serialize;
use std::io::Error;

use sfs_global::global::network::post::{option2i, PostOption};
use sfs_rpc::sfs_server::sfs_handle_client::SfsHandleClient;
use sfs_rpc::sfs_server::{Post, PostResult};

pub struct NetworkService {}
impl NetworkService {
    #[tokio::main]
    pub async fn post<T: Serialize>(
        endp: &SFSEndpoint,
        data: T,
        opt: PostOption,
    ) -> Result<PostResult, Error> {
        let serialized_data = serialize(&data);
        let post = Post {
            option: option2i(&opt),
            data: serialized_data,
            extra: vec![0; 0],
        };
        let mut client = SfsHandleClient::connect(format!("http://{}:{}", endp.addr, 8082))
            .await
            .unwrap();
        let request = tonic::Request::new(post);
        let post_result = client.handle(request).await;
        if let Err(e) = post_result {
            return Err(Error::new(std::io::ErrorKind::NotConnected, e.to_string()));
        }
        let response = post_result.unwrap().into_inner();
        return Ok(response);
    }

    #[tokio::main]
    pub async fn group_post(posts: Vec<(SFSEndpoint, Post)>) -> Result<Vec<PostResult>, Error> {
        let mut post_results: Vec<PostResult> = Vec::new();
        for (endp, post) in posts {
            let mut client = SfsHandleClient::connect(format!("http://{}:{}", endp.addr, 8082))
                .await
                .unwrap();
            let request = tonic::Request::new(post);
            let post_result = client.handle(request).await;
            if let Err(e) = post_result {
                return Err(Error::new(std::io::ErrorKind::NotConnected, e.to_string()));
            }
            let response = post_result.unwrap().into_inner();
            post_results.push(response);
        }
        return Ok(post_results);
    }

    #[tokio::main]
    pub async fn post_stream<T: Serialize>(
        endp: &SFSEndpoint,
        data: Vec<T>,
        opt: PostOption,
    ) -> Result<Vec<PostResult>, Error> {
        let mut post_results: Vec<PostResult> = Vec::new();
        let mut client = SfsHandleClient::connect(format!("http://{}:{}", endp.addr, 8082))
            .await
            .unwrap();
        let posts = data
            .iter()
            .map(|x| Post {
                option: option2i(&opt),
                data: serialize(&x),
                extra: vec![0; 0],
            })
            .collect::<Vec<_>>();
        let request = tonic::Request::new(iter(posts));
        let post_result = client.handle_stream(request).await;
        if let Err(e) = post_result {
            return Err(Error::new(std::io::ErrorKind::NotConnected, e.to_string()));
        }
        let mut response = post_result.unwrap().into_inner();
        while let Some(res) = response.message().await.unwrap() {
            post_results.push(res);
        }
        return Ok(post_results);
    }
}
