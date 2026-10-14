# Terminus

The universal fullstack application framework for Rust and Axum powering web, mobile APIs, desktop, and IoT.

[![CI](https://github.com/prateekbhujel/terminus/actions/workflows/rust.yml/badge.svg)](https://github.com/prateekbhujel/terminus/actions/workflows/rust.yml)
[![Version: 0.1.0](https://img.shields.io/badge/crates.io-v0.1.0-emerald.svg)](https://crates.io/crates/terminus-framework)
[![Docs](https://img.shields.io/badge/docs.rs-terminus--framework-blue.svg)](https://docs.rs/terminus-framework)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Terminus is an integrated fullstack application platform and developer toolkit for Rust and Axum. It wraps the power, safety, and speed of Rust inside an ultra-ergonomic developer experience designed to eliminate boilerplate, complex lifetime management, and concurrency ceremony.

Whether building high-throughput web backends, rich content management systems, multi-vendor eCommerce platforms, mobile APIs for Android and iOS, desktop tools, or lightweight IoT telemetry ingestors, Terminus unifies all essential capabilities into a single compile-time checked executable.

---

## Core Capabilities

- **Zero-Boilerplate Application Engine**: High-level `App` bootstrap with fluent routing, automatic JSON serialization, CORS, and graceful server management without manual lifetime or concurrency ceremony.
- **Zero-Ceremony Global Context Helpers**: Access `auth()`, `user()`, `can()`, `cannot()`, `request()`, `abort()`, `abort_if()`, `now()`, and `today()` globally from any function, controller, or domain model without threading context structs through call stacks.
- **Interactive UI & Visual Builders**: Drag-and-drop form block builders, sortable table rows with HTML5 drag handles, and inline-editable table cells with instantaneous asynchronous persistence.
- **Advanced Filtering Engine**: Comprehensive filtering with date-range pickers, numeric range sliders, multi-select tag badges, boolean toggles, full-text fuzzy search, and saved filter presets.
- **Native Document Generation**: OpenXML Microsoft Excel (`.xlsx`), CSV export/import, and native PDF report and invoice compilation with zero external C dependencies.
- **Identity, Passkeys & Webhooks**: WebAuthn / Passkey registration and authentication, OAuth 2.0 social authentication (GitHub, Google, custom OIDC), and cryptographic webhook signature verification for external services.
- **Multi-Platform Ready**: Tailored endpoints and payloads for mobile push notifications (APNs / FCM), IoT device token telemetry ingestion, and desktop webview/Tauri IPC integration.
- **Dynamic Role-Based Access Control**: Granular direct permissions, explicit deny precedence overriding administrative privileges, wildcard hierarchies (`orders.*`), and 30-second TTL in-memory grant caching with instant invalidation on mutation.
- **Active Record ORM & Fluent Query Builder**: Expressive query chains with parameterized SQL compilation, schema migrations, and SQLite persistence.
- **Real-Time Collaboration & Telemetry**: Native Axum WebSocket broadcasting (`/admin/ws`), multi-admin presence, background queue worker, transactional mailer, and terminal UI telemetry dashboard (`terminus studio`).
- **Production-Ready & Self-Contained**: Compiles into a single high-performance static binary with zero external runtime dependencies, ready for Docker, systemd, bare metal, or cloud container deployment.

---

## Architecture

```
                    ┌──────────────────────────────────────────────┐
                    │               Terminus App                   │
                    │        (Web, Mobile APIs, Desktop, IoT)      │
                    └──────────────────────┬───────────────────────┘
                                           │
         ┌───────────────────┬─────────────┴─────────────┬───────────────────┐
         ▼                   ▼                           ▼                   ▼
┌─────────────────┐ ┌─────────────────┐         ┌─────────────────┐ ┌─────────────────┐
│  Administrative │ │  REST API Engine│         │ Real-Time WS    │ │ Document Engine │
│  Panel & Forms  │ │  & Mobile Sync  │         │ Broadcaster     │ │ (Excel & PDF)   │
└────────┬────────┘ └────────┬────────┘         └────────┬────────┘ └────────┬────────┘
         │                   │                           │                   │
         └───────────────────┼───────────────────────────┴───────────────────┘
                             ▼
                    ┌─────────────────┐
                    │  Terminus Core  │
                    │  • Global Auth  │
                    │  • Dynamic RBAC │
                    │  • ORM & Cache  │
                    └────────┬────────┘
                             ▼
                    ┌─────────────────┐
                    │ Storage Backend │
                    │ (SQLite / Redis)│
                    └─────────────────┘
```

---

## Quickstart

### 1. Scaffold a New Project

Scaffold a new project using the CLI wizard:

```bash
cargo terminus new my_app
```

The CLI wizard lets you compose your exact stack and client targets:
- **Target Platforms**: Choose any combination:
  - **Web**: HTTP Server, Admin Dashboard, REST & OpenAPI 3.0 specification.
  - **Desktop**: Offline SQLite sync queue and Tauri / Electron IPC bridge.
  - **Mobile**: Native Swift for iOS and Flutter/Dart for Android with Push Dispatchers.
  - **IoT**: Hardware barcode scanners, HMAC device registry, and sensor streams.
- **Frontend Template**: React 19 + TypeScript, Vue 3 + TypeScript, HTMX + Tailwind, or Built-in Admin Dashboard (zero build step).
- **Database Engine**: SQLite (single-file, zero setup) or PostgreSQL.

You can also scaffold non-interactively with exact flags:

```bash
# Desktop-only workstation (Tauri bridge + offline SQLite sync queue)
terminus new warehouse_pos --targets=desktop

# IoT-only hardware scanner gateway (HMAC device auth + telemetry script)
terminus new scanner_hub --targets=iot

# Both Web & Desktop with React 19 frontend
terminus new store_manager --targets=web,desktop --frontend=react

# Web, Desktop, and Mobile omni-channel ecosystem
terminus new omni_retail --targets=web,desktop,mobile

# All 4 platforms (Universal ecosystem)
terminus new enterprise_wms --targets=all
```

### 2. Built-in Terminal Studio (TUI)

Terminus includes an interactive cross-platform terminal dashboard to inspect database records, memory and Redis caches, queue depths, API tokens, and live telemetry in real time:

```bash
cargo terminus studio
```

```text
╭────────────────────────────────────────────────────────────────────────────────────────────╮
│  TERMINUS STUDIO — Terminal Telemetry, Database & Cache Inspector                          │
│  ONLINE  │  DB: CONNECTED (SQLite)  │  HOST: macos/aarch64  │  UTC Live Telemetry          │
╰────────────────────────────────────────────────────────────────────────────────────────────╯
   [1] Overview  [2] Database  [3] Cache & Redis  [4] Queue  [5] API Tokens  [6] Mail Outbox  [7] Audit Logs  
──────────────────────────────────────────────────────────────────────────────────────────────
SYSTEM TELEMETRY & SERVICE HEALTH:

  ┌──────────────────────┬──────────────────────┬──────────────────────┐
  │ DATABASE (SQLite)    │ CACHE & KEY-VALUE    │ QUEUE & JOBS         │
  │ Active Connections   │ In-Memory + Redis    │ 0 Pending Jobs       │
  │ Path: terminus.db    │ Sub-millisecond TTL  │ Worker: Idle / Ready │
  ├──────────────────────┼──────────────────────┼──────────────────────┤
  │ API ACCESS TOKENS    │ AUDIT TRAIL LOGS     │ HTTP ENGINE          │
  │ RFC 6234 Bearer Auth │ Immutable Append Log │ Axum 0.7 / Tokio     │
  │ Abilities & Scopes   │ Forensics IP & Agent │ Zero GC Latency      │
  └──────────────────────┴──────────────────────┴──────────────────────┘

Shortcuts: [1-7] Switch Tab  [R] Refresh  [T <table_name>] Select Table  [Q] Exit
```

### 3. Add to an Existing Project

Add Terminus to your `Cargo.toml`:

```toml
[dependencies]
terminus-framework = "0.1.0"
tokio = { version = "1", features = ["full"] }
serde_json = "1.0"
```

### 4. Minimal Web & API Application

```rust
use terminus::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::new()
        .name("Terminus Application")
        .port(3000)
        .get("/", || async { "Welcome to Terminus!" })
        .get("/api/health", || async {
            json_response(serde_json::json!({
                "status": "healthy",
                "framework": "Terminus",
                "time": now().to_rfc3339()
            }))
        })
        .run()
        .await
}
```

### 2. Defining an Admin Resource with Rich Media & Inline Editing

```rust
use terminus::prelude::*;

pub struct ProductResource;

impl Resource for ProductResource {
    fn name(&self) -> &'static str { "Product" }
    fn slug(&self) -> &'static str { "products" }

    fn table(&self) -> Table {
        Table::new()
            .column(Column::text("title").searchable().sortable().editable())
            .column(Column::text("category").editable_select(vec!["Hardware", "Software", "Merch"]))
            .column(Column::numeric("price").sortable().editable())
            .column(Column::image("image").label("Thumbnail"))
            .column(Column::badge("status", vec![
                ("active", "emerald"),
                ("draft", "amber"),
                ("archived", "rose"),
            ]))
            .action(Action::view())
            .action(Action::edit())
            .bulk_action(BulkAction::delete())
            .bulk_action(BulkAction::export())
            .sortable_rows()
    }

    fn form(&self) -> Form {
        Form::new()
            .section(
                FormSection::make("General Information")
                    .description("Core product identifiers and merchandising details.")
                    .schema(vec![
                        FormField::text("title").required().placeholder("Mechanical Keyboard"),
                        FormField::select("category", vec![
                            ("hardware", "Hardware"),
                            ("software", "Software"),
                        ]),
                        FormField::number("price").required().prefix("$"),
                        FormField::rich_text("description").helper_text("Markdown formatted product copy."),
                    ])
            )
            .section(
                FormSection::make("Media & Layout Blocks")
                    .schema(vec![
                        FormField::image("image").label("Product Hero Image"),
                        FormField::barcode("barcode").placeholder("UPC-A / EAN-13"),
                        FormField::color("accent_color").label("Theme Accent Color"),
                        FormField::draggable_blocks("page_layout", vec![
                            "hero_banner", "feature_grid", "specs_table", "reviews"
                        ]),
                    ])
            )
    }

    fn fetch_rows(&self, query: &QueryState) -> (Vec<RowData>, usize) {
        // Query database via SQLite or Terminus ORM
        (vec![], 0)
    }
}
```

---

## Agent-Native Architecture & Deterministic Introspection

Terminus is engineered from first principles around what makes sense in Rust, and in a world where AI agents build applications and humans no longer write boilerplate code by hand.

### 1. Deterministic Machine Introspection (`terminus schema --json`)
AI agents query complete application topology in one deterministic command with zero hallucination:

```bash
cargo terminus schema --json
```

Returns models, database tables, route endpoints, administrative resources, and client SDK bindings:

```json
{
  "framework": "terminus-framework",
  "version": "0.1.0",
  "app_name": "Warehouse Master",
  "routes": [
    { "method": "GET", "path": "/api/items", "handler": "list_items", "auth": true }
  ],
  "models": [
    { "name": "Item", "table": "items", "fields": [ ... ] }
  ],
  "resources": [
    { "name": "Item", "slug": "items", "searchable": ["sku", "name"] }
  ]
}
```

### 2. Full-Stack Vertical Domain Slices (`terminus make:slice`)
In classical frameworks, adding an entity requires editing 6+ scattered files. In Terminus, every domain feature is a self-contained, compile-checked vertical slice:

```bash
cargo terminus make:slice Order --fields="customer_id:string,total:f64,status:string"
```

Generates `src/modules/order/`:
- `mod.rs`: `AppModule` implementation mounting routes and admin resource.
- `model.rs`: Typed Rust struct with camelCase serde derivation.
- `repository.rs`: Strongly-typed repository with transactional methods.
- `request.rs`: Compile-time validated request DTOs.
- `controller.rs`: Axum handler functions returning `ApiResponse<T>`.
- `resource.rs`: AdminPanel specification with searchable and sortable columns.
- `migration.sql`: Typed database table definition.

### 3. Compile-Time Type-Safe Validation (Zero Stringly Rules)
Eliminate runtime parsing failures and typo bugs. Terminus replaces runtime string rules with compile-checked fluent validation:

```rust
use terminus::prelude::*;

let result = Validator::new(&payload)
    .rule("email", |r| !r.email.is_empty(), "Email is required")
    .rule("email", |r| r.email.contains('@'), "Must be a valid email")
    .rule("amount", |r| r.amount > 0.0, "Amount must be positive")
    .rule_when(|r| r.is_rush, "amount", |r| r.amount >= 50.0, "Rush orders require min $50")
    .validate();
```

---

## Global Ergonomic Helpers

Terminus provides zero-ceremony global helpers accessible everywhere in your application without passing context instances around:

```rust
use terminus::prelude::*;

pub async fn handle_checkout() -> Result<ApiResponse<serde_json::Value>, AbortError> {
    // 1. Authorization checks
    abort_unless(can("orders.create"), 403, "Permission Denied")?;

    // 2. Access active user
    if let Some(current_user) = user() {
        println!("Order initiated by: {}", current_user.name);
    }

    // 3. Inspect incoming request
    let payment_token = request().input("token").unwrap_or_default();
    let client_ip = request().ip();

    // 4. Time and configuration utilities
    let created_date = today();
    let site_title = config("app.name").unwrap_or("Terminus".to_string());

    Ok(ApiResponse::ok(serde_json::json!({
        "status": "success",
        "created_at": created_date,
        "ip": client_ip
    })))
}
```

---

## Advanced Table & Query Filters

Filter complex datasets using date ranges, numeric sliders, multi-select tags, and saved presets:

```rust
use terminus::prelude::*;

let date_filter = Filter::date_range("created_at").label("Order Date");
let price_filter = Filter::range_slider("total", 0.0, 1000.0, 25.0).label("Total Spend");
let status_filter = Filter::multi_select("status", vec![
    ("paid", "Paid"),
    ("pending", "Pending"),
    ("cancelled", "Cancelled"),
]);

let vip_preset = FilterPreset::make("vip_orders", "High Value Orders")
    .with("total_min", "500")
    .with("status", "paid");
```

---

## Native Document Generation (Excel & PDF)

Compile clean spreadsheets and professional invoices with zero external system requirements:

```rust
use terminus::prelude::*;

// 1. Export Microsoft Excel XML and CSV
let mut excel = ExcelDocument::new("Transactions")
    .headers(vec!["Reference", "Customer", "Amount", "Status"]);

excel.add_row(vec!["TX-1001", "Alice Smith", "$450.00", "Settled"]);
let csv_data = excel.to_csv();
let xlsx_xml = excel.to_excel_xml();

// 2. Compile PDF Invoice
let pdf = PdfDocument::new("INVOICE #1092")
    .company("Terminus Platform Inc.", "100 Market St, San Francisco, CA")
    .recipient("Acme Corporation", "200 Broadway, New York, NY")
    .table(
        vec!["Description", "Qty", "Price", "Amount"],
        vec![
            vec!["Enterprise Annual License", "1", "$1,200.00", "$1,200.00"],
            vec!["Priority 24/7 SLA", "1", "$300.00", "$300.00"],
        ]
    )
    .total("$1,500.00")
    .notes("Payment processed via wire transfer.");

let pdf_bytes = pdf.to_pdf_bytes();
```

---

## Identity, Passkeys & Webhooks

Terminus includes built-in flows for modern passkey authentication, OAuth social redirection, and cryptographic webhook verification for external integrations:

```rust
use terminus::prelude::*;

// 1. WebAuthn / Passkey Registration Challenge
let challenge = PasskeyChallenge::generate("usr_99", "pratik", "terminus.dev");

// 2. OAuth Social Redirection
let github_oauth = OAuthProvider::GitHub {
    client_id: "gh_client_id".into(),
    client_secret: "gh_client_secret".into(),
};
let redirect_url = github_oauth.authorization_url("https://terminus.dev/auth/callback", "random_state");

// 3. Cryptographic Webhook Signature Verification
// Compatible with any payment gateway, SaaS event dispatcher, or Git webhook
let is_valid = WebhookVerifier::verify_signature(&payload, &signature_header, &webhook_secret);
if is_valid {
    // Process verified event payload safely with your custom domain logic
}
```

---

---

## Multi-Platform Engine (Mobile, IoT, Desktop)

Terminus acts as the universal domain and rule foundation powering web, mobile apps, desktop POS workstations, and IoT hardware scanners:

```rust
use terminus::prelude::*;

// 1. Mobile Platform Adapter: Push Notifications & Typed SDK Generation
let push_dispatcher = PushDispatcher::new();
let notification = PushNotification::new("Order Shipped", "Package #901 on the way", "fcm_token_alice")
    .with_data("tracking", "TRK-901")
    .badge(1);

push_dispatcher.enqueue(notification);
let delivered = push_dispatcher.deliver_all();

// Automatically generate typed client SDKs for mobile & frontend teams
let sdk = SdkGenerator::new("Shop App", "1.0.0")
    .type_schema("Product", vec![("id", "string"), ("name", "string"), ("price", "number")])
    .endpoint("GET", "/api/products", None, "Product[]", "Fetch product catalog");
let typescript_code = sdk.to_typescript();
let dart_code = sdk.to_dart();
let swift_code = sdk.to_swift();
let openapi_spec = sdk.to_openapi_json();

// 2. Desktop Platform Adapter: Offline Workstation Sync Queue
let sync_queue = OfflineSyncQueue::new();
let tx_id = sync_queue.enqueue(
    "dispatch_order",
    "/api/orders/dispatch",
    serde_json::json!({ "orderId": "ORD-101", "dock": "Station-B" }),
    "idemp_98102"
);
// Workstation continues processing offline; drains and replays when connected:
sync_queue.mark_synced(&tx_id);

// 3. IoT & Hardware Adapter: Device Registry & Barcode/RFID Scanners
let registry = DeviceRegistry::new();
registry.register("scanner_01", "Handheld Zebra Gun", "psk_hardware_secret", "barcode_scanner");

let scan = BarcodeScanEvent::new("scanner_01", "079357318921", "UPC-A")
    .location("Warehouse-Bay-3");

if registry.verify_signature(&scan.device_id, &scan.barcode, "psk_hardware_secret") {
    // Verified hardware scan feeds directly into central order & inventory state
}
```

---

## Dynamic Role-Based Access Control (RBAC)

```rust
use terminus::prelude::*;

let mut user = User::new("1", "Pratik", "pratik@terminus.dev", Role::Superadmin);

// 1. Direct grants
user.give_permission_to("orders.refund");

// 2. Explicit Deny Precedence (Overrides Superadmin privileges)
user.deny_permission("finance.export");

assert!(!user.can("finance.export")); // Denied: Explicit deny overrides Superadmin!
assert!(user.can("orders.view"));     // Allowed: Superadmin default grant
```

---

## The `terminus` Command-Line Tool

Install the CLI globally:

```bash
cargo install --path . --bin terminus
```

Available commands:

```bash
terminus new [name]                   # Scaffold a new application (interactive wizard or flags)
terminus studio                       # Launch terminal UI telemetry dashboard
terminus make:resource <Name>         # Scaffold an administrative resource
terminus make:module <Name>           # Scaffold an enterprise domain module
terminus make:model <Name>            # Scaffold an Active Record model
terminus make:repo <Name>             # Scaffold a SQLite repository implementation
terminus make:migration <name>        # Scaffold a timestamped SQL migration
terminus migrate                      # Run pending database migrations
terminus db:tables                    # Inspect database schema and statistics
terminus db:seed                      # Seed initial database records
terminus make:sdk [--lang=ts|dart|swift|openapi] # Export typed client SDK for mobile & frontend
terminus make:desktop [Name]          # Scaffold Tauri / desktop workstation offline bridge
terminus make:iot [DeviceId]          # Scaffold IoT hardware scanner & telemetry script
terminus make:job <Name>              # Scaffold an asynchronous job
terminus make:mail <Name>             # Scaffold a transactional mail message
terminus make:token <name>            # Issue a personal access token
terminus token:list                   # List active access tokens
terminus token:revoke <id>            # Revoke an access token
terminus cache:clear                  # Clear memory and Redis cache stores
terminus routes                       # List registered routes and API endpoints
terminus ws:serve                     # Start dedicated WebSocket service
terminus theme:preview                # Display theme color configurations
```

---

## Testing Framework

Terminus includes an expressive, chainable testing library:

```rust
use terminus::prelude::*;

#[test]
fn test_order_processing() {
    terminus_test("verifies order totals and status", || {
        expect(&order.status).to_be("paid")
            .and(&order.items).to_have_length(3);

        expect(order.total).to_be_greater_than(100);
    });
}
```

---

## Reference Examples

Run the curated examples directly from the repository:

```bash
# Minimal web application and admin panel starter
cargo run --example quickstart

# Universal multi-platform commerce and logistics hub
# (Web Admin, Mobile Push & SDKs, Desktop Offline Sync, IoT Barcode Scanner)
cargo run --example universal_wms
```

---

## License

MIT License © 2026 Pratik.
