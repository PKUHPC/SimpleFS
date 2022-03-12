use lazy_static::*;
use serde::{Serialize, Deserialize};
use tarpc::{tokio_serde::formats::Json, client::{self}, context};
use std::{sync::{Mutex, MutexGuard}, net::{IpAddr}, io::{Error}};

use crate::{client::client_endpoint::SFSEndpoint, global::network::{post::{PostOption, Post, PostResult}, rpc::SFSServerClient}};

pub struct NetworkService{

}
lazy_static!{
    static ref NTS: NetworkService = NetworkService{};
}
impl NetworkService{
    #[tokio::main]
    pub async fn post<T: Serialize>(endp: &SFSEndpoint, data: T, opt: PostOption) -> Result<PostResult, Error>{
        let serialized_data = serde_json::to_string(&data)?;
        let post = Post{
            option: opt.clone(),
            data: serialized_data
        };
        let buf = serde_json::to_string(&post)?;

        let addr = (IpAddr::V4(endp.addr.as_str().parse().expect("fail to parse endpoint address")), 8082);
        let transport = tarpc::serde_transport::tcp::connect(&addr, Json::default);
        let client = SFSServerClient::new(client::Config::default(), transport.await?).spawn();
        
        let post_result = async move{
            tokio::select! {
                res1 = client.handle(context::current(),  buf.clone()) => {res1}
            }
        }.await;
        if let Err(e) = post_result{
            return Err(Error::new(std::io::ErrorKind::NotConnected, e.to_string()));
        }
        let result = post_result.unwrap();
        return Ok(serde_json::from_str(&result).unwrap());

    }
}