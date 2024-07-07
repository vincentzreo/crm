use crm::pb::{user_service_client::UserServiceClient, CreateUserRequest};
use tonic::Request;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = UserServiceClient::connect("http://[::1]:50051").await?;

    let request = Request::new(CreateUserRequest {
        name: "Alice".to_string(),
        email: "alice@acme.com".to_string(),
    });

    let response = client.create_user(request).await?;
    println!("Response: {:?}", response);
    Ok(())
}
