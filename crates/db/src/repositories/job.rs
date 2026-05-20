use sea_orm::*;
use sea_orm::prelude::Expr;

use crate::entities::job;
use crate::entities::job::Entity as Job;

pub struct JobRepository;

impl JobRepository {
    pub async fn find_by_id(
        db: &DatabaseConnection,
        id: &str,
    ) -> Result<Option<job::Model>, DbErr> {
        Job::find_by_id(id.to_owned()).one(db).await
    }

    pub async fn find_by_ids(
        db: &DatabaseConnection,
        ids: &[String],
    ) -> Result<Vec<job::Model>, DbErr> {
        Job::find()
            .filter(job::Column::Id.is_in(ids.to_vec()))
            .all(db)
            .await
    }

    pub async fn find_by_user(
        db: &DatabaseConnection,
        email: &str,
    ) -> Result<Vec<job::Model>, DbErr> {
        Job::find()
            .filter(job::Column::User.eq(email))
            .order_by_desc(job::Column::TimeCreated)
            .all(db)
            .await
    }

    pub async fn find_queued(db: &DatabaseConnection) -> Result<Option<job::Model>, DbErr> {
        Job::find()
            .filter(job::Column::Status.eq("Queued"))
            .order_by_desc(job::Column::TimeCreated)
            .one(db)
            .await
    }

    pub async fn insert(
        db: &DatabaseConnection,
        model: job::ActiveModel,
    ) -> Result<job::Model, DbErr> {
        model.insert(db).await
    }

    pub async fn update_status(
        db: &DatabaseConnection,
        ids: &[String],
        status: &str,
    ) -> Result<UpdateResult, DbErr> {
        Job::update_many()
            .filter(job::Column::Id.is_in(ids.to_vec()))
            .col_expr(job::Column::Status, Expr::value(status))
            .exec(db)
            .await
    }

    pub async fn update_result(
        db: &DatabaseConnection,
        id: &str,
        result: serde_json::Value,
    ) -> Result<UpdateResult, DbErr> {
        Job::update_many()
            .filter(job::Column::Id.eq(id))
            .col_expr(job::Column::Result, Expr::value(result))
            .exec(db)
            .await
    }

    pub async fn update_favorite(
        db: &DatabaseConnection,
        ids: &[String],
        favorite: bool,
    ) -> Result<UpdateResult, DbErr> {
        Job::update_many()
            .filter(job::Column::Id.is_in(ids.to_vec()))
            .col_expr(job::Column::Favorite, Expr::value(favorite))
            .exec(db)
            .await
    }

    pub async fn update_chat(
        db: &DatabaseConnection,
        id: &str,
        chat: serde_json::Value,
    ) -> Result<UpdateResult, DbErr> {
        Job::update_many()
            .filter(job::Column::Id.eq(id))
            .col_expr(job::Column::Chat, Expr::value(chat))
            .exec(db)
            .await
    }

    pub async fn delete_by_ids(
        db: &DatabaseConnection,
        ids: &[String],
    ) -> Result<DeleteResult, DbErr> {
        Job::delete_many()
            .filter(job::Column::Id.is_in(ids.to_vec()))
            .exec(db)
            .await
    }

    pub async fn count_by_user(db: &DatabaseConnection, email: &str) -> Result<u64, DbErr> {
        Job::find()
            .filter(job::Column::User.eq(email))
            .count(db)
            .await
    }
}
