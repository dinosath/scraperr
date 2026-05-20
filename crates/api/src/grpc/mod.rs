pub mod interceptor;
pub mod services;

pub mod proto {
    pub mod auth {
        tonic::include_proto!("scraperr.auth");
    }
    pub mod jobs {
        tonic::include_proto!("scraperr.jobs");
    }
}
