-- Профиль пользователя в RequiemProject. Ссылается на accounts.uuid логически
-- (кросс-БД FK не существует). Никаких nickname/password здесь нет.

CREATE TABLE IF NOT EXISTS requiem_profiles (
    uuid         UUID        PRIMARY KEY,
    email        TEXT        UNIQUE,
    display_name TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);
