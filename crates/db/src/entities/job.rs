use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "jobs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[sea_orm(column_type = "String(StringLen::N(2048))")]
    pub url: String,
    #[sea_orm(column_type = "Json")]
    pub elements: serde_json::Value,
    pub user: Option<String>,
    pub time_created: DateTimeWithTimeZone,
    #[sea_orm(column_type = "Json")]
    pub result: serde_json::Value,
    #[sea_orm(column_type = "String(StringLen::N(50))")]
    pub status: String,
    #[sea_orm(column_type = "Json", nullable)]
    pub chat: Option<serde_json::Value>,
    #[sea_orm(column_type = "Json", nullable)]
    pub job_options: Option<serde_json::Value>,
    #[sea_orm(default_value = "false")]
    pub agent_mode: bool,
    #[sea_orm(column_type = "String(StringLen::N(1024))", nullable)]
    pub prompt: Option<String>,
    #[sea_orm(default_value = "false")]
    pub favorite: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::User",
        to = "super::user::Column::Email"
    )]
    User,
    #[sea_orm(has_many = "super::cron_job::Entity")]
    CronJobs,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::cron_job::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CronJobs.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
