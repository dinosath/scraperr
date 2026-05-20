use sea_orm::DatabaseConnection;
use thiserror::Error;
use tracing::info;

use crate::domain::user::User;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("User not found")]
    UserNotFound,
    #[error("User is disabled")]
    UserDisabled,
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
}

pub struct AuthService;

impl AuthService {
    /// Look up or provision a user from OIDC claims.
    /// With OIDC, the IdP handles authentication. This method ensures the user
    /// exists in our database (provisioning on first login).
    pub async fn ensure_user(
        db: &DatabaseConnection,
        email: &str,
        full_name: Option<&str>,
    ) -> Result<User, AuthError> {
        use scraperr_db::entities::user;
        use scraperr_db::repositories::user::UserRepository;

        let existing = UserRepository::find_by_email(db, email).await?;

        match existing {
            Some(u) => {
                if u.disabled {
                    return Err(AuthError::UserDisabled);
                }
                Ok(User {
                    email: u.email,
                    full_name: u.full_name,
                    disabled: u.disabled,
                })
            }
            None => {
                info!("Provisioning new user from OIDC: {}", email);
                use sea_orm::Set;
                let model = user::ActiveModel {
                    email: Set(email.to_string()),
                    hashed_password: Set(String::new()), // OIDC users don't have local passwords
                    full_name: Set(full_name.map(|s| s.to_string())),
                    disabled: Set(false),
                };
                let created = UserRepository::insert(db, model).await?;
                Ok(User {
                    email: created.email,
                    full_name: created.full_name,
                    disabled: created.disabled,
                })
            }
        }
    }
}
