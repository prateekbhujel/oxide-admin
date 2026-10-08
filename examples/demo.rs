use axum::Router;
use oxide_admin::prelude::*;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// =========================================================================
// User Resource (Powered by UserRepository)
// =========================================================================

pub struct UserResource {
    repo: Arc<dyn UserRepository>,
}

impl UserResource {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self { repo }
    }
}

impl Resource for UserResource {
    fn name(&self) -> &str {
        "User"
    }

    fn plural_name(&self) -> &str {
        "Users"
    }

    fn slug(&self) -> &str {
        "users"
    }

    fn form_mode(&self) -> FormMode {
        FormMode::Page
    }

    fn table(&self) -> Table {
        Table::new()
            .striped()
            .filter(TableFilter::select("role", "Role", vec![
                ("Superadmin", "Superadmin"),
                ("Admin", "Admin"),
                ("Editor", "Editor"),
                ("Member", "Member"),
            ]))
            .filter(TableFilter::select("status", "Status", vec![
                ("Active", "Active"),
                ("Pending", "Pending"),
                ("Suspended", "Suspended"),
            ]))
            .column(Column::text("id").label("ID").sortable())
            .column(Column::text("name").label("Full Name").searchable().sortable())
            .column(Column::text("email").label("Email Address").searchable())
            .column(Column::badge("role", vec![
                ("Founder", "amber"),
                ("Superadmin", "indigo"),
                ("Admin", "blue"),
                ("Editor", "blue"),
                ("Member", "slate"),
            ]))
            .column(Column::badge("status", vec![
                ("Active", "emerald"),
                ("Pending", "amber"),
                ("Suspended", "rose"),
            ]))
            .column(Column::text("createdAt").label("Joined Date").sortable())
            .pageSize(8)
    }

    fn form(&self) -> Form {
        Form::new()
            .field(FormField::text("name").required().placeholder("Full Name"))
            .field(FormField::email("email").required().placeholder("user@example.com"))
            .field(FormField::searchableSelect("role", vec![
                ("Member", "Member (Standard Team Account)"),
                ("Editor", "Editor (Orders & Content Management)"),
                ("Admin", "Admin (Team Management)"),
                ("Superadmin", "Superadmin (Full Control)"),
            ]))
            .field(FormField::select("status", vec![
                ("Active", "Active"),
                ("Pending", "Pending"),
                ("Suspended", "Suspended"),
            ]))
    }

    fn fetch_rows(&self, query: &QueryState) -> (Vec<RowData>, usize) {
        let (mut users, total) = self.repo.list(query);
        if let Some(role_filter) = query.filters.get("role") {
            if !role_filter.is_empty() {
                users.retain(|u| u.role.as_str() == role_filter);
            }
        }
        if let Some(status_filter) = query.filters.get("status") {
            if !status_filter.is_empty() {
                users.retain(|u| u.status.as_str() == status_filter);
            }
        }
        let total_count = if query.filters.is_empty() { total } else { users.len() };
        let rows = users
            .into_iter()
            .map(|u| {
                RowData::new(&u.id)
                    .insert("id", &u.id)
                    .insert("name", &u.name)
                    .insert("email", &u.email)
                    .insert("role", u.role.as_str())
                    .insert("status", u.status.as_str())
                    .insert("createdAt", &u.created_at)
            })
            .collect();
        (rows, total_count)
    }

    fn get_row(&self, id: &str) -> Option<RowData> {
        self.repo.find_by_id(id).map(|u| {
            RowData::new(&u.id)
                .insert("name", &u.name)
                .insert("email", &u.email)
                .insert("role", u.role.as_str())
                .insert("status", u.status.as_str())
        })
    }

    fn create_row(&self, values: HashMap<String, String>) -> Result<String, String> {
        let name = values.get("name").cloned().unwrap_or_else(|| "Unnamed".into());
        let email = values.get("email").cloned().unwrap_or_else(|| "noemail@example.com".into());
        let role = Role::from_str_loose(values.get("role").map(|s| s.as_str()).unwrap_or("Member"));
        let status = UserStatus::from_str_loose(values.get("status").map(|s| s.as_str()).unwrap_or("Active"));

        // Generate auto ID
        let (_, total) = self.repo.list(&QueryState::default());
        let new_id = (total + 1).to_string();

        let user = User {
            id: new_id,
            name,
            email,
            role,
            status,
            created_at: "2026-10-07".to_string(),
        };

        self.repo.save(user)
    }

    fn update_row(&self, id: &str, values: HashMap<String, String>) -> Result<(), String> {
        let name = values.get("name").cloned().unwrap_or_default();
        let email = values.get("email").cloned().unwrap_or_default();
        let role = Role::from_str_loose(values.get("role").map(|s| s.as_str()).unwrap_or("Member"));
        let status = UserStatus::from_str_loose(values.get("status").map(|s| s.as_str()).unwrap_or("Active"));

        self.repo.update(id, name, email, role, status)
    }

    fn delete_row(&self, id: &str) -> Result<(), String> {
        self.repo.delete(id)
    }
}

// =========================================================================
// Order Resource (Powered by OrderRepository)
// =========================================================================

pub struct OrderResource {
    repo: Arc<dyn OrderRepository>,
}

impl OrderResource {
    pub fn new(repo: Arc<dyn OrderRepository>) -> Self {
        Self { repo }
    }
}

impl Resource for OrderResource {
    fn name(&self) -> &str {
        "Order"
    }

