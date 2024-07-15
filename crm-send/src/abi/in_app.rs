use tonic::Status;
use tracing::warn;

use crate::{
    pb::{send_request::Msg, InAppMessage, SendRequest, SendResponse},
    NotificationService,
};

use super::{to_ts, Sender};

impl Sender for InAppMessage {
    async fn send(self, svc: NotificationService) -> Result<SendResponse, Status> {
        let message_id = self.message_id.clone();
        svc.sender.send(Msg::InApp(self)).await.map_err(|e| {
            warn!("Failed to send message: {:?}", e);
            Status::internal("Failed to send in-app mseeage")
        })?;
        Ok(SendResponse {
            message_id,
            timestamp: Some(to_ts()),
        })
    }
}

impl From<InAppMessage> for Msg {
    fn from(value: InAppMessage) -> Self {
        Msg::InApp(value)
    }
}

impl From<InAppMessage> for SendRequest {
    fn from(value: InAppMessage) -> Self {
        let msg: Msg = value.into();
        Self { msg: Some(msg) }
    }
}

#[cfg(feature = "test_utils")]
impl InAppMessage {
    pub fn fake() -> Self {
        use uuid::Uuid;
        Self {
            message_id: Uuid::new_v4().to_string(),
            derive_id: Uuid::new_v4().to_string(),
            title: "Hello".to_string(),
            body: "Hello, world!".to_string(),
        }
    }
}
