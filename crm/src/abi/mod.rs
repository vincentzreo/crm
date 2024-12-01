use std::sync::Arc;
pub mod auth;

use chrono::{Duration, Utc};
use crm_metadata::pb::{Content, MaterializeRequest};
use crm_send::pb::SendRequest;
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Response, Status};
use tracing::warn;
use user_stat::pb::QueryRequest;

use crate::{
    pb::{
        RecallRequest, RecallResponse, RemindRequest, RemindResponse, WelcomeRequest,
        WelcomeResponse,
    },
    CrmService,
};

impl CrmService {
    pub async fn welcome(&self, req: WelcomeRequest) -> Result<Response<WelcomeResponse>, Status> {
        let req_id = req.id;
        let d1 = Utc::now() - Duration::days(req.interval as _);
        let d2 = d1 + Duration::days(1);
        let query = QueryRequest::new_with_dt("created_at", d1, d2);
        let mut res = self.user_stats.clone().query(query).await?.into_inner();

        let contents = self
            .metadata
            .clone()
            .materialize(MaterializeRequest::new_with_ids(&req.contents_ids))
            .await?
            .into_inner();
        let contents: Vec<Content> = contents
            .filter_map(|v| async move { v.ok() })
            .collect()
            .await;
        let contents = Arc::new(contents);
        let sender = self.config.server.sender_email.clone();

        let (tx, rx) = mpsc::channel(1024);
        tokio::spawn(async move {
            while let Some(Ok(v)) = res.next().await {
                let contents = contents.clone();
                let tx = tx.clone();
                let sender = sender.clone();
                let req = SendRequest::new("Welcome".to_string(), sender, &[v.email], &contents);
                if let Err(e) = tx.send(req).await {
                    warn!("Failed to send message: {}", e);
                }
            }
        });

        let reqs = ReceiverStream::new(rx);
        self.notification.clone().send(reqs).await?;
        Ok(Response::new(WelcomeResponse { id: req_id }))
    }

    pub async fn recall(&self, req: RecallRequest) -> Result<Response<RecallResponse>, Status> {
        let req_id = req.id;
        let d1 = Utc::now() - Duration::days(req.last_visit_interval as _);
        let d2 = d1 + Duration::days(1);
        let query = QueryRequest::new_with_dt("last_visit_at", d1, d2);
        let mut res = self.user_stats.clone().query(query).await?.into_inner();

        let contents = self
            .metadata
            .clone()
            .materialize(MaterializeRequest::new_with_ids(&req.contents_ids))
            .await?
            .into_inner();
        let contents: Vec<Content> = contents
            .filter_map(|v| async move { v.ok() })
            .collect()
            .await;
        let contents = Arc::new(contents);
        let sender = self.config.server.sender_email.clone();

        let (tx, rx) = mpsc::channel(1024);

        tokio::spawn(async move {
            while let Some(Ok(v)) = res.next().await {
                let contents = contents.clone();
                let tx = tx.clone();
                let sender = sender.clone();
                let req = SendRequest::new("Recall".to_string(), sender, &[v.email], &contents);
                if let Err(e) = tx.send(req).await {
                    warn!("Failed to send message: {}", e);
                }
            }
        });

        let reqs = ReceiverStream::new(rx);
        self.notification.clone().send(reqs).await?;
        Ok(Response::new(RecallResponse { id: req_id }))
    }

    pub async fn remind(&self, req: RemindRequest) -> Result<Response<RemindResponse>, Status> {
        let req_id = req.id;
        let d1 = Utc::now() - Duration::days(req.last_visit_interval as _);
        let d2 = d1 + Duration::days(1);
        let query = QueryRequest::new_with_dt("last_visit_at", d1, d2);
        let mut res = self.user_stats.clone().query(query).await?.into_inner();

        let sender = self.config.server.sender_email.clone();

        let (tx, rx) = mpsc::channel(1024);

        tokio::spawn(async move {
            while let Some(Ok(v)) = res.next().await {
                let sender = sender.clone();
                let tx = tx.clone();
                let req = SendRequest::new(
                    "You haven't visited us for a long time".to_string(),
                    sender,
                    &[v.email],
                    &[],
                );
                if let Err(e) = tx.send(req).await {
                    warn!("Failed to send message: {}", e);
                }
            }
        });

        let reqs = ReceiverStream::new(rx);
        self.notification.clone().send(reqs).await?;
        Ok(Response::new(RemindResponse { id: req_id }))
    }
}
