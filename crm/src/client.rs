use crm::{
    pb::{crm_client::CrmClient, WelcomeRequestBuilder},
    AppConfig,
};
use tonic::{
    metadata::MetadataValue,
    transport::{Certificate, Channel, ClientTlsConfig},
    Request,
};
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::load().expect("Failed to load config");
    let addr = Box::leak(Box::new(format!("https://[::1]:{}", config.server.port)));
    println!("{}", addr);
    let pem = include_str!("../../fixtures/rootCA.pem");
    let tls = ClientTlsConfig::new()
        .ca_certificate(Certificate::from_pem(pem))
        .domain_name("localhost");
    let channel = Channel::from_static(addr)
        .tls_config(tls)?
        .connect()
        .await?;

    let token = include_str!("../../fixtures/token").trim();
    let token: MetadataValue<_> = format!("Bearer {}", token).parse()?;

    let mut client = CrmClient::with_interceptor(channel, move |mut req: Request<()>| {
        req.metadata_mut().insert("authorization", token.clone());
        Ok(req)
    });
    let req = WelcomeRequestBuilder::default()
        .id(Uuid::new_v4().to_string())
        .interval(9)
        .contents_ids([1, 2, 3])
        .build()?;
    let resp = client.welcome(Request::new(req)).await?;
    println!("Response: {:?}", resp);
    Ok(())
}
