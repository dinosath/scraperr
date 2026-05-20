use sea_orm::*;

use crate::entities::user;
use crate::entities::user::Entity as User;

pub struct UserRepository;

impl UserRepository {
    pub async fn find_by_email(
        db: &DatabaseConnection,
        email: &str,
    ) -> Result<Option<user::Model>, DbErr> {
        User::find_by_id(email.to_owned()).one(db).await
    }

    pub async fn insert(
        db: &DatabaseConnection,
        model: user::ActiveModel,
    ) -> Result<user::Model, DbErr> {
        model.insert(db).await
    }

    pub async fn update(
        db: &DatabaseConnection,
        model: user::ActiveModel,
    ) -> Result<user::Model, DbErr> {
        model.update(db).await
    }
}
