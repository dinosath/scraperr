use sea_orm::{DatabaseConnection, EntityTrait};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use scraperr_db::entities::job;
use scraperr_db::repositories::job::JobRepository;

use crate::grpc::proto::jobs::{
    job_service_server::JobService, DeleteJobsRequest, DeleteJobsResponse, GetJobRequest,
    JobResponse, JobStatusUpdate, ListJobsRequest, ListJobsResponse, SubmitJobRequest,
    SubmitJobResponse, UpdateJobsRequest, UpdateJobsResponse,
};

pub struct JobServiceImpl {
    pub db: DatabaseConnection,
}

#[tonic::async_trait]
impl JobService for JobServiceImpl {
    async fn submit_job(
        &self,
        request: Request<SubmitJobRequest>,
    ) -> Result<Response<SubmitJobResponse>, Status> {
        let req = request.into_inner();
        let id = req.id.unwrap_or_else(|| Uuid::new_v4().to_string());

        let elements: serde_json::Value =
            serde_json::from_str(&req.elements_json).map_err(|e| {
                Status::invalid_argument(format!("Invalid elements JSON: {e}"))
            })?;

        let job_options: serde_json::Value =
            serde_json::from_str(&req.job_options_json).map_err(|e| {
                Status::invalid_argument(format!("Invalid job_options JSON: {e}"))
            })?;

        let model = job::ActiveModel {
            id: sea_orm::ActiveValue::Set(id.clone()),
            url: sea_orm::ActiveValue::Set(req.url),
            elements: sea_orm::ActiveValue::Set(elements),
            status: sea_orm::ActiveValue::Set("pending".to_string()),
            result: sea_orm::ActiveValue::Set(serde_json::json!([])),
            time_created: sea_orm::ActiveValue::Set(chrono::Utc::now().into()),
            agent_mode: sea_orm::ActiveValue::Set(req.agent_mode),
            prompt: sea_orm::ActiveValue::Set(req.prompt),
            favorite: sea_orm::ActiveValue::Set(false),
            job_options: sea_orm::ActiveValue::Set(Some(job_options)),
            chat: sea_orm::ActiveValue::NotSet,
            user: sea_orm::ActiveValue::NotSet,
        };

        JobRepository::insert(&self.db, model)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(SubmitJobResponse {
            id,
            message: "Job submitted".to_string(),
        }))
    }

    async fn get_job(
        &self,
        request: Request<GetJobRequest>,
    ) -> Result<Response<JobResponse>, Status> {
        let id = request.into_inner().id;
        let job = JobRepository::find_by_id(&self.db, &id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Job not found"))?;

        Ok(Response::new(model_to_response(job)))
    }

    async fn list_jobs(
        &self,
        _request: Request<ListJobsRequest>,
    ) -> Result<Response<ListJobsResponse>, Status> {
        let jobs: Vec<job::Model> = job::Entity::find()
            .all(&self.db)
            .await
            .map_err(|e: sea_orm::DbErr| Status::internal(e.to_string()))?;

        Ok(Response::new(ListJobsResponse {
            jobs: jobs.into_iter().map(model_to_response).collect(),
        }))
    }

    async fn delete_jobs(
        &self,
        request: Request<DeleteJobsRequest>,
    ) -> Result<Response<DeleteJobsResponse>, Status> {
        let ids = request.into_inner().ids;
        JobRepository::delete_by_ids(&self.db, &ids)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(DeleteJobsResponse {
            message: "Deleted".to_string(),
        }))
    }

    async fn update_jobs(
        &self,
        request: Request<UpdateJobsRequest>,
    ) -> Result<Response<UpdateJobsResponse>, Status> {
        let req = request.into_inner();

        match req.field.as_str() {
            "status" => {
                let value = req.value_json.trim_matches('"');
                JobRepository::update_status(&self.db, &req.ids, value)
                    .await
                    .map_err(|e| Status::internal(e.to_string()))?;
            }
            "favorite" => {
                let value: bool = serde_json::from_str(&req.value_json)
                    .map_err(|e| Status::invalid_argument(format!("Invalid bool: {e}")))?;
                JobRepository::update_favorite(&self.db, &req.ids, value)
                    .await
                    .map_err(|e| Status::internal(e.to_string()))?;
            }
            _ => return Err(Status::invalid_argument(format!("Unknown field: {}", req.field))),
        }

        Ok(Response::new(UpdateJobsResponse {
            message: "Updated".to_string(),
        }))
    }

    type StreamJobStatusStream =
        tokio_stream::wrappers::ReceiverStream<Result<JobStatusUpdate, Status>>;

    async fn stream_job_status(
        &self,
        request: Request<GetJobRequest>,
    ) -> Result<Response<Self::StreamJobStatusStream>, Status> {
        let id = request.into_inner().id;
        let db = self.db.clone();

        let (tx, rx) = tokio::sync::mpsc::channel(16);

        tokio::spawn(async move {
            loop {
                let job = JobRepository::find_by_id(&db, &id).await;
                match job {
                    Ok(Some(j)) => {
                        let done = j.status == "completed" || j.status == "error";
                        let result_json = if done {
                            Some(j.result.to_string())
                        } else {
                            None
                        };
                        let _ = tx
                            .send(Ok(JobStatusUpdate {
                                id: j.id.clone(),
                                status: j.status.clone(),
                                result_json,
                            }))
                            .await;
                        if done {
                            break;
                        }
                    }
                    _ => {
                        let _ = tx
                            .send(Err(Status::internal("Failed to fetch job")))
                            .await;
                        break;
                    }
                }
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        });

        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(
            rx,
        )))
    }
}

fn model_to_response(j: job::Model) -> JobResponse {
    use prost_types::Timestamp;

    let seconds = j.time_created.timestamp();
    let nanos = j.time_created.timestamp_subsec_nanos() as i32;

    JobResponse {
        id: j.id,
        url: j.url,
        elements_json: j.elements.to_string(),
        status: j.status,
        result_json: j.result.to_string(),
        time_created: Some(Timestamp { seconds, nanos }),
        agent_mode: j.agent_mode,
        prompt: j.prompt,
        favorite: j.favorite,
    }
}
