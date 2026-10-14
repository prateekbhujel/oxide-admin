# Terminus

**An agent-native, batteries-included application framework for Axum and Rust.**

Web Admin • REST API • SQLite Persistence • Machine Introspection (`schema --json`)

[![CI](https://github.com/prateekbhujel/terminus/actions/workflows/rust.yml/badge.svg)](https://github.com/prateekbhujel/terminus/actions/workflows/rust.yml)
[![Version: 0.1.0](https://img.shields.io/badge/crates.io-v0.1.0-emerald.svg)](https://crates.io/crates/terminus-framework)
[![Docs](https://img.shields.io/badge/docs.rs-terminus--framework-blue.svg)](https://docs.rs/terminus-framework)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

---

## Why Terminus?

[Axum](https://github.com/tokio-rs/axum) is a world-class, high-performance web router. But it is intentionally minimal. For every production application, developers repeatedly re-implement the same foundational layers from scratch:

- An administrative dashboard and CRUD interfaces
- SQLite persistence, connection management, and database migrations
- Request validation with structured JSON error responses
- Authentication, dynamic RBAC, and personal access tokens
- **Deterministic machine introspection**, so autonomous AI coding agents can inspect models, routes, and database tables without hallucination

Terminus provides strong conventions and full-stack solutions to these needs, designed from first principles around what makes sense in Rust.

---

## 2-Minute Quickstart

### Option 1: Run the Working Quickstart Example

The fastest way to test Terminus is to clone the repository and run the included working quickstart:

```bash
git clone https://github.com/prateekbhujel/terminus.git
cd terminus
cargo run --example quickstart
```

Open `http://localhost:3000` in your browser. Browse to `http://localhost:3000/admin` to see the live admin dashboard backed by SQLite.

### Option 2: Add to Your Own Project

Add Terminus to your `Cargo.toml`:

```toml
[dependencies]
terminus-framework = "0.1.0"
tokio = { version = "1", features = ["full"] }
serde_json = "1.0"
```

Create a minimal application in `src/main.rs`:

```rust
use terminus::prelude::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::open("terminus.db")?;
    let user_repo = Arc::new(SqliteUserRepository::new(db.connection_arc())?);

    let admin = AdminPanel::new()
        .brand_name("My App Admin")
        .resource(AuditLogResource::new(Arc::new(SqliteAuditRepository::new(db.connection_arc())?)));

    App::new()
        .name("My Application")
        .port(3000)
        .get("/", || async {
            html_response("<h1>Welcome to Terminus!</h1><p><a href='/admin'>Admin Panel &rarr;</a></p>")
        })
        .get("/api/health", || async {
            json_response(serde_json::json!({ "status": "ok", "timestamp": now() }))
        })
        .panel(admin)
        .spec_endpoint("/api/_spec")
        .run()
        .await
}
```

---

## Killer Feature: Deterministic Machine Introspection

In a world where developers pair program with autonomous AI coding agents, agents frequently hallucinate route names, database columns, and model relationships because they must guess across dozens of files.

Terminus solves this with **deterministic application introspection**:

```bash
cargo run --bin terminus -- schema --json
```

Outputs a structured, zero-hallucination JSON specification of your entire application:

```json
{
  "framework": "terminus-framework",
  "version": "0.1.0",
  "app_name": "Warehouse Master",
  "routes": [
    { "method": "GET", "path": "/api/items", "handler_name": "list_items", "auth_required": true }
  ],
  "models": [
    {
      "name": "Item",
      "table": "items",
      "fields": [
        { "name": "id", "rust_type": "String", "sql_type": "TEXT", "primary_key": true },
        { "name": "sku", "rust_type": "String", "sql_type": "TEXT" },
        { "name": "quantity", "rust_type": "i64", "sql_type": "INTEGER" }
      ]
    }
  ],
  "resources": [
    { "name": "Item", "slug": "items", "searchable_fields": ["sku", "name"] }
  ]
}
```

You can also view a clean terminal summary:

```bash
cargo run --bin terminus -- schema
```

---

## Full-Stack Vertical Domain Slices

In classical MVC frameworks, adding a single entity requires scattering code across migrations, models, controllers, requests, routes, and admin panels.

Terminus groups features into self-contained vertical domain slices under `src/modules/<slug>/`:

```bash
cargo run --bin terminus -- make:slice Order --fields="customer_id:string,total:f64,status:string"
```

Generates:
- `mod.rs`: `AppModule` implementation mounting API routes and returning admin `Resource`.
- `model.rs`: Typed domain struct with `camelCase` serde serialization.
- `repository.rs`: Strongly-typed repository trait and SQLite store with CRUD methods.
- `request.rs`: Request DTOs with compile-time type-safe `Validator`.
- `controller.rs`: Axum REST handlers returning `ApiResponse<T>`.
- `resource.rs`: Admin panel resource with searchable and sortable columns.
- `migration.sql`: Typed database table definition.

To register the slice in `src/main.rs`:
```rust
app = app.module(OrderModule::new(repo));
```

---

## Compile-Time Type-Safe Validation

Classical web frameworks use stringly-typed rules like `"required|email|max:255"`, which fail silently at runtime if an agent or developer makes a typo.

Terminus uses fluent, compile-checked validation in Rust:

```rust
use terminus::prelude::*;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub name: String,
    pub age: Option<u32>,
}

impl CreateUserRequest {
    pub fn validate(&self) -> Result<&Self, ValidationErrors> {
        Validator::new(self)
            .rule("email", |r| !r.email.is_empty(), "Email is required")
            .rule("email", |r| r.email.contains('@'), "Valid email address required")
            .rule("name", |r| r.name.len() >= 2, "Name must be at least 2 characters")
            .rule_when(|r| r.age.is_some(), "age", |r| r.age.unwrap() >= 18, "Must be at least 18")
            .validate()
    }
}
```

When validation fails, `ValidationErrors` implements Axum's `IntoResponse`, automatically returning HTTP 422 with structured JSON:

```json
{
  "message": "Validation failed",
  "errors": {
    "email": ["Valid email address required"]
  }
}
```

For untyped JSON payloads, use `JsonValidator`:

```rust
let mut jv = JsonValidator::new(&payload);
let email = jv.require_email("email");
let price = jv.require_number("price");
jv.validate()?;
```

---

## Admin Panel Resources

Define rich, server-rendered admin interfaces using declarative Rust:

```rust
use terminus::prelude::*;

pub struct ProductResource;

impl Resource for ProductResource {
    fn name(&self) -> &str { "Product" }
    fn plural_name(&self) -> &str { "Products" }
    fn slug(&self) -> &str { "products" }

    fn table(&self) -> Table {
        Table::new()
            .column(Column::text("id").label("ID").sortable().searchable())
            .column(Column::text("name").label("Name").sortable().searchable())
            .column(Column::badge("status", vec![
                ("active", "emerald"),
                ("draft", "amber"),
                ("archived", "rose"),
            ]))
            .column(Column::text("createdAt").label("Created At").sortable())
            .page_size(10)
    }

    fn form(&self) -> Form {
        Form::new()
            .field(FormField::text("name").required().placeholder("Item name"))
            .field(FormField::select("status", vec![
                ("active", "Active"),
                ("draft", "Draft"),
                ("archived", "Archived"),
            ]))
    }

    fn fetch_rows(&self, query: &QueryState) -> (Vec<RowData>, usize) {
        // Query SQLite or in-memory store
        (vec![], 0)
    }
}
```

---

## Global Ergonomic Helpers

Access common application state from handlers and domain logic:

```rust
use terminus::prelude::*;

pub async fn handle_checkout() -> Result<ApiResponse<serde_json::Value>, AbortError> {
    // 1. Authorization gate
    abort_unless(can("orders.create"), 403, "Permission Denied")?;

    // 2. Access active user context
    if let Some(current_user) = user() {
        println!("Order by: {}", current_user.name);
    }

    // 3. Current request data
    let token = request().input("token").unwrap_or_default();

    Ok(ApiResponse::ok(serde_json::json!({
        "status": "success",
        "timestamp": now()
    })))
}
```

---

## Dynamic Role-Based Access Control (RBAC)

```rust
use terminus::prelude::*;

let mut user = User::new("1", "Pratik", "pratik@example.com", Role::Superadmin);

// 1. Direct permission grant
user.give_permission_to("orders.refund");

// 2. Explicit Deny Precedence (Overrides administrative default grants)
user.deny_permission("finance.export");

assert!(!user.can("finance.export")); // Denied: Explicit deny overrides Superadmin!
assert!(user.can("orders.view"));     // Allowed: Superadmin default grant
```

---

## Password Hashing & Cryptography

Terminus includes pure-Rust cryptographic primitives with zero external C dependencies:

- **PBKDF2-HMAC-SHA256 (`Hash::make` & `Hash::check`)** conforming to RFC 2898 / NIST SP 800-132:
  ```rust
  use terminus::prelude::*;

  let hash = Hash::make("secret_password");
  assert!(Hash::check("secret_password", &hash));
  assert!(!Hash::check("wrong_password", &hash));
  ```
- **Cryptographic Webhook Verification (`WebhookVerifier`)**:
  Supports Stripe-style timestamped signatures (`t=...,v1=...`), GitHub `sha256=...` signatures, and constant-time comparison to prevent timing attacks.

---

## Terminal UI Studio (`terminus studio`)

Terminus includes an interactive cross-platform terminal dashboard to inspect database tables, active tokens, and service health directly in your terminal:

```bash
cargo run --bin terminus -- studio
```

```text
================================================================================
  TERMINUS STUDIO — Terminal Telemetry, Database & Cache Inspector
  ONLINE  |  DB: CONNECTED (SQLite)  |  HOST: 127.0.0.1:3000
================================================================================
   [1] Overview  [2] Database  [3] Cache & Redis  [4] Queue  [5] API Tokens
--------------------------------------------------------------------------------
[TELEMETRY] SYSTEM TELEMETRY & SERVICE HEALTH:

  ┌──────────────────────┬──────────────────────┬──────────────────────┐
  │ DATABASE (SQLite)    │ CACHE & KEY-VALUE    │ QUEUE & JOBS         │
  │ Active Connections   │ In-Memory + Redis    │ 0 Pending Jobs       │
  │ Path: terminus.db    │ Sub-millisecond TTL  │ Worker: Idle / Ready │
  └──────────────────────┴──────────────────────┴──────────────────────┘

Shortcuts: [1-5] Switch Tab  [R] Refresh  [T <table_name>] Select Table  [Q] Exit
```

---

## CLI Reference

```bash
cargo run --bin terminus -- <command> [options]
```

| Command | Description |
|---|---|
| `new [name] [--targets=...]` | Scaffold a new project (interactive wizard or flags) |
| `schema [--json]` | Inspect application topology (models, routes, admin resources) |
| `make:slice <Name> [--fields=...]` | Scaffold a self-contained vertical domain slice |
| `agent:guide [--json]` | Display AI agent conventions and architecture directives |
| `studio` | Launch terminal UI telemetry dashboard and database inspector |
| `make:resource <Name>` | Scaffold an administrative resource |
| `make:module <Name>` | Scaffold a domain module |
| `make:model <Name>` | Scaffold a domain model struct |
| `make:repo <Name>` | Scaffold a SQLite repository implementation |
| `make:migration <Name>` | Generate a timestamped SQL migration file |
| `migrate` | Run pending database migrations |
| `db:tables` | Inspect database schema and statistics |
| `db:seed` | Seed demo records into database |
| `make:sdk [--lang=ts\|swift\|dart]` | Export typed client SDKs |
| `make:token [name]` | Issue a personal access token |
| `token:list` | List active access tokens |
| `token:revoke [id]` | Revoke an access token |
| `cache:clear` | Clear memory cache store |

---

## Modular Extensions (Multi-Platform Matrix)

For teams building client touchpoints, Terminus provides scaffoldable adapters connecting to the single Rust domain core:

- **Desktop Bridge (`--targets=desktop`)**: Tauri/Electron IPC bridge with an offline SQLite sync queue (`/api/desktop/sync`).
- **IoT Hardware Gateway (`--targets=iot`)**: HMAC-SHA256 authenticated barcode scanner & sensor telemetry scripts.
- **Mobile Push & SDKs (`--targets=mobile`)**: Push notification dispatchers (APNs/FCM) and typed Swift/Dart client SDK generators.

These are modular client bindings, keeping your core Rust backend clean and focused.

---

## Headless Testing & Assertions

Writing HTTP integration tests for web APIs often requires spinning up TCP listeners, managing OS ports, or writing verbose `tower::ServiceExt::oneshot` boilerplate.

Terminus provides an in-memory, headless `TestClient` with fluent assertions:

```rust
use terminus::prelude::*;

#[tokio::test]
async fn test_items_api() {
    let app = App::new()
        .get("/api/health", || async {
            json_response(serde_json::json!({ "status": "ok" }))
        })
        .post("/api/items", |axum::Json(payload): axum::Json<serde_json::Value>| async move {
            let mut jv = JsonValidator::new(&payload);
            jv.require_str("name");
            match jv.validate() {
                Ok(_) => (
                    axum::http::StatusCode::CREATED,
                    [("Content-Type", "application/json")],
                    payload.to_string(),
                )
                    .into_response(),
                Err(e) => e.into_response(),
            }
        });

    let client = TestClient::new(app);

    // 1. GET assertion
    let res = client.get("/api/health").await;
    res.assert_ok().assert_contains("ok");

    // 2. Validation failure assertion (HTTP 422)
    let invalid = client.post("/api/items", serde_json::json!({})).await;
    invalid.assert_unprocessable();

    // 3. Successful POST assertion (HTTP 201)
    let valid = client.post("/api/items", serde_json::json!({ "name": "Laptop" })).await;
    valid.assert_created();
}
```

---

## Empirical Performance: Terminus vs. Plain Axum

Run the benchmark suite locally:

```bash
cargo run --example benchmark --release
```

Measured in-memory across 50,000 iterations per scenario on Apple Silicon (zero socket jitter):

| Scenario | Throughput | p50 Latency | p95 Latency | p99 Latency | Framework Overhead |
|---|---|---|---|---|---|
| **Plain Axum 0.7 (GET)** | 951,123 req/s | 0.75 µs | 0.88 µs | 0.96 µs | Baseline |
| **Terminus App (Core Router)** | **1,025,256 req/s** | **0.75 µs** | **0.88 µs** | **0.92 µs** | **0.00 µs (0%)** |
| **Terminus App (+ CORS Layer)** | 497,362 req/s | 1.79 µs | 1.92 µs | 2.00 µs | +1.04 µs (CORS headers) |
| **Plain Axum (POST JSON)** | 631,719 req/s | 1.29 µs | 1.42 µs | 1.50 µs | Baseline |
| **Terminus App (POST JSON)** | 382,570 req/s | 2.29 µs | 2.46 µs | 2.58 µs | +1.00 µs (CORS headers) |
| **Terminus Fluent Validator** | **13,159,554 ops/s** | **42 ns** | **42 ns** | **42 ns** | Zero string parsing |
| **SQLite In-Memory Query** | 457,049 ops/s | 2.08 µs | 2.21 µs | 2.88 µs | Direct C-binding |

Key takeaway: Terminus is built directly on Axum. When using Terminus Core, your HTTP routing latency is identical to plain Axum (0.75 µs p50). You get batteries-included features (Admin panel, machine introspection, request validation, RBAC) with zero performance penalty.

---

## Verification & Tests

Terminus maintains a comprehensive test suite across feature tests, audit logging, and RBAC:

```bash
cargo test
```

All 60 unit and integration tests run in under 0.05 seconds with zero external service dependencies.

---

## License

MIT License © 2026 Pratik.
