use axum::Router;
use oxide_admin::prelude::*;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// --- Model 1: User Resource with Real Mutability ---
#[derive(Clone)]
struct UserData {
    id: String,
    name: String,
    email: String,
    role: String,
    status: String,
    created_at: String,
}

#[derive(Clone)]
pub struct UserResource {
    data: Arc<RwLock<Vec<UserData>>>,
}

impl UserResource {
    pub fn new() -> Self {
        let initial_users = vec![
            UserData {
                id: "1".to_string(),
                name: "Hari Bahadur Bhujel".to_string(),
                email: "hari.bhujel@oxideadmin.dev".to_string(),
                role: "Founder".to_string(),
                status: "Active".to_string(),
                created_at: "2026-09-01".to_string(),
            },
            UserData {
                id: "2".to_string(),
                name: "Pratik Bhujel".to_string(),
                email: "pratik.bhujel@oxideadmin.dev".to_string(),
                role: "Superadmin".to_string(),
                status: "Active".to_string(),
                created_at: "2026-09-02".to_string(),
            },
            UserData {
                id: "3".to_string(),
                name: "Dharma Raj Shrestha".to_string(),
                email: "dharma.shrestha@oxideadmin.dev".to_string(),
                role: "Admin".to_string(),
                status: "Active".to_string(),
                created_at: "2026-09-05".to_string(),
            },
            UserData {
                id: "4".to_string(),
                name: "Gokul Subedi".to_string(),
                email: "gokul.subedi@oxideadmin.dev".to_string(),
                role: "Admin".to_string(),
                status: "Active".to_string(),
                created_at: "2026-09-08".to_string(),
            },
            UserData {
                id: "5".to_string(),
                name: "Ranjan Gumanju".to_string(),
                email: "ranjan.gumanju@oxideadmin.dev".to_string(),
                role: "Member".to_string(),
                status: "Active".to_string(),
                created_at: "2026-09-12".to_string(),
            },
            UserData {
                id: "6".to_string(),
                name: "Sampanna Rimal".to_string(),
                email: "sampanna.rimal@oxideadmin.dev".to_string(),
                role: "Member".to_string(),
                status: "Active".to_string(),
                created_at: "2026-09-15".to_string(),
            },
            UserData {
                id: "7".to_string(),
                name: "Subash Ranabat".to_string(),
                email: "subash.ranabat@oxideadmin.dev".to_string(),
                role: "Member".to_string(),
                status: "Active".to_string(),
                created_at: "2026-09-19".to_string(),
            },
            UserData {
                id: "8".to_string(),
                name: "Lasta Chaudhary".to_string(),
                email: "lasta.chaudhary@oxideadmin.dev".to_string(),
                role: "Editor".to_string(),
                status: "Active".to_string(),
                created_at: "2026-09-22".to_string(),
            },
            UserData {
                id: "9".to_string(),
                name: "Ryan Koirala".to_string(),
                email: "ryan.koirala@oxideadmin.dev".to_string(),
                role: "Member".to_string(),
                status: "Pending".to_string(),
                created_at: "2026-10-01".to_string(),
            },
            UserData {
                id: "10".to_string(),
                name: "Alex Vance".to_string(),
                email: "alex.vance@example.com".to_string(),
                role: "Member".to_string(),
                status: "Suspended".to_string(),
                created_at: "2026-10-04".to_string(),
            },
        ];

        Self {
            data: Arc::new(RwLock::new(initial_users)),
        }
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
            .column(Column::text("created_at").label("Joined Date").sortable())
            .page_size(8)
    }

    fn form(&self) -> Form {
        Form::new()
            .field(FormField::text("name").required().placeholder("Full Name"))
            .field(FormField::email("email").required().placeholder("user@example.com"))
            .field(FormField::select("role", vec![
                ("Member", "Member"),
                ("Editor", "Editor"),
                ("Admin", "Admin"),
                ("Superadmin", "Superadmin"),
            ]))
            .field(FormField::select("status", vec![
                ("Active", "Active"),
                ("Pending", "Pending"),
                ("Suspended", "Suspended"),
            ]))
    }

