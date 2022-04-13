use grpcio::Error;
use serde::Serialize;
use sfs_global::global::util::serde_util::serialize;
use sfs_global::global::{network::post::post};
use sfs_rpc::proto::{server::PostResult, server_grpc::SfsHandleClient};

use sfs_global::global::network::post::{option2i, PostOption};

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
}
