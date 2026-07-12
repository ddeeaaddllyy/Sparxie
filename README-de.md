# nedovolen

[![Rust](https://img.shields.io/badge/Rust-stable-orange.svg)](https://www.rust-lang.org/)
[![Axum](https://img.shields.io/badge/web-Axum%200.8-blue.svg)](https://github.com/tokio-rs/axum)
[![PostgreSQL](https://img.shields.io/badge/db-PostgreSQL%2016-336791.svg)](https://www.postgresql.org/)
[![Kafka](https://img.shields.io/badge/events-Kafka-231F20.svg)](https://kafka.apache.org/)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](#lizenz)

**Eine zentralisierte Authentifizierungs- und Kontoverwaltungsplattform für mehrere Anwendungen, geschrieben in Rust.**

`nedovolen` ist die zentrale Quelle der Wahrheit für die Benutzeridentität. Mehrere unabhängige Anwendungen (derzeit **RequiemProject** und **Zenith**) delegieren die gesamte Anmeldung, Registrierung und Kontoverwaltung an sie und erhalten für jeden Benutzer eine global eindeutige `UUID`. Jede Anwendung speichert nur diese `UUID` plus ihre eigenen Domänendaten — niemals einen Login oder ein Passwort.

> **Lesen Sie dies, wenn Sie das Projekt noch nie gesehen haben.** Jeder Abschnitt unten setzt kein Vorwissen voraus und erklärt nicht nur *wie*, sondern auch *warum*.

---

## Inhaltsverzeichnis

- [Welches Problem löst es?](#welches-problem-löst-es)
- [Hauptfunktionen](#hauptfunktionen)
- [Architektur](#architektur)
- [Wie die Teile miteinander kommunizieren](#wie-die-teile-miteinander-kommunizieren)
- [Technologie-Stack](#technologie-stack)
- [Repository-Aufbau](#repository-aufbau)
- [Dienste und Ports](#dienste-und-ports)
- [API-Referenz](#api-referenz)
- [Erste Schritte](#erste-schritte)
- [Konfiguration](#konfiguration)
- [Entwicklung](#entwicklung)
- [Kapazität und Skalierung](#kapazität-und-skalierung-wie-viele-benutzer-verkraftet-es)
- [Sicherheitsmodell](#sicherheitsmodell)
- [Roadmap](#roadmap)
- [Lizenz](#lizenz)

---

## Welches Problem löst es?

Stellen Sie sich vor, Sie betreiben mehrere Apps: einen Fitness-Tracker, ein Spiel, ein Forum. Ohne einen zentralen Identitätsdienst **hat jede App ihre eigene Benutzertabelle, ihre eigenen Passwörter, ihren eigenen Anmeldebildschirm**. Ein Benutzer landet bei drei Konten und drei Passwörtern, und Sie müssen die Passwortsicherheit (Hashing, Resets, Datenlecks) dreimal pflegen.

`nedovolen` behebt das. Es ist ein **Identity Provider (IdP)**:

1. Ein Benutzer registriert sich **einmal** bei `nedovolen` und erhält eine eindeutige `UUID`.
2. Jede App vertraut `nedovolen`, wenn dieser sagt: „Ja, das ist Benutzer `UUID=…`".
3. Jede App speichert nur ihre eigenen Daten, verknüpft mit dieser `UUID` (z. B. Zenith speichert Ihre Größe und Ihr Gewicht; RequiemProject speichert Ihre E-Mail und Ihren Anzeigenamen).
4. Apps sehen oder speichern **niemals** Passwörter.

Das ist dasselbe Muster wie hinter „Mit Google/Apple anmelden", nur selbst gehostet und auf Ihre eigenen Anwendungen zugeschnitten.

---

## Hauptfunktionen

| Bereich | Was Sie bekommen |
|---------|------------------|
| **Authentifizierung** | Registrierung, Anmeldung, Abmeldung, Access- + Refresh-Token, Token-Erneuerung mit Rotation |
| **Token** | JWT signiert mit **EdDSA (Ed25519)** — Apps verifizieren Token **offline** mit einem öffentlichen Schlüssel, ohne Netzwerkaufruf an den Auth-Server |
| **Passwörter** | Gehasht mit **Argon2id** (speicherintensiv, GPU-resistent). Klartext-Passwörter werden nie gespeichert oder geloggt |
| **Sitzungen** | Refresh-Token sind einmalig (bei jeder Nutzung rotiert); die Abmeldung widerruft Access-Token sofort über eine Redis-Blacklist |
| **Ereignisgesteuert** | Ereignisse des Benutzerlebenszyklus (`UserRegistered`, `UserDeleted`, …) werden an **Kafka** veröffentlicht; jede App reagiert unabhängig |
| **Getrennte Datenbanken pro Dienst** | Die Auth-Datenbank enthält nur `uuid`, `nickname`, `password_hash`. Jede App hat ihre **eigene** Datenbank |
| **Clean Architecture** | Jeder Dienst ist in `domain → application → infrastructure → presentation` aufgeteilt, Abhängigkeiten zeigen strikt nach innen |
| **Typsicheres SQL** | Abfragen werden **zur Kompilierzeit** gegen das echte Schema per SQLx geprüft — kein ORM, keine Laufzeit-Überraschungen |
| **Beobachtbarkeit** | Strukturierte `tracing`-Logs, Request-IDs, Health-/Readiness-Proben in jedem Dienst |

---

## Architektur

`nedovolen` ist ein **Cargo-Workspace** (ein einzelnes Repository mit mehreren Rust-Paketen, genannt *Crates*). Es gibt drei ausführbare Dienste und zwei gemeinsame Bibliotheken.

```mermaid
graph TB
    subgraph Clients["Client-Anwendungen"]
        RQ_APP[RequiemProject-App]
        ZN_APP[Zenith-App]
    end

    subgraph Auth["auth-service (Identity Provider)"]
        AUTH[HTTP-API :8080]
        AUTHDB[(PostgreSQL<br/>accounts)]
        REDIS[(Redis<br/>Refresh-Token + Blacklist)]
    end

    BUS[[Kafka-Topic<br/>nedovolen.user.events]]

    subgraph Requiem["requiem-service :8081"]
        RQ[HTTP-API + Consumer]
        RQDB[(PostgreSQL<br/>requiem)]
    end

    subgraph Zenith["zenith-service :8082"]
        ZN[HTTP-API + Consumer]
        ZNDB[(PostgreSQL<br/>zenith)]
    end

    RQ_APP -- Registrierung / Anmeldung --> AUTH
    ZN_APP -- Registrierung / Anmeldung --> AUTH
    AUTH --- AUTHDB
    AUTH --- REDIS
    AUTH -- Ereignisse veröffentlichen --> BUS

    BUS -- konsumieren --> RQ
    BUS -- konsumieren --> ZN
    RQ --- RQDB
    ZN --- ZNDB

    RQ_APP -- "API-Aufrufe + JWT" --> RQ
    ZN_APP -- "API-Aufrufe + JWT" --> ZN

    RQ -. "JWT offline prüfen (öffentl. Schlüssel)" .-> RQ
    ZN -. "JWT offline prüfen (öffentl. Schlüssel)" .-> ZN
```

**Clean Architecture innerhalb jedes Dienstes** — Abhängigkeiten zeigen immer nur nach innen:

```
presentation  ──▶  application  ──▶  domain
      │                 ▲
      ▼                 │  (implementiert Ports / Traits)
infrastructure ─────────┘
```

- **domain** — reine Geschäftstypen und -regeln. Weiß nichts über HTTP, SQL oder Kafka.
- **application** — Anwendungsfälle (die eigentlichen Szenarien) und *Ports* (Schnittstellen, die die Außenwelt implementieren muss).
- **infrastructure** — konkrete Adapter: PostgreSQL, Redis, Kafka, Argon2, Ed25519.
- **presentation** — dünne HTTP-Schicht (Axum-Handler, DTOs, Middleware). Keine Geschäftslogik hier.

Dependency Injection erfolgt über den Anwendungszustand (`AppState`), daher gibt es **keine globalen Variablen**.

---

## Wie die Teile miteinander kommunizieren

Es gibt zwei unabhängige Kommunikationskanäle zwischen dem Auth-Server und den Apps, und das ist das Herzstück des Designs:

**1. Synchron — Offline-Token-Verifizierung.**
Wenn sich ein Benutzer anmeldet, stellt `nedovolen` ein mit seinem **privaten** Ed25519-Schlüssel signiertes JWT aus. Client-Apps besitzen nur den passenden **öffentlichen** Schlüssel. Sie verifizieren jedes eingehende Token **lokal**, in Mikrosekunden, ohne jemals den Auth-Server aufzurufen. Deshalb skaliert das System: authentifizierter Datenverkehr erreicht den Auth-Server überhaupt nicht.

**2. Asynchron — Kafka-Ereignisse.**
Wenn etwas Bedeutsames passiert (ein Benutzer registriert sich, ändert sein Passwort oder löscht sein Konto), veröffentlicht `nedovolen` ein Ereignis im Kafka-Topic `nedovolen.user.events`. Jede App hat ihren eigenen *Consumer*, der reagiert:

```mermaid
sequenceDiagram
    autonumber
    participant User as Benutzer
    participant Auth as auth-service
    participant Kafka
    participant Zenith as zenith-service
    participant ZDB as zenith-DB

    User->>Auth: POST /auth/register
    Auth->>Auth: Passwort hashen (Argon2id), Konto speichern
    Auth-->>User: Access- + Refresh-Token
    Auth->>Kafka: UserRegistered{uuid, nickname} veröffentlichen
    Kafka-->>Zenith: UserRegistered zustellen
    Zenith->>ZDB: leeres Profil für uuid erstellen (idempotent)

    User->>Zenith: PUT /zenith/profile/me (Bearer-Token)
    Zenith->>Zenith: JWT offline prüfen
    Zenith->>ZDB: Größe / Gewicht / Alter aktualisieren

    User->>Auth: DELETE /accounts/me
    Auth->>Kafka: UserDeleted{uuid} veröffentlichen
    Kafka-->>Zenith: UserDeleted zustellen
    Zenith->>ZDB: Profil löschen (Mahlzeiten & Workouts kaskadierend)
```

Ereignisse werden nach `uuid` geschlüsselt (damit die Ereignisse eines Benutzers ihre Reihenfolge behalten) und Consumer sind **idempotent** (Kafka garantiert Zustellung „mindestens einmal", daher müssen Handler Duplikate vertragen).

---

## Technologie-Stack

| Anliegen | Wahl | Warum |
|----------|------|-------|
| Sprache | **Rust (stable)** | Speichersicherheit ohne Garbage Collector; furchtlose Nebenläufigkeit |
| Async-Runtime | **Tokio** | Der De-facto-Standard für Async |
| HTTP-Framework | **Axum 0.8** | Ergonomisch, `tower`-basiert, native Tokio-Integration |
| Datenbank | **PostgreSQL 16** | Zuverlässige, funktionsreiche relationale Datenbank |
| DB-Zugriff | **SQLx** (kein ORM) | Async, **zur Kompilierzeit geprüftes** SQL gegen das echte Schema |
| Cache / Sitzungen | **Redis** | Refresh-Token-Register, JWT-Blacklist, Caching |
| Event-Bus | **Apache Kafka** (`rdkafka`) | Dauerhafte, geordnete, wiederholbare dienstübergreifende Ereignisse |
| Passwörter | **Argon2id** | Gewinner der Password Hashing Competition; speicherintensiv |
| Token | **JWT / EdDSA (Ed25519)** via `jsonwebtoken` | Schnell, kleine Schlüssel, Offline-Verifizierung |
| Serialisierung | **Serde** | Das De-facto-Serialisierungs-Framework in Rust |
| Fehler | **thiserror** (Bibliotheken) + **anyhow** (oberste Ebene) | Typisierte Domänenfehler, ergonomische App-Fehler |
| Logging | **tracing** + **tracing-subscriber** | Strukturierte, Span-bewusste Logs |
| Validierung | **validator** | Deklarative Request-Validierung |
| Middleware | **tower** / **tower-http** | Request-ID, Tracing, CORS, Kompression, Timeout |

---

## Repository-Aufbau

```
nedovolen/
├── Cargo.toml                # Workspace-Manifest (gemeinsame Abhängigkeitsversionen)
├── docker-compose.yml        # lokale Postgres + Redis + Kafka
├── .env.example              # Konfigurationsvorlage
├── scripts/gen_keys.sh       # erzeugt das Ed25519-JWT-Schlüsselpaar
├── migrations/               # (pro Dienst, siehe unten)
│
└── crates/
    ├── shared/               # dienstübergreifender Kern: config, telemetry, errors,
    │                         #   JWT-Verifier, wiederverwendbare web/auth-Middleware
    ├── contracts/            # Kafka-Ereignisschemata — zentrale Quelle der Wahrheit
    │
    ├── auth-service/         # ★ der Identity Provider (Binary: nedovolen-auth)
    │   ├── domain/           #   Account, Nickname, Password, PasswordHash
    │   ├── application/      #   Anwendungsfälle (register/login/…) + Ports
    │   ├── infrastructure/   #   Postgres, Redis, Kafka, Argon2, Ed25519
    │   └── presentation/     #   Axum-Handler, DTOs, JWT-Middleware
    │
    ├── requiem-service/      # RequiemProject: uuid, email, display_name
    │   └── (dieselben vier Schichten) + Kafka-Consumer
    │
    └── zenith-service/       # Zenith: uuid, height, weight, age, streak
        └── (dieselben vier Schichten) + food_entries + workout_entries + Consumer
```

Jeder Dienst besitzt sein eigenes `migrations/`-Verzeichnis und seinen eigenen `.sqlx/`-Offline-Abfrage-Cache.

---

## Dienste und Ports

| Dienst | Binary | Standard-Port | Datenbank | Rolle |
|--------|--------|---------------|-----------|-------|
| auth-service | `nedovolen-auth` | `8080` | `nedovolen` | Identity Provider: Konten, Token, Ereignis-Producer |
| requiem-service | `requiem-service` | `8081` | `requiem` | RequiemProject-Profile; Ereignis-Consumer |
| zenith-service | `zenith-service` | `8082` | `zenith` | Zenith-Fitnessdaten; Ereignis-Consumer |

---

## API-Referenz

Alle Request-/Response-Bodys sind JSON. Geschützte Endpunkte erfordern einen `Authorization: Bearer <access_token>`-Header. Fehler haben eine einheitliche Form:

```json
{ "error": { "code": "INVALID_CREDENTIALS", "message": "invalid credentials" } }
```

### auth-service (`:8080`)

| Methode | Pfad | Auth | Body | Erfolg |
|---------|------|------|------|--------|
| `POST` | `/api/v1/auth/register` | — | `{ nickname, password }` | `201` Token |
| `POST` | `/api/v1/auth/login` | — | `{ nickname, password }` | `200` Token |
| `POST` | `/api/v1/auth/refresh` | — | `{ refresh_token }` | `200` Token (rotiert) |
| `POST` | `/api/v1/auth/logout` | access | `{ refresh_token }` | `204` |
| `GET`  | `/api/v1/accounts/me` | access | — | `200` `{ user_id, nickname, created_at }` |
| `PATCH`| `/api/v1/accounts/me/password` | access | `{ old_password, new_password }` | `204` |
| `DELETE`| `/api/v1/accounts/me` | access | `{ password }` | `204` |
| `GET`  | `/healthz`, `/readyz` | — | — | Health-Proben |

Die **Token**-Antwort lautet:

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

| Methode | Pfad | Auth | Body | Erfolg |
|---------|------|------|------|--------|
| `GET` | `/api/v1/requiem/profile/me` | access | — | `200` Profil |
| `PUT` | `/api/v1/requiem/profile/me` | access | `{ email?, display_name? }` | `200` aktualisiertes Profil |
| `GET` | `/healthz`, `/readyz` | — | — | Health-Proben |

### zenith-service (`:8082`)

| Methode | Pfad | Auth | Body | Erfolg |
|---------|------|------|------|--------|
| `GET`  | `/api/v1/zenith/profile/me` | access | — | `200` Profil |
| `PUT`  | `/api/v1/zenith/profile/me` | access | `{ height?, weight?, age? }` | `200` aktualisiertes Profil |
| `POST` | `/api/v1/zenith/food` | access | `{ name, calories, eaten_at? }` | `201` Eintrag |
| `GET`  | `/api/v1/zenith/food` | access | — | `200` Liste |
| `POST` | `/api/v1/zenith/workout` | access | `{ kind, duration_min, performed_at? }` | `201` Eintrag |
| `GET`  | `/api/v1/zenith/workout` | access | — | `200` Liste |
| `GET`  | `/healthz`, `/readyz` | — | — | Health-Proben |

---

## Erste Schritte

### Voraussetzungen

Sie benötigen Folgendes:

- **[Rust](https://rustup.rs/)** (stable, 1.85+) — `rustup` ist der einfachste Weg.
- **[Docker](https://docs.docker.com/get-docker/)** mit Docker Compose — um PostgreSQL, Redis und Kafka ohne manuelle Installation auszuführen.
- **OpenSSL** — zum Erzeugen der JWT-Signierschlüssel (auf macOS/Linux vorinstalliert; unter Windows in Git für Windows / Git Bash enthalten).

> Führen Sie unter Windows die folgenden Befehle in **Git Bash** aus, nicht in `cmd`/PowerShell.

### 1. Repository klonen

```bash
git clone <ihre-repo-url> nedovolen
cd nedovolen/server
```

### 2. Infrastruktur starten

Dies startet PostgreSQL (mit allen drei vorab erstellten Datenbanken), Redis und Kafka:

```bash
docker compose up -d
```

Warten Sie ~15 Sekunden, bis Kafka gesund ist. Prüfen Sie mit `docker compose ps`.

### 3. JWT-Signierschlüssel erzeugen

Der Auth-Server signiert Token mit einem privaten Schlüssel; die Apps verifizieren sie mit dem passenden öffentlichen Schlüssel.

```bash
bash scripts/gen_keys.sh          # erzeugt keys/ed25519_private.pem und keys/ed25519_public.pem
```

### 4. Konfigurationsdatei erstellen

```bash
cp .env.example .env
```

Die Standardwerte passen bereits zu `docker-compose.yml`, sodass Sie normalerweise nichts ändern müssen.

### 5. Dienste ausführen

Jeder Dienst ist ein eigenes Programm. Öffnen Sie **drei Terminals** (alle in `nedovolen/server`):

```bash
# Terminal 1 — der Identity Provider
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

Jeder Dienst führt beim Start seine eigenen Datenbankmigrationen automatisch aus. Der Auth-Dienst benötigt außerdem `REDIS_URL` und den **privaten** JWT-Schlüssel (bereits in `.env`); die Client-Dienste benötigen nur den Pfad zum **öffentlichen** JWT-Schlüssel (ebenfalls in `.env`).

### Schnelltest

```bash
# 1) Benutzer registrieren (gibt Token zurück)
curl -s -X POST http://localhost:8080/api/v1/auth/register \
  -H 'content-type: application/json' \
  -d '{"nickname":"alice","password":"Password123"}'

# Kopieren Sie das access_token aus der Antwort, dann:
TOKEN=<access_token einfügen>

# 2) Ihr Konto
curl -s http://localhost:8080/api/v1/accounts/me -H "authorization: Bearer $TOKEN"

# 3) Ihr Zenith-Profil wurde per Kafka automatisch erstellt — aktualisieren wir es
curl -s -X PUT http://localhost:8082/api/v1/zenith/profile/me \
  -H "authorization: Bearer $TOKEN" -H 'content-type: application/json' \
  -d '{"height":180,"weight":75,"age":30}'

# 4) Eine Mahlzeit erfassen
curl -s -X POST http://localhost:8082/api/v1/zenith/food \
  -H "authorization: Bearer $TOKEN" -H 'content-type: application/json' \
  -d '{"name":"Oatmeal","calories":350}'
```

---

## Konfiguration

Die gesamte Konfiguration stammt aus Umgebungsvariablen (bei Vorhandensein aus `.env` geladen).

| Variable | Verwendet von | Standard | Beschreibung |
|----------|---------------|----------|--------------|
| `SERVER_HOST` | alle | `0.0.0.0` | Bind-Adresse |
| `SERVER_PORT` | alle | `8080` / `8081` / `8082` | HTTP-Port |
| `REQUEST_TIMEOUT_SECS` | alle | `15` | Timeout pro Anfrage |
| `DATABASE_URL` | alle | — | PostgreSQL-Verbindungsstring (pro Dienst) |
| `REDIS_URL` | auth | `redis://localhost:6379` | Redis-Verbindungsstring |
| `KAFKA_BROKERS` | alle | `localhost:9092` | Kafka-Bootstrap-Server |
| `KAFKA_USER_EVENTS_TOPIC` | alle | `nedovolen.user.events` | Ereignis-Topic |
| `KAFKA_GROUP_ID` | Consumer | `requiem-service` / `zenith-service` | Consumer-Gruppe |
| `JWT_PRIVATE_KEY_PATH` | auth | `./keys/ed25519_private.pem` | Privater Ed25519-Schlüssel (geheim) |
| `JWT_PUBLIC_KEY_PATH` | alle | `./keys/ed25519_public.pem` | Öffentlicher Ed25519-Schlüssel |
| `JWT_ISSUER` | alle | `nedovolen` | Erwarteter Token-Aussteller |
| `ACCESS_TOKEN_TTL_SECS` | auth | `900` (15 Min.) | Lebensdauer des Access-Tokens |
| `REFRESH_TOKEN_TTL_SECS` | auth | `1209600` (14 Tage) | Lebensdauer des Refresh-Tokens |
| `LOG_FORMAT` | alle | `json` | `json` oder `pretty` |
| `RUST_LOG` | alle | `info` | Log-Level-Filter |

---

## Entwicklung

Das Projekt baut **standardmäßig offline** — Sie benötigen keine laufende Datenbank zum Kompilieren. Das funktioniert, weil die SQLx-Abfrage-Metadaten im `.sqlx/`-Verzeichnis jedes Crates zwischengespeichert sind (in git eingecheckt). `.cargo/config.toml` setzt `SQLX_OFFLINE=true`.

```bash
# Alles bauen
cargo build --workspace

# Testsuite ausführen (Unit-Tests + Krypto-Round-Trips)
cargo test --workspace

# Tests, die eine laufende Datenbank benötigen
DATABASE_URL=postgres://nedovolen:nedovolen@localhost:5432/nedovolen \
cargo test -p auth-service -- --include-ignored
```

**Nach dem Ändern einer SQL-Abfrage** erzeugen Sie den Offline-Cache dieses Crates gegen eine laufende Datenbank neu:

```bash
cd crates/auth-service   # (oder requiem-service / zenith-service)
DATABASE_URL=postgres://nedovolen:nedovolen@localhost:5432/nedovolen \
SQLX_OFFLINE=false cargo sqlx prepare
```

---

## Kapazität und Skalierung (wie viele Benutzer verkraftet es?)

Kurze Antwort: **Eine einzelne bescheidene Instanz bedient bequem Hunderttausende täglich aktiver Benutzer, und das Design skaliert horizontal bis in die Millionen.** Hier die ehrliche Begründung, denn die Zahl hängt vollständig davon ab, *welche* Operation Sie messen.

Der Trick, der das System skalierbar macht, ist die **Offline-Token-Verifizierung**: Sobald ein Benutzer angemeldet ist, verifiziert seine App das JWT lokal (ein paar Mikrosekunden Ed25519-Mathematik) und kontaktiert den Auth-Server nie wieder, bis das 15-minütige Access-Token abläuft. Der Großteil des Datenverkehrs berührt also nie den Engpass.

Referenzwerte auf einem einzelnen bescheidenen Knoten (≈4 vCPU / 8 GB), mit angemessen dimensioniertem Postgres/Redis/Kafka:

| Operation | Kostentreiber | Ungefährer Durchsatz (eine Instanz) | Hinweise |
|-----------|---------------|--------------------------------------|----------|
| **Authentifizierte Lesevorgänge** (Profil, `/me`, JWT-geschützte Aufrufe) | Postgres/Redis-I/O; JWT lokal geprüft | **Tausende – Zehntausende / Sek.** | Dies ist der Großteil des realen Datenverkehrs |
| **Token-Erneuerung** | Redis-Lookup + Ed25519-Signierung | **Tausende / Sek.** | Günstig |
| **Anmeldung / Registrierung** | **Argon2id-Hashing (CPU- + speichergebunden)** | **~100 – 300 / Sek.** | Der eigentliche Begrenzer |

Warum Anmeldung/Registrierung der Begrenzer ist: Argon2id ist *absichtlich* langsam (genau das macht gestohlene Passwort-Hashes nutzlos). Mit Standardparametern kostet jeder Hash Dutzende Millisekunden und ein Stück RAM, ausgeführt in einem dedizierten Blocking-Thread-Pool. Das begrenzt Anmeldungen auf einige Hundert pro Sekunde pro Instanz — aber Anmeldungen sind im Vergleich zur normalen Nutzung selten (ein Benutzer meldet sich einmal an und nutzt dann stundenlang Token).

In Benutzerzahlen umgerechnet (mit typischen Annahmen — Anmeldung ≈1×/Tag, viele Aufrufe/Tag):

- **Gleichzeitige aktive Sitzungen:** faktisch durch die Datenbanken der *Client-Dienste* begrenzt, **nicht** durch den Auth-Server, da Token offline verifiziert werden. **Hunderttausende bis Millionen** gleichzeitiger Sitzungen sind bei richtiger DB-Dimensionierung realistisch.
- **Täglich aktive Benutzer auf einer Auth-Instanz:** in der Größenordnung von **100k – 1M**, begrenzt durch die Spitzen-Anmelderate.
- **Weitere Skalierung:** Jeder Dienst ist **zustandslos** (der gesamte Zustand liegt in Postgres/Redis/Kafka), daher skalieren Sie **horizontal** — stellen Sie N Instanzen hinter einen Load Balancer, und der Durchsatz wächst annähernd linear.

Skalierungsstellschrauben, in der Reihenfolge, in der Sie danach greifen werden:

1. **Auth-Service-Instanzen hinzufügen** hinter einem Load Balancer → höherer Anmelde-/Registrierungsdurchsatz.
2. **Argon2-Parameter anpassen**, um Hashing-Kosten gegen Durchsatz für Ihr Bedrohungsmodell abzuwägen.
3. **Connection Pooling** (z. B. PgBouncer) und **Read Replicas** für PostgreSQL.
4. **Mehr Kafka-Partitionen** (das Topic hat 3) → mehr parallele Consumer pro Dienst (eine Consumer-Gruppe skaliert bis zur Partitionsanzahl).
5. **Redis-Cluster / HA** für das Refresh-Token-Register und die Blacklist.

> Dies sind technische Schätzungen, keine Benchmarks. Reale Zahlen hängen von Hardware, Argon2-Einstellungen, Netzwerk und Payload-Größen ab — führen Sie stets Lasttests mit Ihrer eigenen Arbeitslast durch, bevor Sie Kapazitätszusagen machen.

---

## Sicherheitsmodell

- **Passwörter** werden mit Argon2id gehasht und niemals gespeichert, geloggt oder zurückgegeben. Die `Debug`-Ausgabe für Passworttypen ist geschwärzt.
- **Access-Token** sind kurzlebig (15 Min.) und werden von Client-Diensten offline verifiziert.
- **Refresh-Token** sind langlebig (14 Tage), in einer Redis-Whitelist gespeichert und werden **bei jeder Nutzung rotiert** — ein gestohlenes und wiederverwendetes Refresh-Token wird abgelehnt.
- **Die Abmeldung** setzt das aktuelle Access-Token auf die Blacklist (bis es ohnehin ablaufen würde) und widerruft das Refresh-Token.
- **Eine Passwortänderung** widerruft *alle* Sitzungen dieses Benutzers.
- **Client-Dienste vertrauen niemals dem Request-Body** in Bezug auf die Identität — die Benutzer-ID stammt immer aus dem verifizierten JWT, nie aus der Payload.
- Die Auth-Datenbank speichert nur `uuid`, `nickname`, `password_hash`. Keine App kann die Daten einer anderen App lesen; sie teilen nichts außer der `uuid`.

---

## Roadmap

- Transaktionaler Outbox für garantierte Ereigniszustellung (derzeit werden Ereignisse als Best-Effort mit Fehler-Logging veröffentlicht).
- Rate-Limiting für `/login` und `/register` (z. B. via `governor`).
- Weitere Client-Dienste — die Architektur ist so gestaltet, dass sich ein neuer Dienst durch das Konsumieren des vorhandenen Ereignisstroms einklinkt, ohne Änderungen am bestehenden Code.

---

## Lizenz

MIT. Siehe das Badge oben; fügen Sie eine `LICENSE`-Datei hinzu, um sie zu formalisieren.

---

<sub>Dokumentation auch verfügbar auf [English](README.md) und [Русский](README-ru.md).</sub>
