use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub email: String,
    pub hashed_password: String,
    pub full_name: Option<String>,
    #[sea_orm(default_value = "false")]
    pub disabled: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::job::Entity")]
    Jobs,
    #[sea_orm(has_many = "super::cron_job::Entity")]
    CronJobs,
}

impl Related<super::job::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Jobs.def()
    }
}

impl Related<super::cron_job::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CronJobs.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
