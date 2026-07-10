//! Сущность `RequiemProfile`.

use time::OffsetDateTime;
use uuid::Uuid;

use super::value_objects::{DisplayName, Email};

/// Профиль пользователя в RequiemProject.
///
/// Создаётся «пустым» по событию `UserRegistered` (известен только `user_id`);
/// `email`/`display_name` пользователь заполняет позже через REST.
#[derive(Debug, Clone)]
pub struct RequiemProfile {
    user_id: Uuid,
    email: Option<Email>,
    display_name: Option<DisplayName>,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

impl RequiemProfile {
    /// Восстанавливает сущность из строки БД.
    pub fn from_persistence(
        user_id: Uuid,
        email: Option<Email>,
        display_name: Option<DisplayName>,
        created_at: OffsetDateTime,
        updated_at: OffsetDateTime,
    ) -> Self {
        Self {
            user_id,
            email,
            display_name,
            created_at,
            updated_at,
        }
    }

    /// Применяет изменения профиля (частичное обновление).
    pub fn apply_update(&mut self, email: Option<Email>, display_name: Option<DisplayName>) {
        if let Some(email) = email {
            self.email = Some(email);
        }
        if let Some(display_name) = display_name {
            self.display_name = Some(display_name);
        }
        self.updated_at = OffsetDateTime::now_utc();
    }

    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    pub fn email(&self) -> Option<&Email> {
        self.email.as_ref()
    }

    pub fn display_name(&self) -> Option<&DisplayName> {
        self.display_name.as_ref()
    }

    pub fn created_at(&self) -> OffsetDateTime {
        self.created_at
    }
}
