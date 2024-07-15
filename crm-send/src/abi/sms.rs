use tonic::Status;
use tracing::warn;

use crate::{
    pb::{send_request::Msg, SendRequest, SendResponse, SmsMessage},
    NotificationService,
};

use super::{to_ts, Sender};

impl Sender for SmsMessage {
    async fn send(self, svc: NotificationService) -> Result<SendResponse, Status> {
        let message_id = self.message_id.clone();
        svc.sender.send(Msg::Sms(self)).await.map_err(|e| {
            warn!("Failed to send message: {:?}", e);
            Status::internal("Failed to send sms mseeage")
        })?;
        Ok(SendResponse {
            message_id,
            timestamp: Some(to_ts()),
        })
    }
}

impl From<SmsMessage> for Msg {
    fn from(value: SmsMessage) -> Self {
        Msg::Sms(value)
    }
}

impl From<SmsMessage> for SendRequest {
    fn from(value: SmsMessage) -> Self {
        let msg: Msg = value.into();
        Self { msg: Some(msg) }
    }
}

#[cfg(feature = "test_utils")]
impl SmsMessage {
    pub fn fake() -> Self {
        use fake::{faker::phone_number::en::PhoneNumber, Fake};
        use uuid::Uuid;
        Self {
            message_id: Uuid::new_v4().to_string(),
            sender: PhoneNumber().fake(),
            recipients: vec![PhoneNumber().fake()],
            body: "Hello, world!".to_string(),
        }
    }
}
