use tonic::{Request, Status};

/// gRPC auth interceptor that validates bearer tokens from the `authorization` metadata.
///
/// Follows the tonic authentication example pattern:
/// https://github.com/hyperium/tonic/tree/master/examples/src/authentication
pub fn check_auth(req: Request<()>) -> Result<Request<()>, Status> {
    match req.metadata().get("authorization") {
        Some(value) => {
            let token = value
                .to_str()
                .map_err(|_| Status::unauthenticated("Invalid authorization metadata"))?;

            if token.starts_with("Bearer ") && token.len() > 7 {
                Ok(req)
            } else {
                Err(Status::unauthenticated("Invalid bearer token format"))
            }
        }
        None => Err(Status::unauthenticated("Missing authorization token")),
    }
}
