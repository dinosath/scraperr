use sea_orm::*;

use crate::entities::cron_job;
use crate::entities::cron_job::Entity as CronJob;

pub struct CronJobRepository;

impl CronJobRepository {
    pub async fn find_by_id(
        db: &DatabaseConnection,
        id: &str,
    ) -> Result<Option<cron_job::Model>, DbErr> {
        CronJob::find_by_id(id.to_owned()).one(db).await
    }

    pub async fn find_by_user(
        db: &DatabaseConnection,
        email: &str,
    ) -> Result<Vec<cron_job::Model>, DbErr> {
        CronJob::find()
            .filter(cron_job::Column::UserEmail.eq(email))
            .all(db)
            .await
    }

    pub async fn find_all(db: &DatabaseConnection) -> Result<Vec<cron_job::Model>, DbErr> {
        CronJob::find().all(db).await
    }

    pub async fn insert(
        db: &DatabaseConnection,
        model: cron_job::ActiveModel,
    ) -> Result<cron_job::Model, DbErr> {
        model.insert(db).await
    }

    pub async fn delete(
        db: &DatabaseConnection,
        id: &str,
        user_email: &str,
    ) -> Result<DeleteResult, DbErr> {
        CronJob::delete_many()
            .filter(cron_job::Column::Id.eq(id))
            .filter(cron_job::Column::UserEmail.eq(user_email))
            .exec(db)
            .await
    }
}
