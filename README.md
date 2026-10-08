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

---

## 🛡️ Declarative Roles & Resource Policies (RBAC)

OxideAdmin includes built-in, type-safe Role-Based Access Control inspired by Laravel Gates and Spatie Permissions:

- **Roles & Permissions**: Fine-grained permissions (`users.view`, `users.create`, `orders.delete`, `audit.view`, etc.) configured per role (`Founder`, `Superadmin`, `Admin`, `Editor`, `Member`).
- **Resource Policy Hooks**: Override `can_view`, `can_create`, `can_edit`, and `can_delete` on any `Resource`.
- **Permission-Driven UI**: Unauthorized buttons (`New Record`, `Edit`, `Delete`) are omitted server-side. Direct unauthorized POST requests return `403 Forbidden`.

```rust
impl Resource for OrderResource {
    // Only users with 'orders.delete' can see or execute delete
    fn can_delete(&self, user: &User) -> bool {
        user.can("orders.delete")
    }
}
```

---

## 📜 Activity Audit Trail

Track system mutations automatically. Register the `AuditLogResource` to view immutable audit events stored in SQLite:

```rust
let audit_repo = Arc::new(SqliteAuditRepository::new(db_conn.clone()).unwrap());

let admin = AdminPanel::new()
    .users(user_repo)
    .audit(audit_repo.clone())
    .register(UserResource::new(user_repo))
    .register(AuditLogResource::new(audit_repo));
```

---

## 📊 Executive Dashboard & Live KPIs

OxideAdmin greets you with an executive operations dashboard (`/admin/dashboard`):
- **Live KPI Metric Cards**: Gross Revenue calculation, total orders processed, team member counts, and audit security events.
- **Recent Audit Stream**: Real-time mutation log showing who did what and when.
- **Quick Operations**: Fast jump links directly into resource drawers and policy-gated views.

---

## 🪟 Flexible Form Modes: Modal, Slide-Over, or Dedicated Page

Not all forms fit in a popup modal. OxideAdmin gives each resource control over how its forms render via `form_mode()`:

```rust
impl Resource for OrderResource {
    // Choose between: FormMode::Modal, FormMode::SlideOver, or FormMode::Page
    fn form_mode(&self) -> FormMode {
        FormMode::SlideOver // Renders a right-anchored slide-in drawer
    }
}
```

- **`FormMode::Modal`**: Centered dialog modal, ideal for quick 2-3 field actions.
- **`FormMode::SlideOver`**: Smooth off-canvas drawer anchored on the right edge, perfect for dense forms without losing table context.
- **`FormMode::Page`**: Dedicated full-page route at `/admin/:slug/create` and `/admin/:slug/edit/:id` with breadcrumb navigation.

---

## 🔍 Fluent Table Filters & Customizable Styles

Easily add dropdown filter menus and customize row presentation:

```rust
Table::new()
    .striped() // or .compact()
    .filter(TableFilter::select("role", "Role", vec![
        ("Admin", "Admin"),
        ("Editor", "Editor"),
        ("Member", "Member"),
    ]))
    .filter(TableFilter::select("status", "Status", vec![
        ("Active", "Active"),
        ("Pending", "Pending"),
    ]))
```

---

## ⚡ Native Searchable Select Dropdowns (Select2-Style)

Select from long lists with zero external JavaScript dependencies or npm bloat:

```rust
Form::new()
    .field(FormField::searchable_select("assignee", vec![
        ("1", "Pratik Bhujel (Superadmin)"),
        ("2", "Dharma Raj Shrestha (Admin)"),
        ("3", "Lasta Chaudhary (Editor)"),
    ]))
```

## 🐪 Laravel-Grade Developer Experience: camelCase Everywhere

Inspired by Laravel and Inertia, OxideAdmin supports first-class `camelCase` across every layer:

- **Bidirectional Key Resolution**: `RowData::get("createdAt")` and `RowData::get("created_at")` both work transparently, so you never get bitten by casing mismatches.
- **Auto-Headline Labels**: Defining `Column::text("createdAt")` automatically renders `"Created At"` as the table header label without manual configuration.
- **camelCase Query Parameters**: URL parameters use modern web standards (`?sortBy=name&sortDesc=true&filterRole=Admin`). Legacy `sort_by` and `filter_role` remain fully supported.
- **Fluent camelCase Aliases**: Use Laravel-style camelCase methods on builders:
  ```rust
  Table::new().pageSize(10).defaultSortBy("createdAt")
  FormField::searchableSelect("role", options)
  ```

---

## ⚡ Headless REST JSON API (`/admin/:slug/api`)

Every resource automatically serves a clean, paginated `camelCase` JSON API:

```bash
curl -H "Cookie: oxide_session=..." \
  "http://127.0.0.1:3000/admin/users/api?sortBy=name&sortDesc=true&filterRole=Superadmin"
```

```json
{
  "data": [
    {
      "id": "2",
      "name": "Pratik Bhujel",
      "email": "pratik.bhujel@oxideadmin.dev",
      "role": "Superadmin",
      "status": "Active",
      "createdAt": "2026-09-02"
    }
  ],
  "page": 1,
  "perPage": 8,
  "sortBy": "name",
  "sortDesc": true,
  "total": 1
}
```

This allows OxideAdmin to power both server-rendered HTML admin panels and headless frontends (Inertia.js, React, Vue, Svelte, or mobile apps) from the same Rust codebase.

---

## 🔐 OAuth SSO & Role Switcher

In addition to email/password authentication, OxideAdmin supports single-sign-on (SSO) integration with GitHub OAuth (`/admin/oauth/github`), as well as a one-click demo role switcher to test permission levels on the fly.

---

## 🏃 Running the Demo

Clone the repo and run:

```bash
cargo run --example demo
```

Open **`http://127.0.0.1:3000/admin`** in your browser.  
Demo accounts (password for all: `admin123`):
- **Superadmin**: `pratik.bhujel@oxideadmin.dev` (Full access)
- **Admin**: `dharma.shrestha@oxideadmin.dev` (Manage users and orders)
- **Editor**: `lasta.chaudhary@oxideadmin.dev` (Manage orders, read-only users)
- **Member**: `ranjan.gumanju@oxideadmin.dev` (Read-only access)

---

## 🗺️ Roadmap

- [x] Axum 0.7 routing and session cookie authentication
- [x] Repository pattern with SQLite persistence (`rusqlite`)
- [x] Executive KPI operations dashboard (`/admin/dashboard`)
- [x] Server-rendered dialog modals, slide-over drawers, and full-page forms (`FormMode`)
- [x] Fluent table filters and custom styling (`TableStyle::Striped`, `TableStyle::Compact`)
- [x] Native searchable select dropdowns (`FormField::searchable_select`)
- [x] Live debounced search and pagination
- [x] Role-Based Access Control (RBAC) with declarative resource policies
- [x] Permission-driven UI rendering with 403 Forbidden security guards
- [x] Activity audit trail logging with SQLite persistence (`AuditLogResource`)
- [x] GitHub OAuth SSO integration
- [x] Laravel-grade camelCase DX (bidirectional key resolution, auto-headline)
- [x] Headless REST JSON API endpoint (`/:slug/api`) for Inertia & SPAs
- [ ] Procedural macro `#[derive(Resource)]` to eliminate boilerplate
- [ ] PostgreSQL and SeaORM repository adapters
- [ ] Bulk actions and CSV export queue
- [ ] File uploads with Cloudflare R2 / AWS S3 presigned URLs

---

## 📄 License

MIT License © 2026 Pratik Bhujel.
