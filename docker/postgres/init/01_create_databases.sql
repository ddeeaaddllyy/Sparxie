-- Создаёт отдельные БД для сервисов-клиентов (per-service databases).
-- Общая база `nedovolen` создаётся самим контейнером (POSTGRES_DB).
-- Схемы таблиц управляются миграциями SQLx каждого сервиса (этапы 3 и 5).

CREATE DATABASE requiem OWNER nedovolen;
CREATE DATABASE zenith OWNER nedovolen;
