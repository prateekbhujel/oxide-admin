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
- **Identity, Passkeys & Payments**: WebAuthn/Passkey registration and authentication, OAuth 2.0 social authentication (GitHub, Google, custom OIDC), and universal Payment Gateway webhook verification (Stripe, LemonSqueezy, PayPack).
- **Multi-Platform Ready**: Tailored endpoints and payloads for mobile push notifications (APNs / FCM), IoT device token telemetry ingestion, and desktop webview/Tauri IPC integration.
- **Dynamic Role-Based Access Control**: Granular direct permissions, explicit deny precedence overriding administrative privileges, wildcard hierarchies (`orders.*`), and 30-second TTL in-memory grant caching with instant invalidation on mutation.
- **Active Record ORM & Fluent Query Builder**: Expressive query chains with parameterized SQL compilation, schema migrations, and SQLite persistence.
- **Real-Time Collaboration & Telemetry**: Native Axum WebSocket broadcasting (`/admin/ws`), multi-admin presence, background queue worker, transactional mailer, and terminal UI telemetry dashboard (`terminus studio`).
- **Production Deployment Engine**: One-command generation of self-healing deployment bundles (`terminus make:cpanel`) with Apache reverse proxy configuration and POSIX watchdog supervisors.

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

Add Terminus to your `Cargo.toml`:

```toml
[dependencies]
terminus-framework = "0.1.0"
tokio = { version = "1", features = ["full"] }
serde_json = "1.0"
```

### 1. Minimal Web & API Application

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

## Global Ergonomic Helpers

Terminus provides zero-ceremony global helpers accessible everywhere in your application without passing context instances around:

```rust
use terminus::prelude::*;

pub async fn handle_checkout() -> Result<ApiResponse, AbortError> {
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

## Identity, Passkeys & Payments

Terminus includes built-in flows for modern authentication and payment gateway webhooks:

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

// 3. Payment Webhook Verification & Deserialization
let is_valid = PaymentWebhook::verify_signature(&payload, &signature_header, &webhook_secret);
if is_valid {
    if let Ok(event) = PaymentWebhook::parse_event(&payload) {
        match event {
            PaymentEvent::PaymentSucceeded { transaction_id, amount_cents, .. } => {
                println!("Payment received: {} cents for tx {}", amount_cents, transaction_id);
            }
            PaymentEvent::SubscriptionCreated { subscription_id, plan_id, .. } => {
                println!("New subscription {} on plan {}", subscription_id, plan_id);
            }
            _ => {}
        }
    }
}
```

---

## Multi-Platform Engine (Mobile, IoT, Desktop)

Deploy services consumable across mobile applications, embedded devices, and desktop environments:

```rust
use terminus::prelude::*;

// 1. Mobile Push Notification Envelope (FCM & APNs)
let notification = PushNotification::new("Order Shipped", "Your order #104 is on the way", "fcm_token_here")
    .with_data("tracking_code", "TRK-99201");
let fcm_json = notification.to_fcm_payload();

// 2. IoT Telemetry Packet
let telemetry = DeviceTelemetry::new("edge_device_42")
    .metric("temperature", 22.8)
    .metric("battery_level", 98.0);

// 3. Desktop Tauri / Webview IPC Message
let desktop_event = DesktopMessage::new("window_resize", serde_json::json!({ "width": 1280, "height": 800 }));
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
terminus studio                       # Launch terminal UI telemetry dashboard
terminus make:resource <Name>         # Scaffold an administrative resource
terminus make:module <Name>           # Scaffold an enterprise domain module
terminus make:model <Name>            # Scaffold an Active Record model
terminus make:repo <Name>             # Scaffold a SQLite repository implementation
terminus make:migration <name>        # Scaffold a timestamped SQL migration
terminus migrate                      # Run pending database migrations
terminus db:tables                    # Inspect database schema and statistics
terminus db:seed                      # Seed initial database records
terminus make:job <Name>              # Scaffold an asynchronous job
terminus make:mail <Name>             # Scaffold a transactional mail message
terminus make:token <name>            # Issue a personal access token
terminus token:list                   # List active access tokens
terminus token:revoke <id>            # Revoke an access token
terminus cache:clear                  # Clear memory and Redis cache stores
terminus make:cpanel                  # Scaffold cPanel / shared hosting reverse proxy bundle
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

## License

MIT License © 2026 Pratik.
