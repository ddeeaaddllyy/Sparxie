-- Общая база аккаунтов nedovolen. Хранит только идентичность:
-- uuid, nickname, password_hash + временные метки. Никаких профилей сервисов.

CREATE TABLE IF NOT EXISTS accounts (
    uuid          UUID        PRIMARY KEY,
    nickname      TEXT        NOT NULL UNIQUE,
    password_hash TEXT        NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
