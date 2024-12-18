pub mod abi;
mod config;
pub mod pb;

use abi::auth;
pub use config::AppConfig;

use crm_metadata::pb::metadata_client::MetadataClient;
use crm_send::pb::notification_client::NotificationClient;

use pb::{
    crm_server::{Crm, CrmServer},
    RecallRequest, RecallResponse, RemindRequest, RemindResponse, WelcomeRequest, WelcomeResponse,
};
use tonic::{
    async_trait, service::interceptor::InterceptedService, transport::Channel, Request, Response,
    Status,
};

use tracing::info;
use user_stat::pb::user_stats_client::UserStatsClient;

pub struct CrmService {
    config: AppConfig,
    user_stats: UserStatsClient<Channel>,
    notification: NotificationClient<Channel>,
    metadata: MetadataClient<Channel>,
}

#[async_trait]
impl Crm for CrmService {
    async fn welcome(
        &self,
        request: Request<WelcomeRequest>,
    ) -> Result<Response<WelcomeResponse>, Status> {
        let user: &auth::User = request.extensions().get().unwrap();
        info!("User: {:?}", user);
        self.welcome(request.into_inner()).await
    }

    async fn recall(
        &self,
        request: Request<RecallRequest>,
    ) -> Result<tonic::Response<RecallResponse>, Status> {
        let user: &auth::User = request.extensions().get().unwrap();
        info!("User: {:?}", user);
        self.recall(request.into_inner()).await
    }

    async fn remind(
        &self,
        request: Request<RemindRequest>,
    ) -> Result<Response<RemindResponse>, Status> {
        let user: &auth::User = request.extensions().get().unwrap();
        info!("User: {:?}", user);
        self.remind(request.into_inner()).await
    }
}

impl CrmService {
    pub async fn try_new(config: AppConfig) -> anyhow::Result<Self> {
        let user_stats = UserStatsClient::connect(config.server.user_stats.clone()).await?;
        let notification = NotificationClient::connect(config.server.notification.clone()).await?;
        let metadata = MetadataClient::connect(config.server.metadata.clone()).await?;
        Ok(Self {
            config,
            user_stats,
            notification,
            metadata,
        })
    }
    pub fn into_server(
        self,
    ) -> anyhow::Result<InterceptedService<CrmServer<CrmService>, auth::DecodingKey>> {
        let dk = auth::DecodingKey::load(&self.config.auth.pk)?;
        Ok(CrmServer::with_interceptor(self, dk))
    }
}
