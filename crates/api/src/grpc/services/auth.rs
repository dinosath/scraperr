use tonic::{Request, Response, Status};

use crate::grpc::proto::auth::{
    auth_service_server::AuthService, CheckAuthRequest, CheckAuthResponse, GetMeRequest,
    UserResponse,
};

pub struct AuthServiceImpl {
    pub registration_enabled: bool,
    pub recordings_enabled: bool,
}

#[tonic::async_trait]
impl AuthService for AuthServiceImpl {
    async fn get_me(
        &self,
        request: Request<GetMeRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        let email = request
            .metadata()
            .get("x-user-email")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let full_name = request
            .metadata()
            .get("x-user-name")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        Ok(Response::new(UserResponse { email, full_name }))
    }

    async fn check_auth(
        &self,
        _request: Request<CheckAuthRequest>,
    ) -> Result<Response<CheckAuthResponse>, Status> {
        Ok(Response::new(CheckAuthResponse {
            registration_enabled: self.registration_enabled,
            recordings_enabled: self.recordings_enabled,
        }))
    }
}
