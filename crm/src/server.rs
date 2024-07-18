use std::mem;

use crm::{AppConfig, CrmService};

use tonic::transport::{Identity, Server, ServerTlsConfig};
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{
    fmt::Layer, layer::SubscriberExt as _, util::SubscriberInitExt as _, Layer as _,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let mut config = AppConfig::load()?;
    let tls = mem::take(&mut config.server.tls);
    let addr = config.server.port;
    let addr = format!("[::1]:{}", addr).parse()?;
    let scv = CrmService::try_new(config).await?.into_server();

    info!("Crm Server listening on {}", addr);
    if let Some(tls) = tls {
        let identity = Identity::from_pem(tls.cert, tls.key);
        Server::builder()
            .tls_config(ServerTlsConfig::new().identity(identity))?
            .add_service(scv)
            .serve(addr)
            .await?;
    } else {
        Server::builder().add_service(scv).serve(addr).await?;
    }
    Ok(())
}