    fn fetch_rows(&self, query: &QueryState) -> (Vec<RowData>, usize) {
        let store = self.data.read().unwrap();
        let mut filtered: Vec<&UserData> = store.iter().collect();

        if !query.search.is_empty() {
            let q = query.search.to_lowercase();
            filtered.retain(|u| {
                u.name.to_lowercase().contains(&q)
                    || u.email.to_lowercase().contains(&q)
                    || u.role.to_lowercase().contains(&q)
            });
        }

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

    fn get_row(&self, id: &str) -> Option<RowData> {
        let store = self.data.read().unwrap();
        store.iter().find(|u| u.id == id).map(|u| {
            RowData::new(&u.id)
                .insert("name", &u.name)
                .insert("email", &u.email)
                .insert("role", &u.role)
                .insert("status", &u.status)
        })
    }

    fn create_row(&self, values: HashMap<String, String>) -> Result<String, String> {
        let mut store = self.data.write().unwrap();
        let next_id = (store.len() + 1).to_string();
        let user = UserData {
            id: next_id.clone(),
            name: values.get("name").cloned().unwrap_or_else(|| "Unnamed".into()),
            email: values.get("email").cloned().unwrap_or_else(|| "noemail@example.com".into()),
            role: values.get("role").cloned().unwrap_or_else(|| "Member".into()),
            status: values.get("status").cloned().unwrap_or_else(|| "Active".into()),
            created_at: "2026-10-07".to_string(),
        };
        store.push(user);
        Ok(next_id)
    }

    fn update_row(&self, id: &str, values: HashMap<String, String>) -> Result<(), String> {
        let mut store = self.data.write().unwrap();
        if let Some(user) = store.iter_mut().find(|u| u.id == id) {
            if let Some(name) = values.get("name") { user.name = name.clone(); }
            if let Some(email) = values.get("email") { user.email = email.clone(); }
            if let Some(role) = values.get("role") { user.role = role.clone(); }
            if let Some(status) = values.get("status") { user.status = status.clone(); }
            Ok(())
        } else {
            Err("User not found".into())
        }
    }

    fn delete_row(&self, id: &str) -> Result<(), String> {
        let mut store = self.data.write().unwrap();
        store.retain(|u| u.id != id);
        Ok(())
    }
}

// --- Model 2: Order Resource with Real Mutability ---
#[derive(Clone)]
struct OrderData {
    id: String,
    customer: String,
    amount: String,
    status: String,
    date: String,
}

#[derive(Clone)]
pub struct OrderResource {
    data: Arc<RwLock<Vec<OrderData>>>,
}

impl OrderResource {
    pub fn new() -> Self {
        let initial_orders = vec![
            OrderData {
                id: "ORD-1001".to_string(),
                customer: "Hari Bahadur Bhujel".to_string(),
                amount: "$1,500.00".to_string(),
                status: "Paid".to_string(),
                date: "2026-10-01".to_string(),
            },
            OrderData {
                id: "ORD-1002".to_string(),
                customer: "Dharma Raj Shrestha".to_string(),
                amount: "$750.00".to_string(),
                status: "Paid".to_string(),
                date: "2026-10-02".to_string(),
            },
            OrderData {
                id: "ORD-1003".to_string(),
                customer: "Gokul Subedi".to_string(),
                amount: "$320.00".to_string(),
                status: "Paid".to_string(),
                date: "2026-10-03".to_string(),
            },
            OrderData {
                id: "ORD-1004".to_string(),
                customer: "Ranjan Gumanju".to_string(),
                amount: "$490.00".to_string(),
                status: "Pending".to_string(),
                date: "2026-10-04".to_string(),
            },
            OrderData {
                id: "ORD-1005".to_string(),
                customer: "Sampanna Rimal".to_string(),
                amount: "$850.00".to_string(),
                status: "Paid".to_string(),
                date: "2026-10-05".to_string(),
            },
            OrderData {
                id: "ORD-1006".to_string(),
                customer: "Subash Ranabat".to_string(),
                amount: "$600.00".to_string(),
                status: "Paid".to_string(),
                date: "2026-10-06".to_string(),
            },
            OrderData {
                id: "ORD-1007".to_string(),
                customer: "Lasta Chaudhary".to_string(),
                amount: "$210.00".to_string(),
                status: "Refunded".to_string(),
                date: "2026-10-07".to_string(),
            },
            OrderData {
                id: "ORD-1008".to_string(),
                customer: "Ryan Koirala".to_string(),
                amount: "$990.00".to_string(),
                status: "Paid".to_string(),
                date: "2026-10-07".to_string(),
            },
        ];

        Self {
            data: Arc::new(RwLock::new(initial_orders)),
        }
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
        let store = self.data.read().unwrap();
        let mut filtered: Vec<&OrderData> = store.iter().collect();

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
        let per_page = if query.per_page == 0 { 8 } else { query.per_page };
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

    fn get_row(&self, id: &str) -> Option<RowData> {
        let store = self.data.read().unwrap();
        store.iter().find(|o| o.id == id).map(|o| {
            RowData::new(&o.id)
                .insert("customer", &o.customer)
                .insert("amount", &o.amount)
                .insert("status", &o.status)
        })
    }

    fn create_row(&self, values: HashMap<String, String>) -> Result<String, String> {
        let mut store = self.data.write().unwrap();
        let next_id = format!("ORD-{}", 1000 + store.len() + 1);
        let order = OrderData {
            id: next_id.clone(),
            customer: values.get("customer").cloned().unwrap_or_else(|| "Customer".into()),
            amount: values.get("amount").cloned().unwrap_or_else(|| "$100.00".into()),
            status: values.get("status").cloned().unwrap_or_else(|| "Paid".into()),
            date: "2026-10-07".to_string(),
        };
        store.push(order);
        Ok(next_id)
    }

    fn update_row(&self, id: &str, values: HashMap<String, String>) -> Result<(), String> {
        let mut store = self.data.write().unwrap();
        if let Some(order) = store.iter_mut().find(|o| o.id == id) {
            if let Some(customer) = values.get("customer") { order.customer = customer.clone(); }
            if let Some(amount) = values.get("amount") { order.amount = amount.clone(); }
            if let Some(status) = values.get("status") { order.status = status.clone(); }
            Ok(())
        } else {
            Err("Order not found".into())
        }
    }

    fn delete_row(&self, id: &str) -> Result<(), String> {
        let mut store = self.data.write().unwrap();
        store.retain(|o| o.id != id);
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("info"))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let admin = AdminPanel::new()
        .register(UserResource::new())
        .register(OrderResource::new());

    let app = Router::new()
        .nest("/admin", admin.into_router())
        .route("/", axum::routing::get(|| async {
            axum::response::Redirect::temporary("/admin")
        }));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("\n========================================================");
    println!("⚡ OxideAdmin Production-Ready Server Active");
    println!("👉 Open: http://127.0.0.1:3000/admin");
    println!("🔑 Default Credentials: pratik.bhujel@oxideadmin.dev / admin123");
    println!("========================================================\n");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
