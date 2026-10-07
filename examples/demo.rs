use axum::Router;
use oxide_admin::prelude::*;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// --- Model 1: User Resource ---
pub struct UserResource {
    data: Vec<UserData>,
}

#[derive(Clone)]
struct UserData {
    id: String,
    name: String,
    email: String,
    role: String,
    status: String,
    created_at: String,
}

impl UserResource {
    pub fn new() -> Self {
        let roles = ["Admin", "Editor", "Member"];
        let statuses = ["Active", "Pending", "Suspended"];
        let first_names = ["Taylor", "Alex", "Jordan", "Pratik", "Elena", "Marcus", "Sophia", "David", "Aria", "Liam"];
        let last_names = ["Otwell", "Vance", "Smith", "Bhujel", "Rostova", "Chen", "Miller", "Kowalski", "Dupont", "Nakamoto"];

        let mut data = Vec::new();

        // Lead Administrator / Founder
        data.push(UserData {
            id: "1".to_string(),
            name: "Hari Bahadur Bhujel".to_string(),
            email: "hari.bhujel@oxideadmin.dev".to_string(),
            role: "Founder".to_string(),
            status: "Active".to_string(),
            created_at: "2026-10-01".to_string(),
        });

        for i in 2..=35 {
            let fn_idx = (i * 3) % first_names.len();
            let ln_idx = (i * 7) % last_names.len();
            let name = format!("{} {}", first_names[fn_idx], last_names[ln_idx]);
            let email = format!("{}.{}@example.com", first_names[fn_idx].to_lowercase(), last_names[ln_idx].to_lowercase());
            let role = roles[i % roles.len()].to_string();
            let status = statuses[(i * 2) % statuses.len()].to_string();
            let created_at = format!("2026-10-{:02}", (i % 28) + 1);

            data.push(UserData {
                id: i.to_string(),
                name,
                email,
                role,
                status,
                created_at,
            });
        }

        Self { data }
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

    fn table(&self) -> Table {
        Table::new()
            .column(Column::text("id").label("ID").sortable())
            .column(Column::text("name").label("Full Name").searchable().sortable())
            .column(Column::text("email").label("Email Address").searchable())
            .column(Column::badge("role", vec![
                ("Founder", "amber"),
                ("Admin", "indigo"),
                ("Editor", "blue"),
                ("Member", "slate"),
            ]))
            .column(Column::badge("status", vec![
                ("Active", "emerald"),
                ("Pending", "amber"),
                ("Suspended", "rose"),
            ]))
            .column(Column::text("created_at").label("Joined Date").sortable())
            .action(TableAction::new("edit", "Edit"))
            .action(TableAction::new("delete", "Delete").danger())
            .page_size(8)
    }

    fn fetch_rows(&self, query: &QueryState) -> (Vec<RowData>, usize) {
        let mut filtered: Vec<&UserData> = self.data.iter().collect();

        // Search filtering
        if !query.search.is_empty() {
            let q = query.search.to_lowercase();
            filtered.retain(|u| {
                u.name.to_lowercase().contains(&q)
                    || u.email.to_lowercase().contains(&q)
                    || u.role.to_lowercase().contains(&q)
            });
        }

        // Sorting
        if let Some(ref sort_col) = query.sort_by {
            filtered.sort_by(|a, b| {
                let cmp = match sort_col.as_str() {
                    "id" => a.id.parse::<usize>().unwrap_or(0).cmp(&b.id.parse::<usize>().unwrap_or(0)),
                    "name" => a.name.cmp(&b.name),
                    "created_at" => a.created_at.cmp(&b.created_at),
                    _ => std::cmp::Ordering::Equal,
                };
                if query.sort_desc { cmp.reverse() } else { cmp }
            });
        }

        let total = filtered.len();
        let page = if query.page == 0 { 1 } else { query.page };
        let per_page = if query.per_page == 0 { 8 } else { query.per_page };
        let skip = (page - 1) * per_page;

        let rows = filtered
            .into_iter()
            .skip(skip)
            .take(per_page)
            .map(|u| {
                RowData::new(&u.id)
                    .insert("id", &u.id)
                    .insert("name", &u.name)
                    .insert("email", &u.email)
                    .insert("role", &u.role)
                    .insert("status", &u.status)
                    .insert("created_at", &u.created_at)
            })
            .collect();

        (rows, total)
    }
}

// --- Model 2: Order Resource ---
pub struct OrderResource {
    data: Vec<OrderData>,
}

#[derive(Clone)]
struct OrderData {
    id: String,
    customer: String,
    amount: String,
    status: String,
    date: String,
}

impl OrderResource {
    pub fn new() -> Self {
        let customers = ["Acme Corp", "Stripe Inc", "Linear App", "Vercel Ltd", "GitHub Inc", "Supabase"];
        let statuses = ["Paid", "Pending", "Refunded"];
        let mut data = Vec::new();

        for i in 1..=40 {
            let cust = customers[i % customers.len()];
            let amount = format!("${}.00", 99 + (i * 45));
            let status = statuses[i % statuses.len()].to_string();
            let date = format!("2026-10-{:02}", (i % 28) + 1);

            data.push(OrderData {
                id: format!("ORD-{:04}", 1000 + i),
                customer: cust.to_string(),
                amount,
                status,
                date,
            });
        }

        Self { data }
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

    fn table(&self) -> Table {
        Table::new()
            .column(Column::text("id").label("Order Number").searchable().sortable())
            .column(Column::text("customer").label("Customer").searchable())
            .column(Column::text("amount").label("Total Amount").sortable())
            .column(Column::badge("status", vec![
                ("Paid", "emerald"),
                ("Pending", "amber"),
                ("Refunded", "rose"),
            ]))
            .column(Column::text("date").label("Invoice Date").sortable())
            .action(TableAction::new("view", "View Invoice"))
            .page_size(10)
    }

    fn fetch_rows(&self, query: &QueryState) -> (Vec<RowData>, usize) {
        let mut filtered: Vec<&OrderData> = self.data.iter().collect();

        if !query.search.is_empty() {
            let q = query.search.to_lowercase();
            filtered.retain(|o| {
                o.id.to_lowercase().contains(&q)
                    || o.customer.to_lowercase().contains(&q)
                    || o.status.to_lowercase().contains(&q)
            });
        }

        let total = filtered.len();
        let page = if query.page == 0 { 1 } else { query.page };
        let per_page = if query.per_page == 0 { 10 } else { query.per_page };
        let skip = (page - 1) * per_page;

        let rows = filtered
            .into_iter()
            .skip(skip)
            .take(per_page)
            .map(|o| {
                RowData::new(&o.id)
                    .insert("id", &o.id)
                    .insert("customer", &o.customer)
                    .insert("amount", &o.amount)
                    .insert("status", &o.status)
                    .insert("date", &o.date)
            })
            .collect();

        (rows, total)
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("info"))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 1. Create and configure AdminPanel
    let admin = AdminPanel::new()
        .register(UserResource::new())
        .register(OrderResource::new());

    // 2. Mount into standard Axum Router
    let app = Router::new()
        .nest("/admin", admin.into_router())
        .route("/", axum::routing::get(|| async {
            axum::response::Redirect::temporary("/admin")
        }));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("\n========================================================");
    println!("⚡ OxideAdmin Live Demo Server Running!");
    println!("👉 Visit: http://127.0.0.1:3000/admin");
    println!("========================================================\n");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
