use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use terminus::prelude::*;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

// =========================================================================
// Domain Entities & Repositories
// =========================================================================

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProductRecord {
    pub id: String,
    pub title: String,
    pub sku: String,
    pub price: i64,
    pub stock: i64,
    pub category: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CustomerRecord {
    pub id: String,
    pub name: String,
    pub email: String,
    pub plan: String,
    pub total_spent: i64,
    pub status: String,
    pub created_at: String,
}

pub struct ProductStore {
    conn: Arc<Mutex<rusqlite::Connection>>,
}

impl ProductStore {
    pub fn new(conn: Arc<Mutex<rusqlite::Connection>>) -> Self {
        Self { conn }
    }

    pub fn list(&self, query: &QueryState) -> (Vec<ProductRecord>, usize) {
        let lock = self.conn.lock().unwrap();
        let mut sql = "SELECT id, title, sku, price, stock, category, status, created_at FROM products WHERE 1=1".to_string();
        let mut count_sql = "SELECT COUNT(*) FROM products WHERE 1=1".to_string();

        if !query.search.is_empty() {
            let clause = format!(" AND (title LIKE '%{0}%' OR sku LIKE '%{0}%')", query.search);
            sql.push_str(&clause);
            count_sql.push_str(&clause);
        }

        if let Some(status) = query.filters.get("status") {
            if !status.is_empty() {
                let clause = format!(" AND status = '{}'", status);
                sql.push_str(&clause);
                count_sql.push_str(&clause);
            }
        }

        let total: usize = lock
            .query_row(&count_sql, [], |row| row.get(0))
            .unwrap_or(0);

        if let Some(ref sort_col) = query.sort_by {
            let dir = if query.sort_desc { "DESC" } else { "ASC" };
            sql.push_str(&format!(" ORDER BY {} {}", sort_col, dir));
        } else {
            sql.push_str(" ORDER BY created_at DESC");
        }

        let per_page = if query.per_page == 0 { 20 } else { query.per_page };
        let page = if query.page == 0 { 1 } else { query.page };
        let offset = (page - 1) * per_page;
        sql.push_str(&format!(" LIMIT {} OFFSET {}", per_page, offset));

        let mut stmt = lock.prepare(&sql).unwrap();
        let rows = stmt
            .query_map([], |row| {
                Ok(ProductRecord {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    sku: row.get(2)?,
                    price: row.get(3)?,
                    stock: row.get(4)?,
                    category: row.get(5)?,
                    status: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        (rows, total)
    }

    pub fn find(&self, id: &str) -> Option<ProductRecord> {
        let lock = self.conn.lock().unwrap();
        lock.query_row(
            "SELECT id, title, sku, price, stock, category, status, created_at FROM products WHERE id = ?1",
            [id],
            |row| {
                Ok(ProductRecord {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    sku: row.get(2)?,
                    price: row.get(3)?,
                    stock: row.get(4)?,
                    category: row.get(5)?,
                    status: row.get(6)?,
                    created_at: row.get(7)?,
                })
            },
        ).ok()
    }

    pub fn create(&self, title: &str, sku: &str, price: i64, stock: i64, category: &str, status: &str) -> Result<String, String> {
        let lock = self.conn.lock().unwrap();
        let id = format!("prod_{}", uuid_fast());
        let now = "2026-10-11";
        lock.execute(
            "INSERT INTO products (id, title, sku, price, stock, category, status, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![id, title, sku, price, stock, category, status, now],
        ).map_err(|e| e.to_string())?;
        Ok(id)
    }

    pub fn update(&self, id: &str, title: &str, sku: &str, price: i64, stock: i64, category: &str, status: &str) -> Result<(), String> {
        let lock = self.conn.lock().unwrap();
        lock.execute(
            "UPDATE products SET title = ?1, sku = ?2, price = ?3, stock = ?4, category = ?5, status = ?6 WHERE id = ?7",
            rusqlite::params![title, sku, price, stock, category, status, id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn delete(&self, id: &str) -> Result<(), String> {
        let lock = self.conn.lock().unwrap();
        lock.execute("DELETE FROM products WHERE id = ?1", [id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

// =========================================================================
// Product Resource (Slide-Over & Sections Layout)
// =========================================================================

pub struct ProductResource {
    store: Arc<ProductStore>,
}

impl ProductResource {
    pub fn new(store: Arc<ProductStore>) -> Self {
        Self { store }
    }
}

impl Resource for ProductResource {
    fn name(&self) -> &str {
        "Product"
    }

    fn plural_name(&self) -> &str {
        "Products"
    }

    fn slug(&self) -> &str {
        "products"
    }

    fn form_mode(&self) -> FormMode {
        FormMode::SlideOver
    }

    fn table(&self) -> Table {
        Table::new()
            .striped()
            .pageSize(10)
            .defaultSortBy("created_at")
            .filter(TableFilter::select("status", "Status", vec![
                ("Active", "Active"),
                ("Low Stock", "Low Stock"),
                ("Archived", "Archived"),
            ]))
            .column(Column::text("id").label("Product ID").sortable())
            .column(Column::text("title").label("Title").searchable().sortable())
            .column(Column::text("sku").label("SKU").searchable())
            .column(Column::numeric("price").label("Price ($)").sortable())
            .column(Column::numeric("stock").label("Stock Quantity").sortable())
            .column(Column::badge("status", vec![
                ("Active", "emerald"),
                ("Low Stock", "amber"),
                ("Archived", "slate"),
            ]))
            .column(Column::text("category").label("Category").sortable())
            .actions(vec![
                Action::view(),
                Action::edit(),
                Action::replicate(),
                Action::delete(),
            ])
            .bulkActions(vec![
                BulkAction::export(),
                BulkAction::delete(),
            ])
    }

    fn form(&self) -> Form {
        Form::new()
            .section(
                Section::make("General Specifications")
                    .description("Core product identifiers and merchandising details")
                    .columns(2)
                    .schema(vec![
                        FormField::text("title").required().placeholder("Mechanical Keyboard RGB").columnSpan(2),
                        FormField::text("sku").required().placeholder("KB-9900-RGB"),
                        FormField::searchableSelect("category", vec![
                            ("Electronics", "Electronics & Peripherals"),
                            ("Workspace", "Workspace & Ergonomics"),
                            ("Accessories", "Cables & Accessories"),
                        ]),
                    ])
            )
            .section(
                Section::make("Inventory & Financials")
                    .description("Pricing rules and stock replenishment thresholds")
                    .columns(2)
                    .schema(vec![
                        FormField::number("price").required().placeholder("129").defaultValue("99"),
                        FormField::number("stock").required().placeholder("50").defaultValue("100"),
                        FormField::select("status", vec![
                            ("Active", "Active"),
                            ("Low Stock", "Low Stock"),
                            ("Archived", "Archived"),
                        ]),
                    ])
            )
    }

    fn fetch_rows(&self, query: &QueryState) -> (Vec<RowData>, usize) {
        let (records, total) = self.store.list(query);
        let rows = records
            .into_iter()
            .map(|p| {
                RowData::new(&p.id)
                    .insert("id", &p.id)
                    .insert("title", &p.title)
                    .insert("sku", &p.sku)
                    .insert("price", &format!("${}", p.price))
                    .insert("stock", &p.stock.to_string())
                    .insert("category", &p.category)
                    .insert("status", &p.status)
                    .insert("created_at", &p.created_at)
            })
            .collect();
        (rows, total)
    }

    fn get_row(&self, id: &str) -> Option<RowData> {
        self.store.find(id).map(|p| {
            RowData::new(&p.id)
                .insert("id", &p.id)
                .insert("title", &p.title)
                .insert("sku", &p.sku)
                .insert("price", &p.price.to_string())
                .insert("stock", &p.stock.to_string())
                .insert("category", &p.category)
                .insert("status", &p.status)
        })
    }

    fn create_row(&self, values: HashMap<String, String>) -> Result<String, String> {
        let title = values.get("title").cloned().unwrap_or_default();
        let sku = values.get("sku").cloned().unwrap_or_default();
        let price = values.get("price").and_then(|p| p.parse().ok()).unwrap_or(0);
        let stock = values.get("stock").and_then(|s| s.parse().ok()).unwrap_or(0);
        let category = values.get("category").cloned().unwrap_or_else(|| "Electronics".into());
        let status = values.get("status").cloned().unwrap_or_else(|| "Active".into());

        self.store.create(&title, &sku, price, stock, &category, &status)
    }

    fn update_row(&self, id: &str, values: HashMap<String, String>) -> Result<(), String> {
        let title = values.get("title").cloned().unwrap_or_default();
        let sku = values.get("sku").cloned().unwrap_or_default();
        let price = values.get("price").and_then(|p| p.parse().ok()).unwrap_or(0);
        let stock = values.get("stock").and_then(|s| s.parse().ok()).unwrap_or(0);
        let category = values.get("category").cloned().unwrap_or_else(|| "Electronics".into());
        let status = values.get("status").cloned().unwrap_or_else(|| "Active".into());

        self.store.update(id, &title, &sku, price, stock, &category, &status)
    }

    fn delete_row(&self, id: &str) -> Result<(), String> {
        self.store.delete(id)
    }

    #[allow(non_snake_case)]
    fn canDeleteRow(&self, user: &User, row: &RowData) -> bool {
        if !self.canDelete(user) {
            return false;
        }
        // Protect archived catalog items from hard deletion
        row.get("status") != Some("Archived")
    }
}

// =========================================================================
// Customer Resource (Modal Form Presentation)
// =========================================================================

pub struct CustomerStore {
    conn: Arc<Mutex<rusqlite::Connection>>,
}

impl CustomerStore {
    pub fn new(conn: Arc<Mutex<rusqlite::Connection>>) -> Self {
        Self { conn }
    }

    pub fn list(&self) -> Vec<CustomerRecord> {
        let lock = self.conn.lock().unwrap();
        let mut stmt = lock
            .prepare("SELECT id, name, email, plan, total_spent, status, created_at FROM customers ORDER BY total_spent DESC")
            .unwrap();
        stmt.query_map([], |row| {
            Ok(CustomerRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                email: row.get(2)?,
                plan: row.get(3)?,
                total_spent: row.get(4)?,
                status: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }
}

pub struct CustomerResource {
    store: Arc<CustomerStore>,
}

impl CustomerResource {
    pub fn new(store: Arc<CustomerStore>) -> Self {
        Self { store }
    }
}

impl Resource for CustomerResource {
    fn name(&self) -> &str {
        "Customer"
    }

    fn plural_name(&self) -> &str {
        "Customers"
    }

    fn slug(&self) -> &str {
        "customers"
    }

    fn form_mode(&self) -> FormMode {
        FormMode::Modal
    }

    fn table(&self) -> Table {
        Table::new()
            .column(Column::text("id").label("Customer ID"))
            .column(Column::text("name").label("Name").searchable().sortable())
            .column(Column::text("email").label("Email").searchable())
            .column(Column::badge("plan", vec![
                ("Enterprise", "amber"),
                ("Pro", "indigo"),
                ("Free", "slate"),
            ]))
            .column(Column::numeric("total_spent").label("Total Spent ($)").sortable())
            .column(Column::badge("status", vec![
                ("Active", "emerald"),
                ("Delinquent", "rose"),
            ]))
    }

    fn form(&self) -> Form {
        Form::new()
            .field(FormField::text("name").required().placeholder("Full Name"))
            .field(FormField::email("email").required().placeholder("client@domain.com"))
            .field(FormField::select("plan", vec![
                ("Free", "Free Starter"),
                ("Pro", "Pro ($49/mo)"),
                ("Enterprise", "Enterprise ($499/mo)"),
            ]))
            .field(FormField::select("status", vec![
                ("Active", "Active"),
                ("Delinquent", "Payment Delinquent"),
            ]))
    }

    fn fetch_rows(&self, _query: &QueryState) -> (Vec<RowData>, usize) {
        let customers = self.store.list();
        let total = customers.len();
        let rows = customers
            .into_iter()
            .map(|c| {
                RowData::new(&c.id)
                    .insert("id", &c.id)
                    .insert("name", &c.name)
                    .insert("email", &c.email)
                    .insert("plan", &c.plan)
                    .insert("total_spent", &format!("${}", c.total_spent))
                    .insert("status", &c.status)
                    .insert("created_at", &c.created_at)
            })
            .collect();
        (rows, total)
    }

    fn delete_row(&self, _id: &str) -> Result<(), String> {
        Ok(())
    }
}

// =========================================================================
// Enterprise Modular DDD Layer: CatalogModule with Headless API
// =========================================================================

#[derive(serde::Deserialize)]
pub struct CreateProductPayload {
    pub title: String,
    pub price: i64,
}

impl ValidatedRequest for CreateProductPayload {
    fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if self.title.trim().is_empty() {
            errors.push("title is required".into());
        }
        if self.price <= 0 {
            errors.push("price must be greater than zero".into());
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

pub struct CatalogModule {
    store: Arc<ProductStore>,
}

impl CatalogModule {
    pub fn new(store: Arc<ProductStore>) -> Self {
        Self { store }
    }
}

impl AppModule for CatalogModule {
    fn name(&self) -> &str {
        "Catalog"
    }

    fn slug(&self) -> &str {
        "catalog"
    }

    fn description(&self) -> Option<&str> {
        Some("Product catalog management and headless inventory API")
    }

    fn resource(&self) -> Option<Arc<dyn Resource>> {
        Some(Arc::new(ProductResource::new(self.store.clone())))
    }

    fn routes(&self) -> Router {
        let store = self.store.clone();
        Router::new()
            .route(
                "/stats",
                get({
                    let store = store.clone();
                    move || {
                        let (products, total) = store.list(&QueryState::default());
                        let total_inventory: i64 = products.iter().map(|p| p.stock).sum();
                        let valuation: i64 = products.iter().map(|p| p.price * p.stock).sum();
                        async move {
                            Json(ApiResponse::ok(serde_json::json!({
                                "totalProducts": total,
                                "totalInventoryUnits": total_inventory,
                                "inventoryValuationUsd": valuation,
                            })))
                        }
                    }
                }),
            )
            .route(
                "/create",
                post({
                    let store = store.clone();
                    move |Json(payload): Json<CreateProductPayload>| {
                        let store = store.clone();
                        async move {
                            if let Err(errors) = payload.validate() {
                                return (
                                    StatusCode::UNPROCESSABLE_ENTITY,
                                    Json(ApiResponse::validation_errors(errors)),
                                )
                                    .into_response();
                            }
                            match store.create(&payload.title, "AUTOGEN-SKU", payload.price, 10, "General", "Active") {
                                Ok(id) => (
                                    StatusCode::CREATED,
                                    Json(ApiResponse::with_message(id, "Product created successfully")),
                                )
                                    .into_response(),
                                Err(err) => (
                                    StatusCode::BAD_REQUEST,
                                    Json(ApiResponse::error(err)),
                                )
                                    .into_response(),
                            }
                        }
                    }
                }),
            )
    }
}

// =========================================================================
// Utilities & Database Migrations
// =========================================================================

fn uuid_fast() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos();
    nanos % 90000 + 10000
}

fn initialize_saas_database(conn: &rusqlite::Connection) {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS products (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            sku TEXT NOT NULL UNIQUE,
            price INTEGER NOT NULL,
            stock INTEGER NOT NULL,
            category TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS customers (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT NOT NULL UNIQUE,
            plan TEXT NOT NULL,
            total_spent INTEGER NOT NULL,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        INSERT OR IGNORE INTO products (id, title, sku, price, stock, category, status, created_at) VALUES
            ('prod_101', 'CloudFlare Edge Gateway', 'CF-GW-01', 199, 45, 'Electronics', 'Active', '2026-10-01'),
            ('prod_102', 'Ergonomic Standing Desk Pro', 'DSK-990', 599, 12, 'Workspace', 'Active', '2026-10-02'),
            ('prod_103', 'Mechanical Keyboard 75%', 'KB-75-MX', 149, 3, 'Electronics', 'Low Stock', '2026-10-03'),
            ('prod_104', 'USB-C Thunderbolt 4 Cable', 'TB4-2M', 29, 230, 'Accessories', 'Active', '2026-10-04'),
            ('prod_105', 'Legacy Wireless Presenter', 'WP-OLD-9', 39, 0, 'Accessories', 'Archived', '2026-09-15');

        INSERT OR IGNORE INTO customers (id, name, email, plan, total_spent, status, created_at) VALUES
            ('cust_01', 'Acme Corporation', 'billing@acme.com', 'Enterprise', 12450, 'Active', '2026-01-15'),
            ('cust_02', 'Linear Systems Inc', 'ops@linearsys.io', 'Pro', 3490, 'Active', '2026-03-20'),
            ('cust_03', 'Stripe Developer Labs', 'devs@stripe.internal', 'Enterprise', 8900, 'Active', '2026-05-10'),
            ('cust_04', 'Independent Cloud Studio', 'founder@cloudstudio.me', 'Free', 0, 'Active', '2026-09-01');
        "#,
    )
    .expect("Failed to initialize database tables and seed rows");
}

// =========================================================================
// Main Application Runner & Self-Test Verification
// =========================================================================

#[tokio::main]
async fn main() {
    println!("\n========================================================");
    println!("  Terminus Reference Application: SaaS Store & CRM");
    println!("========================================================");

    // 1. Initialize SQLite Database
    let conn = rusqlite::Connection::open_in_memory().expect("Failed to open SQLite database");
    initialize_saas_database(&conn);
    let shared_conn = Arc::new(Mutex::new(conn));

    // 2. Set up Stores
    let product_store = Arc::new(ProductStore::new(shared_conn.clone()));
    let customer_store = Arc::new(CustomerStore::new(shared_conn.clone()));
    let user_repo = Arc::new(SqliteUserRepository::new(shared_conn.clone()).unwrap());
    let audit_repo = Arc::new(SqliteAuditRepository::new(shared_conn).unwrap());

    // 3. Set up In-Memory Cache and Cache Warming
    let cache = MemoryCache::new();
    let cached_valuation = cache.remember("stats:inventory_valuation", 600, || {
        let (products, _) = product_store.list(&QueryState::default());
        let val: i64 = products.iter().map(|p| p.price * p.stock).sum();
        val.to_string()
    });
    println!("  • Cache Warmup: Inventory Valuation = ${}", cached_valuation);

    // 4. Set up Job Queue & Dispatch Background Task
    let queue = Arc::new(JobQueue::new());
    queue.dispatch_fn("sync_catalog_search_index", || {
        println!("  [Queue Worker] Synchronized 5 catalog items to search index.");
        Ok(())
    });
    let processed = queue.work_all();
    println!("  • Background Queue: Processed {} jobs.", processed);

    // 5. Transactional Notification Mailer
    let mailer = Arc::new(Mailer::new());
    MailMessage::to("billing@acme.com")
        .subject("Your Enterprise Subscription Invoice")
        .line("Thank you for using Terminus SaaS.")
        .action("View Account", "/admin/customers")
        .send_via(&mailer)
        .expect("Mail delivery failed");
    println!("  • Transactional Mailer: Dispatched notification email (Outbox size: {}).", mailer.sent_messages().len());

    // 6. Personal Access Token Issuance
    let token_store = Arc::new(MemoryTokenStore::new());
    let pat = token_store
        .create_token(
            "User",
            "user_superadmin",
            "saas-integration-key",
            vec!["products:read".into(), "products:write".into()],
            None,
        )
        .expect("Failed to issue API token");
    println!("  • Personal Access Token: Generated Bearer Token: {}", pat.plain_text_token);

    // 7. Fluent Assertion Self-Test
    terminus_test("verifies catalog inventory calculations and record policies", || {
        let (products, total) = product_store.list(&QueryState::default());
        expect(&total).to_be(&5);
        expect(&products[0].status).to_be(&"Active".to_string());

        let customers = customer_store.list();
        expect(&customers.len()).to_be(&4);
        expect(&customers[0].plan).to_be(&"Enterprise".to_string());
    });
    println!("  • Self-Test: Fluent assertions verified successfully.");

    // 8. Assemble AdminPanel & Mount Modules
    let admin = AdminPanel::new()
        .brand_name("Acme SaaS Control")
        .primary_color(PrimaryColor::Indigo)
        .font_family(FontFamily::Inter)
        .users(user_repo)
        .audit(audit_repo)
        .queue(queue)
        .mailer(mailer)
        .tokens(token_store)
        .register(CustomerResource::new(customer_store))
        .module(CatalogModule::new(product_store));

    let app = Router::new()
        .nest("/admin", admin.into_router())
        .route("/", axum::routing::get(|| async {
            axum::response::Redirect::temporary("/admin")
        }));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    println!("\n  Admin Panel URL: http://127.0.0.1:3001/admin");
    println!("  Headless API:    http://127.0.0.1:3001/admin/api/catalog/stats");
    println!("========================================================\n");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
