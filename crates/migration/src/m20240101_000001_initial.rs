use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Users table
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(string_len(Users::Email, 255).primary_key())
                    .col(string_len(Users::HashedPassword, 255).not_null())
                    .col(string_len_null(Users::FullName, 255))
                    .col(boolean(Users::Disabled).default(false).not_null())
                    .to_owned(),
            )
            .await?;

        // Jobs table
        manager
            .create_table(
                Table::create()
                    .table(Jobs::Table)
                    .if_not_exists()
                    .col(string_len(Jobs::Id, 64).primary_key())
                    .col(string_len(Jobs::Url, 2048).not_null())
                    .col(json(Jobs::Elements).not_null())
                    .col(string_len_null(Jobs::User, 255))
                    .col(
                        timestamp_with_time_zone(Jobs::TimeCreated)
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(json(Jobs::Result).not_null())
                    .col(string_len(Jobs::Status, 50).not_null())
                    .col(json_null(Jobs::Chat))
                    .col(json_null(Jobs::JobOptions))
                    .col(boolean(Jobs::AgentMode).default(false).not_null())
                    .col(string_len_null(Jobs::Prompt, 1024))
                    .col(boolean(Jobs::Favorite).default(false).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Jobs::Table, Jobs::User)
                            .to(Users::Table, Users::Email)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // Cron jobs table
        manager
            .create_table(
                Table::create()
                    .table(CronJobs::Table)
                    .if_not_exists()
                    .col(string_len(CronJobs::Id, 64).primary_key())
                    .col(string_len(CronJobs::UserEmail, 255).not_null())
                    .col(string_len(CronJobs::JobId, 64).not_null())
                    .col(string_len(CronJobs::CronExpression, 255).not_null())
                    .col(
                        timestamp_with_time_zone(CronJobs::TimeCreated)
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(
                        timestamp_with_time_zone(CronJobs::TimeUpdated)
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(CronJobs::Table, CronJobs::UserEmail)
                            .to(Users::Table, Users::Email)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(CronJobs::Table, CronJobs::JobId)
                            .to(Jobs::Table, Jobs::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Indexes for performance
        manager
            .create_index(
                Index::create()
                    .name("idx_jobs_user")
                    .table(Jobs::Table)
                    .col(Jobs::User)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_jobs_status")
                    .table(Jobs::Table)
                    .col(Jobs::Status)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_jobs_time_created")
                    .table(Jobs::Table)
                    .col(Jobs::TimeCreated)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_cron_jobs_user_email")
                    .table(CronJobs::Table)
                    .col(CronJobs::UserEmail)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CronJobs::Table).if_exists().to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Jobs::Table).if_exists().to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Users::Table).if_exists().to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Email,
    HashedPassword,
    FullName,
    Disabled,
}

#[derive(DeriveIden)]
enum Jobs {
    Table,
    Id,
    Url,
    Elements,
    User,
    TimeCreated,
    Result,
    Status,
    Chat,
    JobOptions,
    AgentMode,
    Prompt,
    Favorite,
}

#[derive(DeriveIden)]
enum CronJobs {
    Table,
    Id,
    UserEmail,
    JobId,
    CronExpression,
    TimeCreated,
    TimeUpdated,
}
