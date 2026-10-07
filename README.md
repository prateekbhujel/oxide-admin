# ⚡ OxideAdmin

> **Build fullstack apps & admin panels fast, for your bright ideas.**  
> With a solid Rust foundation and a polished UI, OxideAdmin handles your frontend and backend together so you can focus on what makes your product unique.

[![CI](https://github.com/prateekbhujel/oxide-admin/actions/workflows/rust.yml/badge.svg)](https://github.com/prateekbhujel/oxide-admin/actions/workflows/rust.yml)
[![Status: Alpha](https://img.shields.io/badge/status-alpha%20(v0.1)-yellow.svg)](https://github.com/prateekbhujel/oxide-admin)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-red.svg)](https://www.rust-lang.org)

---

## 💡 The Philosophy: Truly Fullstack

Most modern web development forces you to build two separate applications:
1. A backend API in Rust, Go, or Python.
2. A completely separate frontend in React, Next.js, or Vue, requiring `npm`, `package.json`, Vite configs, state managers, and hundreds of megabytes of `node_modules`.

**OxideAdmin unites frontend and backend into a single Rust codebase.** 

You define your models, repositories, and resources in pure Rust. OxideAdmin renders server-driven, Linear-grade interfaces with native modal dialogs, real-time live search, and full CRUD operations with zero frontend build steps.

---

## 🏗️ Clean Repository Architecture

OxideAdmin uses the **Repository Pattern** so your application code never locks into a specific database. Swap between SQLite, PostgreSQL, or in-memory testing by changing a single line:

```
              ┌──────────────────────────────────────┐
              │             UserResource             │
              └──────────────────┬───────────────────┘
                                 │
                    UserRepository Trait Interface
                                 │
         ┌───────────────────────┴───────────────────────┐
         ▼                                               ▼
┌───────────────────────────────┐       ┌───────────────────────────────┐
│     SqliteUserRepository      │  ...  │     PostgresUserRepository    │
│  (Persistent local oxide.db)  │       │  (Production Enterprise DB)   │
└───────────────────────────────┘       └───────────────────────────────┘
```

---

## 🚀 Quickstart

### 1. Add to `Cargo.toml`

While v0.1 is in active development, install directly from GitHub:

```toml
[dependencies]
oxide-admin = { git = "https://github.com/prateekbhujel/oxide-admin" }
axum = "0.7"
tokio = { version = "1", features = ["full"] }
rusqlite = { version = "0.32", features = ["bundled"] }
```

### 2. Define Your Resource

```rust
use oxide_admin::prelude::*;
use std::sync::Arc;

pub struct UserResource {
    repo: Arc<dyn UserRepository>,
}

impl UserResource {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self { repo }
    }
}

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
                ("Founder", "amber"),
                ("Admin", "blue"),
                ("Member", "slate"),
            ]))
            .column(Column::badge("status", vec![
                ("Active", "emerald"),
                ("Suspended", "rose"),
            ]))
            .page_size(8)
    }

    fn form(&self) -> Form {
        Form::new()
            .field(FormField::text("name").required().placeholder("Full Name"))
            .field(FormField::email("email").required().placeholder("email@example.com"))
    }

    fn fetch_rows(&self, query: &QueryState) -> (Vec<RowData>, usize) {
        let (users, total) = self.repo.list(query);
        let rows = users.into_iter().map(|u| {
            RowData::new(&u.id)
                .insert("id", &u.id)
                .insert("name", &u.name)
                .insert("email", &u.email)
                .insert("role", u.role.as_str())
                .insert("status", u.status.as_str())
                .insert("created_at", &u.created_at)
        }).collect();
        (rows, total)
    }

    fn delete_row(&self, id: &str) -> Result<(), String> {
        self.repo.delete(id)
    }
}
```

### 3. Mount to Axum

```rust
use axum::Router;
use oxide_admin::prelude::*;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() {
    // 1. Initialize SQLite database
    let db_conn = Arc::new(Mutex::new(
        rusqlite::Connection::open("oxide.db").unwrap()
    ));

    // 2. Initialize repository and resource
    let user_repo = Arc::new(SqliteUserRepository::new(db_conn).unwrap());
    let admin = AdminPanel::new()
        .register(UserResource::new(user_repo));

    // 3. Mount into Axum Router
    let app = Router::new()
        .nest("/admin", admin.into_router());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("OxideAdmin running at http://127.0.0.1:3000/admin");
    axum::serve(listener, app).await.unwrap();
}
```

---

## 🏃 Running the Demo

Clone the repo and run:

```bash
cargo run --example demo
```

Open **`http://127.0.0.1:3000/admin`** in your browser.  
Default credentials: `pratik.bhujel@oxideadmin.dev` / `admin123`.

---

## 🗺️ Roadmap

- [x] Axum 0.7 routing and session cookie authentication
- [x] Repository pattern with SQLite persistence (`rusqlite`)
- [x] Server-rendered HTML5 dialog modals (Create, Edit, Delete)
- [x] Live debounced search and pagination
- [ ] Procedural macro `#[derive(Resource)]` to eliminate boilerplate
- [ ] PostgreSQL and SeaORM repository adapters
- [ ] Bulk actions and CSV export queue
- [ ] File uploads with Cloudflare R2 / AWS S3 presigned URLs

---

## 📄 License

MIT License © 2026 Pratik Bhujel.
