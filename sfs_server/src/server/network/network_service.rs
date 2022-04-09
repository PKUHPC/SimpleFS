use serde::Serialize;
use sfs_global::global::endpoint::SFSEndpoint;
use sfs_global::global::util::serde_util::serialize;
use std::io::Error;

use sfs_global::global::network::post::{option2i, PostOption};
use sfs_rpc::sfs_server::sfs_handle_client::SfsHandleClient;
use sfs_rpc::sfs_server::{Post, PostResult};

pub struct NetworkService {}
impl NetworkService {
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
}
