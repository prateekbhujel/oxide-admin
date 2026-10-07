# ⚡ OxideAdmin

> **The declarative, ultra-fast Admin & CRUD engine for Rust and Axum.**  
> Zero JavaScript build tools. Zero React/Node context-switching. Single static binary.

[![Crates.io](https://img.shields.io/badge/crates.io-v0.1.0-orange.svg)](https://crates.io)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-red.svg)](https://www.rust-lang.org)

---

## 💡 Why OxideAdmin?

In web ecosystems like Laravel and Rails, tools like **Filament** and **ActiveAdmin** allowed solo developers and teams to build complex admin dashboards, data tables, and back-office tools in minutes.

In Rust, teams are frequently forced to:
1. Spin up a separate Node.js / Vite / React / Next.js repository.
2. Manually write REST API endpoints, serializers, and CORS handlers.
3. Deal with 5,000+ lines of fragile UI glue code for basic tables, pagination, and forms.

**OxideAdmin eliminates the frontend tax completely.** Define your resource in 20 lines of Rust, mount it into your `axum::Router`, and get an Apple/Linear-grade dark-mode dashboard running on **~15MB of RAM** with sub-millisecond response times.

---

## 🚀 Quickstart

### 1. Add to `Cargo.toml`

```toml
[dependencies]
oxide-admin = "0.1"
axum = "0.7"
tokio = { version = "1", features = ["full"] }
```

### 2. Define a Resource

```rust
use oxide_admin::prelude::*;

pub struct UserResource;

impl Resource for UserResource {
    fn name(&self) -> &str { "User" }
    fn plural_name(&self) -> &str { "Users" }
    fn slug(&self) -> &str { "users" }

    fn table(&self) -> Table {
        Table::new()
            .column(Column::text("id").label("ID").sortable())
            .column(Column::text("name").label("Name").searchable().sortable())
            .column(Column::text("email").label("Email").searchable())
            .column(Column::badge("role", vec![
                ("Admin", "indigo"),
                ("Member", "slate"),
            ]))
            .column(Column::badge("status", vec![
                ("Active", "emerald"),
                ("Suspended", "rose"),
            ]))
            .action(TableAction::new("edit", "Edit"))
            .action(TableAction::new("delete", "Delete").danger())
    }

    fn fetch_rows(&self, query: &QueryState) -> (Vec<RowData>, usize) {
        // Query your database (SQLx, Diesel, SeaORM) or in-memory state
        let rows = vec![/* ... */];
        (rows, 100)
    }
}
```

### 3. Mount to Axum

```rust
use axum::Router;
use oxide_admin::prelude::*;

#[tokio::main]
async fn main() {
    let admin = AdminPanel::new()
        .register(UserResource);

    let app = Router::new()
        .nest("/admin", admin.into_router());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

---

## 🎯 Features

- **⚡ Blazing Fast**: Microsecond render latency, powered directly by native Rust and Axum.
- **🎨 Modern Dark UI**: Linear/Tailwind aesthetic with zero CSS setup.
- **🔄 Live Reactive Updates**: Real-time debounced search, column sorting, and pagination without full page reloads.
- **📦 Single Binary Deployment**: Zero `node_modules`, zero npm build steps. Compiles into a single production binary.
- **🛡️ Type-Safe**: Zero runtime template errors; backed by Rust's strict type system.

---

## 🏃 Running the Demo

Clone the repo and run:

```bash
cargo run --example demo
```

Then open `http://127.0.0.1:3000/admin` in your browser.

---

## 📄 License

MIT License © 2026 Pratik Bhujel.
