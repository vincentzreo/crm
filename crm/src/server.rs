use crm::{AppConfig, CrmService};

use tonic::transport::Server;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{
    fmt::Layer, layer::SubscriberExt as _, util::SubscriberInitExt as _, Layer as _,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let config = AppConfig::load()?;
    let addr = config.server.port;
    let addr = format!("[::1]:{}", addr).parse()?;
    let scv = CrmService::try_new(config).await?.into_server();

    info!("Crm Server listening on {}", addr);
    Server::builder().add_service(scv).serve(addr).await?;
    Ok(())
}
