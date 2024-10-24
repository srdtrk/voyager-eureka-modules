#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    //pub chain_id: ChainId<'static>,
    pub ws_url: String,
    pub grpc_url: String,

    pub prover_endpoints: Vec<String>,
}
#[tokio::main(flavor = "multi_thread")]
async fn main() {
    todo!();
}
