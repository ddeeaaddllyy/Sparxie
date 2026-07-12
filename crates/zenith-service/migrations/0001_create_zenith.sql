-- Данные пользователя в Zenith. Ссылается на accounts.uuid логически.
-- Никаких nickname/password. Записи еды/тренировок каскадно удаляются вместе
-- с профилем (реакция на UserDeleted).

CREATE TABLE IF NOT EXISTS zenith_profiles (
    uuid       UUID        PRIMARY KEY,
    height     INT         NOT NULL DEFAULT 0,
    weight     INT         NOT NULL DEFAULT 0,
    age        INT         NOT NULL DEFAULT 0,
    streak     INT         NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS food_entries (
    id        UUID        PRIMARY KEY,
    user_uuid UUID        NOT NULL REFERENCES zenith_profiles(uuid) ON DELETE CASCADE,
    name      TEXT        NOT NULL,
    calories  INT         NOT NULL,
    eaten_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS food_entries_user_idx ON food_entries (user_uuid);

CREATE TABLE IF NOT EXISTS workout_entries (
    id           UUID        PRIMARY KEY,
    user_uuid    UUID        NOT NULL REFERENCES zenith_profiles(uuid) ON DELETE CASCADE,
    kind         TEXT        NOT NULL,
    duration_min INT         NOT NULL,
    performed_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS workout_entries_user_idx ON workout_entries (user_uuid);
