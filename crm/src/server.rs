use crm::pb::{
    user_service_server::{UserService, UserServiceServer},
    CreateUserRequest, GetUserRequset, User,
};
use tonic::{async_trait, transport::Server, Request, Response, Status};

#[derive(Default)]
pub struct UserServer;

#[async_trait]
impl UserService for UserServer {
    async fn get_user(&self, request: Request<GetUserRequset>) -> Result<Response<User>, Status> {
        let input = request.into_inner();
        println!("get_user: {:?}", input);
        Ok(Response::new(User::default()))
    }
    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<User>, Status> {
        let input = request.into_inner();
        println!("create_user: {:?}", input);
        let user = User::new(1, &input.name, &input.email);
        Ok(Response::new(user))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = "[::1]:50051".parse()?;
    let scv = UserServer;

    println!("Server listening on {}", addr);
    Server::builder()
        .add_service(UserServiceServer::new(scv))
        .serve(addr)
        .await?;
    Ok(())
}
