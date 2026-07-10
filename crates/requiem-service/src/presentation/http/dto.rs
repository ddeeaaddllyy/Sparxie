//! HTTP-DTO RequiemProject.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

use crate::application::dto::ProfileView;

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProfileRequest {
    #[validate(email(message = "invalid email"))]
    pub email: Option<String>,
    #[validate(length(min = 1, max = 64, message = "display_name must be 1-64 characters"))]
    pub display_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProfileResponse {
    pub user_id: Uuid,
    pub email: Option<String>,
    pub display_name: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

impl From<ProfileView> for ProfileResponse {
    fn from(view: ProfileView) -> Self {
        Self {
            user_id: view.user_id,
            email: view.email,
            display_name: view.display_name,
            created_at: view.created_at,
        }
    }
}
