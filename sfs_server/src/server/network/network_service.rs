use grpcio::{ChannelBuilder, Environment};
use serde::Serialize;
use sfs_global::global::{endpoint::SFSEndpoint, network::post::post};
use sfs_global::global::util::serde_util::serialize;
use sfs_rpc::proto::{server::{PostResult}, server_grpc::SfsHandleClient};
use std::{io::Error, sync::Arc};

use sfs_global::global::network::post::{option2i, PostOption};

pub struct NetworkService {}
impl NetworkService {
    pub async fn post<T: Serialize>(
        endp: &SFSEndpoint,
        data: T,
        opt: PostOption,
    ) -> Result<PostResult, Error> {
        let serialized_data = serialize(&data);
        let post = post(
            option2i(&opt),
            serialized_data,
            vec![0; 0],
        );
        let env = Arc::new(Environment::new(12));
        let channel = ChannelBuilder::new(env).connect(&format!("{}:{}", endp.addr, 8082));
        let client = SfsHandleClient::new(channel);
        
        let post_result = client.handle(&post);
        if let Err(e) = post_result {
            return Err(Error::new(std::io::ErrorKind::NotConnected, e.to_string()));
        }
        let response = post_result.unwrap();
        return Ok(response);
    }
}
