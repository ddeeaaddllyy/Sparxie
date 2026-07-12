# nedovolen

[![Rust](https://img.shields.io/badge/Rust-stable-orange.svg)](https://www.rust-lang.org/)
[![Axum](https://img.shields.io/badge/web-Axum%200.8-blue.svg)](https://github.com/tokio-rs/axum)
[![PostgreSQL](https://img.shields.io/badge/db-PostgreSQL%2016-336791.svg)](https://www.postgresql.org/)
[![Kafka](https://img.shields.io/badge/events-Kafka-231F20.svg)](https://kafka.apache.org/)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](#license)

**A centralized authentication and account-management platform for multiple applications, written in Rust.**

`nedovolen` is a single source of truth for user identity. Multiple independent applications (currently **RequiemProject** and **Zenith**) delegate all login, registration, and account management to it, and receive a globally-unique `UUID` for every user. Each application stores only that `UUID` plus its own domain data — never a login or a password.

> **Read this if you have never seen the project before.** Every section below assumes zero prior knowledge and explains not just *how* but *why*.

---

## Table of contents

- [What problem does it solve?](#what-problem-does-it-solve)
- [Key features](#key-features)
- [Architecture](#architecture)
- [How the pieces talk to each other](#how-the-pieces-talk-to-each-other)
- [Technology stack](#technology-stack)
- [Repository layout](#repository-layout)
- [Services and ports](#services-and-ports)
- [API reference](#api-reference)
- [Getting started](#getting-started)
- [Configuration](#configuration)
- [Development](#development)
- [Capacity and scaling](#capacity-and-scaling-how-many-users-can-it-handle)
- [Security model](#security-model)
- [Roadmap](#roadmap)
- [License](#license)

---

## What problem does it solve?

Imagine you run several apps: a fitness tracker, a game, a forum. Without a central identity service, **each app has its own users table, its own passwords, its own login screen**. A user ends up with three accounts and three passwords, and you end up maintaining password security (hashing, resets, breaches) three times.

`nedovolen` fixes this. It is an **Identity Provider (IdP)**:

1. A user registers **once** with `nedovolen` and gets a unique `UUID`.
2. Every app trusts `nedovolen` to say "yes, this is user `UUID=…`".
3. Each app keeps only its own data, keyed by that `UUID` (e.g. Zenith keeps your height and weight; RequiemProject keeps your email and display name).
4. Apps **never** see or store passwords.

This is the same pattern behind "Sign in with Google/Apple", but self-hosted and tailored to your own applications.

---

## Key features

| Area | What you get |
|------|--------------|
| **Authentication** | Register, login, logout, access + refresh tokens, token refresh with rotation |
| **Tokens** | JWT signed with **EdDSA (Ed25519)** — apps verify tokens **offline** with a public key, no network call to the auth server |
| **Passwords** | Hashed with **Argon2id** (memory-hard, GPU-resistant). Plaintext passwords are never stored or logged |
| **Sessions** | Refresh tokens are one-time (rotated on every use); logout instantly revokes access tokens via a Redis blacklist |
| **Event-driven** | User lifecycle events (`UserRegistered`, `UserDeleted`, …) are published to **Kafka**; every app reacts to them independently |
| **Per-service databases** | The auth database holds only `uuid`, `nickname`, `password_hash`. Each app has its **own** database |
| **Clean Architecture** | Every service is split into `domain → application → infrastructure → presentation`, with dependencies pointing strictly inward |
| **Type-safe SQL** | Queries are checked **at compile time** against the real schema via SQLx — no ORM, no runtime SQL surprises |
| **Observability** | Structured `tracing` logs, request IDs, health/readiness probes on every service |

---

## Architecture

`nedovolen` is a **Cargo workspace** (a single repository containing several Rust packages, called *crates*). There are three runnable services and two shared libraries.

```mermaid
graph TB
    subgraph Clients["Client applications"]
        RQ_APP[RequiemProject app]
        ZN_APP[Zenith app]
    end

    subgraph Auth["auth-service (Identity Provider)"]
        AUTH[HTTP API :8080]
        AUTHDB[(PostgreSQL<br/>accounts)]
        REDIS[(Redis<br/>refresh tokens + blacklist)]
    end

    BUS[[Kafka topic<br/>nedovolen.user.events]]

    subgraph Requiem["requiem-service :8081"]
        RQ[HTTP API + consumer]
        RQDB[(PostgreSQL<br/>requiem)]
    end

    subgraph Zenith["zenith-service :8082"]
        ZN[HTTP API + consumer]
        ZNDB[(PostgreSQL<br/>zenith)]
    end

    RQ_APP -- register / login --> AUTH
    ZN_APP -- register / login --> AUTH
    AUTH --- AUTHDB
    AUTH --- REDIS
    AUTH -- publish events --> BUS

    BUS -- consume --> RQ
    BUS -- consume --> ZN
    RQ --- RQDB
    ZN --- ZNDB

    RQ_APP -- "API calls + JWT" --> RQ
    ZN_APP -- "API calls + JWT" --> ZN

    RQ -. "verify JWT offline (public key)" .-> RQ
    ZN -. "verify JWT offline (public key)" .-> ZN
```

**Clean Architecture inside each service** — dependencies only ever point inward:

```
presentation  ──▶  application  ──▶  domain
      │                 ▲
      ▼                 │  (implements ports / traits)
infrastructure ─────────┘
```

- **domain** — pure business types and rules. Knows nothing about HTTP, SQL, or Kafka.
- **application** — use cases (the actual scenarios) and *ports* (interfaces the outside world must implement).
- **infrastructure** — concrete adapters: PostgreSQL, Redis, Kafka, Argon2, Ed25519.
- **presentation** — thin HTTP layer (Axum handlers, DTOs, middleware). No business logic here.

Dependency injection is done through the application state (`AppState`), so there are **no global variables**.

---

## How the pieces talk to each other

There are two independent communication channels between the auth server and the apps, and this is the heart of the design:

**1. Synchronous — offline token verification.**
When a user logs in, `nedovolen` issues a JWT signed with its **private** Ed25519 key. Client apps hold only the matching **public** key. They verify every incoming token **locally**, in microseconds, without ever calling the auth server. This is why the system scales: authenticated traffic never touches the auth server at all.

**2. Asynchronous — Kafka events.**
When something meaningful happens (a user registers, changes their password, or deletes their account), `nedovolen` publishes an event to the Kafka topic `nedovolen.user.events`. Each app has its own *consumer* that reacts:

```mermaid
sequenceDiagram
    autonumber
    participant User
    participant Auth as auth-service
    participant Kafka
    participant Zenith as zenith-service
    participant ZDB as zenith DB

    User->>Auth: POST /auth/register
    Auth->>Auth: hash password (Argon2id), store account
    Auth-->>User: access + refresh tokens
    Auth->>Kafka: publish UserRegistered{uuid, nickname}
    Kafka-->>Zenith: deliver UserRegistered
    Zenith->>ZDB: create empty profile for uuid (idempotent)

    User->>Zenith: PUT /zenith/profile/me (Bearer token)
    Zenith->>Zenith: verify JWT offline
    Zenith->>ZDB: update height / weight / age

    User->>Auth: DELETE /accounts/me
    Auth->>Kafka: publish UserDeleted{uuid}
    Kafka-->>Zenith: deliver UserDeleted
    Zenith->>ZDB: delete profile (food & workouts cascade)
```

Events are keyed by `uuid` (so a user's events keep their order) and consumers are **idempotent** (Kafka guarantees at-least-once delivery, so handlers must tolerate duplicates).

---

## Technology stack

| Concern | Choice | Why |
|---------|--------|-----|
| Language | **Rust (stable)** | Memory safety without a garbage collector; fearless concurrency |
| Async runtime | **Tokio** | Industry-standard async runtime |
| HTTP framework | **Axum 0.8** | Ergonomic, `tower`-based, first-class Tokio integration |
| Database | **PostgreSQL 16** | Reliable, feature-rich relational database |
| DB access | **SQLx** (no ORM) | Async, **compile-time-checked** SQL against the real schema |
| Cache / sessions | **Redis** | Refresh-token registry, JWT blacklist, caching |
| Event bus | **Apache Kafka** (`rdkafka`) | Durable, ordered, replayable inter-service events |
| Passwords | **Argon2id** | Winner of the Password Hashing Competition; memory-hard |
| Tokens | **JWT / EdDSA (Ed25519)** via `jsonwebtoken` | Fast, small keys, offline verification |
| Serialization | **Serde** | The de-facto Rust serialization framework |
| Errors | **thiserror** (libraries) + **anyhow** (top level) | Typed domain errors, ergonomic app-level errors |
| Logging | **tracing** + **tracing-subscriber** | Structured, span-aware logs |
| Validation | **validator** | Declarative request validation |
| Middleware | **tower** / **tower-http** | Request ID, tracing, CORS, compression, timeout |

---

## Repository layout

```
nedovolen/
├── Cargo.toml                # workspace manifest (shared dependency versions)
├── docker-compose.yml        # local Postgres + Redis + Kafka
├── .env.example              # configuration template
├── scripts/gen_keys.sh       # generates the Ed25519 JWT key pair
├── migrations/               # (per service, see below)
│
└── crates/
    ├── shared/               # cross-service kernel: config, telemetry, errors,
    │                         #   JWT verifier, reusable web/auth middleware
    ├── contracts/            # Kafka event schemas — single source of truth
    │
    ├── auth-service/         # ★ the Identity Provider (binary: nedovolen-auth)
    │   ├── domain/           #   Account, Nickname, Password, PasswordHash
    │   ├── application/      #   use cases (register/login/…) + ports
    │   ├── infrastructure/   #   Postgres, Redis, Kafka, Argon2, Ed25519
    │   └── presentation/     #   Axum handlers, DTOs, JWT middleware
    │
    ├── requiem-service/      # RequiemProject: uuid, email, display_name
    │   └── (same four layers) + Kafka consumer
    │
    └── zenith-service/       # Zenith: uuid, height, weight, age, streak
        └── (same four layers) + food_entries + workout_entries + consumer
```

Each service owns its own `migrations/` directory and its own `.sqlx/` offline query cache.

---

## Services and ports

| Service | Binary | Default port | Database | Role |
|---------|--------|--------------|----------|------|
| auth-service | `nedovolen-auth` | `8080` | `nedovolen` | Identity Provider: accounts, tokens, events producer |
| requiem-service | `requiem-service` | `8081` | `requiem` | RequiemProject profiles; events consumer |
| zenith-service | `zenith-service` | `8082` | `zenith` | Zenith fitness data; events consumer |

---

## API reference

All request/response bodies are JSON. Protected endpoints require an `Authorization: Bearer <access_token>` header. Errors share one shape:

```json
{ "error": { "code": "INVALID_CREDENTIALS", "message": "invalid credentials" } }
```

### auth-service (`:8080`)

| Method | Path | Auth | Body | Success |
|--------|------|------|------|---------|
| `POST` | `/api/v1/auth/register` | — | `{ nickname, password }` | `201` tokens |
| `POST` | `/api/v1/auth/login` | — | `{ nickname, password }` | `200` tokens |
| `POST` | `/api/v1/auth/refresh` | — | `{ refresh_token }` | `200` tokens (rotated) |
| `POST` | `/api/v1/auth/logout` | access | `{ refresh_token }` | `204` |
| `GET`  | `/api/v1/accounts/me` | access | — | `200` `{ user_id, nickname, created_at }` |
| `PATCH`| `/api/v1/accounts/me/password` | access | `{ old_password, new_password }` | `204` |
| `DELETE`| `/api/v1/accounts/me` | access | `{ password }` | `204` |
| `GET`  | `/healthz`, `/readyz` | — | — | health probes |

The **tokens** response is:

```json
{
  "user_id": "…uuid…",
  "token_type": "Bearer",
  "access_token": "…jwt…",
  "access_expires_at": "2026-01-01T12:15:00Z",
  "refresh_token": "…jwt…",
  "refresh_expires_at": "2026-01-15T12:00:00Z"
}
```

### requiem-service (`:8081`)

| Method | Path | Auth | Body | Success |
|--------|------|------|------|---------|
| `GET` | `/api/v1/requiem/profile/me` | access | — | `200` profile |
| `PUT` | `/api/v1/requiem/profile/me` | access | `{ email?, display_name? }` | `200` updated profile |
| `GET` | `/healthz`, `/readyz` | — | — | health probes |

### zenith-service (`:8082`)

| Method | Path | Auth | Body | Success |
|--------|------|------|------|---------|
| `GET`  | `/api/v1/zenith/profile/me` | access | — | `200` profile |
| `PUT`  | `/api/v1/zenith/profile/me` | access | `{ height?, weight?, age? }` | `200` updated profile |
| `POST` | `/api/v1/zenith/food` | access | `{ name, calories, eaten_at? }` | `201` entry |
| `GET`  | `/api/v1/zenith/food` | access | — | `200` list |
| `POST` | `/api/v1/zenith/workout` | access | `{ kind, duration_min, performed_at? }` | `201` entry |
| `GET`  | `/api/v1/zenith/workout` | access | — | `200` list |
| `GET`  | `/healthz`, `/readyz` | — | — | health probes |

---

## Getting started

### Prerequisites

You need these installed:

- **[Rust](https://rustup.rs/)** (stable, 1.85+) — `rustup` is the easiest way.
- **[Docker](https://docs.docker.com/get-docker/)** with Docker Compose — to run PostgreSQL, Redis, and Kafka without installing them by hand.
- **OpenSSL** — to generate the JWT signing keys (pre-installed on macOS/Linux; on Windows it ships with Git for Windows / Git Bash).

> On Windows, run the shell commands below in **Git Bash**, not `cmd`/PowerShell.

### 1. Clone the repository

```bash
git clone <your-repo-url> nedovolen
cd nedovolen/server
```

### 2. Start the infrastructure

This launches PostgreSQL (with all three databases pre-created), Redis, and Kafka:

```bash
docker compose up -d
```

Wait ~15 seconds for Kafka to become healthy. Check with `docker compose ps`.

### 3. Generate the JWT signing keys

The auth server signs tokens with a private key; the apps verify them with the matching public key.

```bash
bash scripts/gen_keys.sh          # creates keys/ed25519_private.pem and keys/ed25519_public.pem
```

### 4. Create your configuration file

```bash
cp .env.example .env
```

The defaults already match `docker-compose.yml`, so you usually don't need to edit anything.

### 5. Run the services

Each service is a separate program. Open **three terminals** (all in `nedovolen/server`):

```bash
# Terminal 1 — the Identity Provider
cargo run -p auth-service

# Terminal 2 — RequiemProject
DATABASE_URL=postgres://nedovolen:nedovolen@localhost:5432/requiem \
SERVER_PORT=8081 \
cargo run -p requiem-service

# Terminal 3 — Zenith
DATABASE_URL=postgres://nedovolen:nedovolen@localhost:5432/zenith \
SERVER_PORT=8082 \
cargo run -p zenith-service
```

Each service runs its own database migrations automatically on startup. The auth service also needs `REDIS_URL` and the JWT **private** key (already in `.env`); the client services need only the JWT **public** key path (also in `.env`).

### Quick smoke test

```bash
# 1) Register a user (returns tokens)
curl -s -X POST http://localhost:8080/api/v1/auth/register \
  -H 'content-type: application/json' \
  -d '{"nickname":"alice","password":"Password123"}'

# Copy the access_token from the response, then:
TOKEN=<paste access_token>

# 2) Your account
curl -s http://localhost:8080/api/v1/accounts/me -H "authorization: Bearer $TOKEN"

# 3) Your Zenith profile was auto-created via Kafka — update it
curl -s -X PUT http://localhost:8082/api/v1/zenith/profile/me \
  -H "authorization: Bearer $TOKEN" -H 'content-type: application/json' \
  -d '{"height":180,"weight":75,"age":30}'

# 4) Log a meal
curl -s -X POST http://localhost:8082/api/v1/zenith/food \
  -H "authorization: Bearer $TOKEN" -H 'content-type: application/json' \
  -d '{"name":"Oatmeal","calories":350}'
```

---

## Configuration

All configuration comes from environment variables (loaded from `.env` if present).

| Variable | Used by | Default | Description |
|----------|---------|---------|-------------|
| `SERVER_HOST` | all | `0.0.0.0` | Bind address |
| `SERVER_PORT` | all | `8080` / `8081` / `8082` | HTTP port |
| `REQUEST_TIMEOUT_SECS` | all | `15` | Per-request timeout |
| `DATABASE_URL` | all | — | PostgreSQL connection string (per service) |
| `REDIS_URL` | auth | `redis://localhost:6379` | Redis connection string |
| `KAFKA_BROKERS` | all | `localhost:9092` | Kafka bootstrap servers |
| `KAFKA_USER_EVENTS_TOPIC` | all | `nedovolen.user.events` | Event topic |
| `KAFKA_GROUP_ID` | consumers | `requiem-service` / `zenith-service` | Consumer group |
| `JWT_PRIVATE_KEY_PATH` | auth | `./keys/ed25519_private.pem` | Ed25519 private key (secret) |
| `JWT_PUBLIC_KEY_PATH` | all | `./keys/ed25519_public.pem` | Ed25519 public key |
| `JWT_ISSUER` | all | `nedovolen` | Expected token issuer |
| `ACCESS_TOKEN_TTL_SECS` | auth | `900` (15 min) | Access-token lifetime |
| `REFRESH_TOKEN_TTL_SECS` | auth | `1209600` (14 days) | Refresh-token lifetime |
| `LOG_FORMAT` | all | `json` | `json` or `pretty` |
| `RUST_LOG` | all | `info` | Log level filter |

---

## Development

The project builds **offline by default** — you do not need a running database to compile. This works because SQLx query metadata is cached in each crate's `.sqlx/` directory (checked into git). `.cargo/config.toml` sets `SQLX_OFFLINE=true`.

```bash
# Build everything
cargo build --workspace

# Run the test suite (unit tests + crypto round-trips)
cargo test --workspace

# Run tests that require a live database
DATABASE_URL=postgres://nedovolen:nedovolen@localhost:5432/nedovolen \
cargo test -p auth-service -- --include-ignored
```

**After you change any SQL query**, regenerate that crate's offline cache against a live database:

```bash
cd crates/auth-service   # (or requiem-service / zenith-service)
DATABASE_URL=postgres://nedovolen:nedovolen@localhost:5432/nedovolen \
SQLX_OFFLINE=false cargo sqlx prepare
```

---

## Capacity and scaling (how many users can it handle?)

Short answer: **a single modest instance comfortably serves hundreds of thousands of daily-active users, and the design scales horizontally to millions.** Here is the honest reasoning, because the number depends entirely on *which* operation you measure.

The trick that makes this system scale is **offline token verification**: once a user is logged in, their app verifies the JWT locally (a few microseconds of Ed25519 math) and never contacts the auth server again until the 15-minute access token expires. So the vast majority of traffic never touches the bottleneck.

Reference numbers on a single modest node (≈4 vCPU / 8 GB), with adequately-sized Postgres/Redis/Kafka:

| Operation | Cost driver | Rough throughput (one instance) | Notes |
|-----------|-------------|----------------------------------|-------|
| **Authenticated reads** (profile, `/me`, JWT-protected calls) | Postgres/Redis I/O; JWT verified locally | **thousands – tens of thousands / sec** | This is the bulk of real traffic |
| **Token refresh** | Redis lookup + Ed25519 signing | **thousands / sec** | Cheap |
| **Login / Register** | **Argon2id hashing (CPU + memory bound)** | **~100 – 300 / sec** | This is the real limiter |

Why login/register is the limiter: Argon2id is *deliberately* slow (that's what makes stolen password hashes useless). With default parameters each hash costs tens of milliseconds and a chunk of RAM, run on a dedicated blocking thread pool. That caps sign-ins at a few hundred per second per instance — but sign-ins are rare compared to normal use (a user logs in once, then uses tokens for hours).

Turning that into user counts (with typical assumptions — users log in ≈1×/day, browse many times/day):

- **Concurrent active sessions:** effectively bounded by the *client services'* databases, **not** the auth server, because tokens are verified offline. **Hundreds of thousands to millions** of concurrent sessions are realistic with proper DB sizing.
- **Daily active users on one auth instance:** on the order of **100k – 1M**, limited by peak login rate.
- **Scaling further:** every service is **stateless** (all state lives in Postgres/Redis/Kafka), so you scale **horizontally** — put N instances behind a load balancer and throughput grows roughly linearly.

Scaling knobs, in the order you'll reach for them:

1. **Add auth-service instances** behind a load balancer → more login/register throughput.
2. **Tune Argon2 parameters** to trade hashing cost against throughput for your threat model.
3. **Connection pooling** (e.g. PgBouncer) and **read replicas** for PostgreSQL.
4. **More Kafka partitions** (the topic ships with 3) → more parallel consumers per service (a consumer group scales up to the partition count).
5. **Redis cluster / HA** for the refresh-token registry and blacklist.

> These are engineering estimates, not benchmarks. Real numbers depend on hardware, Argon2 settings, network, and payload sizes — always load-test with your own workload before making capacity promises.

---

## Security model

- **Passwords** are hashed with Argon2id and never stored, logged, or returned. `Debug` output for password types is redacted.
- **Access tokens** are short-lived (15 min) and verified offline by client services.
- **Refresh tokens** are long-lived (14 days), stored in a Redis whitelist, and **rotated on every use** — a stolen-and-reused refresh token is rejected.
- **Logout** blacklists the current access token (until it would expire anyway) and revokes the refresh token.
- **Changing a password** revokes *all* of that user's sessions.
- **Client services never trust request bodies** for identity — the user ID always comes from the verified JWT, never from the payload.
- The auth database stores only `uuid`, `nickname`, `password_hash`. No app can read another app's data; they share nothing but the `uuid`.

---

## Roadmap

- Transactional outbox for guaranteed event delivery (currently events are published best-effort with error logging).
- Rate limiting on `/login` and `/register` (e.g. via `governor`).
- Additional client services — the architecture is designed so new services plug in by consuming the existing event stream, with no changes to existing code.

---

## License

MIT. See the badge at the top; add a `LICENSE` file to formalize.

---

<sub>Documentation also available in [Русский](README-ru.md) and [Deutsch](README-de.md).</sub>