    fn plural_name(&self) -> &str {
        "Orders"
    }

    fn slug(&self) -> &str {
        "orders"
    }

    fn form_mode(&self) -> FormMode {
        FormMode::SlideOver
    }

    fn table(&self) -> Table {
        Table::new()
            .compact()
            .filter(TableFilter::select("status", "Status", vec![
                ("Paid", "Paid"),
                ("Pending", "Pending"),
                ("Refunded", "Refunded"),
            ]))
            .column(Column::text("id").label("Order Number").searchable().sortable())
            .column(Column::text("customer").label("Customer").searchable())
            .column(Column::text("amount").label("Amount").sortable())
            .column(Column::badge("status", vec![
                ("Paid", "emerald"),
                ("Pending", "amber"),
                ("Refunded", "rose"),
            ]))
            .column(Column::text("date").label("Invoice Date").sortable())
            .page_size(8)
    }

    fn form(&self) -> Form {
        Form::new()
            .field(FormField::text("customer").required().placeholder("Customer Name"))
            .field(FormField::text("amount").required().placeholder("$150.00"))
            .field(FormField::select("status", vec![
                ("Paid", "Paid"),
                ("Pending", "Pending"),
                ("Refunded", "Refunded"),
            ]))
    }

    fn fetch_rows(&self, query: &QueryState) -> (Vec<RowData>, usize) {
        let (mut orders, total) = self.repo.list(query);
        if let Some(status_filter) = query.filters.get("status") {
            if !status_filter.is_empty() {
                orders.retain(|o| o.status.as_str() == status_filter);
            }
        }
        let total_count = if query.filters.is_empty() { total } else { orders.len() };
        let rows = orders
            .into_iter()
            .map(|o| {
                RowData::new(&o.id)
                    .insert("id", &o.id)
                    .insert("customer", &o.customer)
                    .insert("amount", &o.amount)
                    .insert("status", o.status.as_str())
                    .insert("date", &o.date)
            })
            .collect();
        (rows, total_count)
    }

    fn get_row(&self, id: &str) -> Option<RowData> {
        self.repo.find_by_id(id).map(|o| {
            RowData::new(&o.id)
                .insert("customer", &o.customer)
                .insert("amount", &o.amount)
                .insert("status", o.status.as_str())
        })
    }

    fn create_row(&self, values: HashMap<String, String>) -> Result<String, String> {
        let customer = values.get("customer").cloned().unwrap_or_else(|| "Customer".into());
        let amount = values.get("amount").cloned().unwrap_or_else(|| "$100.00".into());
        let status = OrderStatus::from_str_loose(values.get("status").map(|s| s.as_str()).unwrap_or("Paid"));

        let (_, total) = self.repo.list(&QueryState::default());
        let new_id = format!("ORD-{}", 1000 + total + 1);

        let order = Order {
            id: new_id,
            customer,
            amount,
            status,
            date: "2026-10-07".to_string(),
        };

        self.repo.save(order)
    }

    fn update_row(&self, id: &str, values: HashMap<String, String>) -> Result<(), String> {
        let customer = values.get("customer").cloned().unwrap_or_default();
        let amount = values.get("amount").cloned().unwrap_or_default();
        let status = OrderStatus::from_str_loose(values.get("status").map(|s| s.as_str()).unwrap_or("Paid"));

        self.repo.update(id, customer, amount, status)
    }

    fn delete_row(&self, id: &str) -> Result<(), String> {
        self.repo.delete(id)
    }
}

// =========================================================================
// Main Server Entrypoint
// =========================================================================

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("info"))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Open or create SQLite connection to local file "oxide.db"
    let db_conn = Arc::new(std::sync::Mutex::new(
        rusqlite::Connection::open("oxide.db").expect("Failed to open SQLite database oxide.db"),
    ));

    // 1. Initialize Repositories (Powered by real SQLite database)
    let user_repo = Arc::new(SqliteUserRepository::new(db_conn.clone()).unwrap());
    let order_repo = Arc::new(SqliteOrderRepository::new(db_conn.clone()).unwrap());
    let audit_repo = Arc::new(SqliteAuditRepository::new(db_conn).unwrap());

    // 2. Initialize Resources with injected Repositories
    let user_resource = UserResource::new(user_repo.clone());
    let order_resource = OrderResource::new(order_repo);
    let audit_resource = AuditLogResource::new(audit_repo.clone());

    // 3. Register into AdminPanel with RBAC user store & audit logger
    let admin = AdminPanel::new()
        .users(user_repo)
        .audit(audit_repo)
        .register(user_resource)
        .register(order_resource)
        .register(audit_resource);

    // 4. Mount into Axum Router
    let app = Router::new()
        .nest("/admin", admin.into_router())
        .route("/", axum::routing::get(|| async {
            axum::response::Redirect::temporary("/admin")
        }));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("\n========================================================");
    println!("OxideAdmin Server running at http://127.0.0.1:3000/admin");
    println!("Demo Accounts (Password for all: admin123):");
    println!("  • Superadmin : pratik.bhujel@oxideadmin.dev (Full Access)");
    println!("  • Admin      : dharma.shrestha@oxideadmin.dev (Manage users & orders)");
    println!("  • Editor     : lasta.chaudhary@oxideadmin.dev (Manage orders, view users)");
    println!("  • Member     : ranjan.gumanju@oxideadmin.dev (Read-only)");
    println!("========================================================\n");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
