use crm::{
    pb::{crm_client::CrmClient, WelcomeRequestBuilder},
    AppConfig,
};
use tonic::Request;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::load().expect("Failed to load config");
    let addr = format!("http://[::1]:{}", config.server.port);
    println!("{}", addr);
    let mut client = CrmClient::connect(addr).await?;
    let req = WelcomeRequestBuilder::default()
        .id(Uuid::new_v4().to_string())
        .interval(9)
        .contents_ids([1, 2, 3])
        .build()?;
    let resp = client.welcome(Request::new(req)).await?;
    println!("Response: {:?}", resp);
    Ok(())
}
