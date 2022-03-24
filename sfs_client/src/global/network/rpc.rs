#[tarpc::service]
pub trait SFSServer {
    async fn handle(post: String) -> String;
}
