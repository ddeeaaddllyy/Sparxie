//! Прикладные команды и представления.

use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::RequiemProfile;

/// Частичное обновление профиля.
#[derive(Debug)]
pub struct UpdateProfileCommand {
    pub user_id: Uuid,
    pub email: Option<String>,
    pub display_name: Option<String>,
}

/// Представление профиля для API.
#[derive(Debug, Clone)]
pub struct ProfileView {
    pub user_id: Uuid,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub created_at: OffsetDateTime,
}

impl From<&RequiemProfile> for ProfileView {
    fn from(p: &RequiemProfile) -> Self {
        Self {
            user_id: p.user_id(),
            email: p.email().map(|e| e.as_str().to_owned()),
            display_name: p.display_name().map(|d| d.as_str().to_owned()),
            created_at: p.created_at(),
        }
    }
}
