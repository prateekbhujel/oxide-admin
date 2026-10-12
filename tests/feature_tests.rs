use terminus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

struct TestModalResource;
impl Resource for TestModalResource {
    fn name(&self) -> &str { "Item" }
    fn plural_name(&self) -> &str { "Items" }
    fn slug(&self) -> &str { "items" }
    fn table(&self) -> Table { Table::new() }
    fn form(&self) -> Form { Form::new() }
    fn fetch_rows(&self, _query: &QueryState) -> (Vec<RowData>, usize) { (vec![], 0) }
}

struct TestPageResource;
impl Resource for TestPageResource {
    fn name(&self) -> &str { "Article" }
    fn plural_name(&self) -> &str { "Articles" }
    fn slug(&self) -> &str { "articles" }
    fn form_mode(&self) -> FormMode { FormMode::Page }
    fn table(&self) -> Table {
        Table::new()
            .striped()
            .filter(TableFilter::select("status", "Status", vec![("draft", "Draft"), ("published", "Published")]))
    }
    fn form(&self) -> Form {
        Form::new()
            .field(FormField::searchable_select("category", vec![("tech", "Technology"), ("life", "Lifestyle")]))
    }
    fn fetch_rows(&self, _query: &QueryState) -> (Vec<RowData>, usize) { (vec![], 0) }
}

#[test]
fn test_form_mode_defaults_and_customization() {
    let modal_res = TestModalResource;
    assert_eq!(modal_res.form_mode(), FormMode::Modal);

    let page_res = TestPageResource;
    assert_eq!(page_res.form_mode(), FormMode::Page);
}

#[test]
fn test_table_styles_and_filters() {
    let table = Table::new()
        .striped()
        .compact()
        .filter(TableFilter::select("role", "Role", vec![("admin", "Admin"), ("user", "User")]));

    assert_eq!(table.style, TableStyle::Compact);
    assert_eq!(table.filters.len(), 1);
    assert_eq!(table.filters[0].name, "role");
    assert_eq!(table.filters[0].label, "Role");
    assert_eq!(table.filters[0].options.len(), 2);
}

#[test]
fn test_searchable_select_form_field() {
    let field = FormField::searchable_select(
        "assignee",
        vec![("1", "Pratik"), ("2", "Dharma Raj"), ("3", "Elena Rostova")],
    );

    assert_eq!(field.name, "assignee");
    assert_eq!(field.label, "Assignee");
    match field.field_type {
        terminus::form::FieldType::SearchableSelect { options } => {
            assert_eq!(options.len(), 3);
            assert_eq!(options[0], ("1".to_string(), "Pratik".to_string()));
        }
        _ => panic!("Expected SearchableSelect field type"),
    }
}

#[test]
fn test_query_state_with_filters() {
    let mut filters = HashMap::new();
    filters.insert("role".to_string(), "Superadmin".to_string());
    filters.insert("status".to_string(), "Active".to_string());

    let query = QueryState {
        page: 1,
        per_page: 8,
        search: "pratik".to_string(),
        sort_by: Some("name".to_string()),
        sort_desc: false,
        filters,
    };

    assert_eq!(query.filters.get("role").unwrap(), "Superadmin");
    assert_eq!(query.filters.get("status").unwrap(), "Active");
}

#[test]
fn test_camel_case_and_headline_utilities() {
    assert_eq!(to_camel_case("created_at"), "createdAt");
    assert_eq!(to_camel_case("order_number"), "orderNumber");
    assert_eq!(to_snake_case("createdAt"), "created_at");
    assert_eq!(to_snake_case("orderNumber"), "order_number");

    assert_eq!(headline("createdAt"), "Created At");
    assert_eq!(headline("created_at"), "Created At");
    assert_eq!(headline("userName"), "User Name");
    assert_eq!(headline("user_name"), "User Name");
    assert_eq!(headline("id"), "ID");
}

#[test]
fn test_row_data_bidirectional_case_fallback() {
    let row1 = RowData::new("1").insert("createdAt", "2026-10-08");
    assert_eq!(row1.get("createdAt"), Some("2026-10-08"));
    assert_eq!(row1.get("created_at"), Some("2026-10-08"));

    let row2 = RowData::new("2").insert("user_name", "Pratik");
    assert_eq!(row2.get("userName"), Some("Pratik"));
    assert_eq!(row2.get("user_name"), Some("Pratik"));
}

#[test]
fn test_camel_case_fluent_builder_methods() {
    let page_res = TestPageResource;
    assert_eq!(page_res.pluralName(), "Articles");
    assert_eq!(page_res.formMode(), FormMode::Page);

    let table = Table::new().pageSize(25).defaultSortBy("createdAt");
    assert_eq!(table.default_page_size, 25);
    assert_eq!(table.default_sort_by, Some("createdAt".to_string()));

    let field = FormField::searchableSelect("role", vec![("admin", "Admin")]);
    assert_eq!(field.name, "role");
}

#[test]
fn test_domain_json_camel_case_serialization() {
    let user = User {
        id: "1".into(),
        name: "Pratik".into(),
        email: "pratik@example.com".into(),
        role: Role::Superadmin,
        status: UserStatus::Active,
        created_at: "2026-10-08".into(),
        ..Default::default()
    };

    let user_json = serde_json::to_string(&user).unwrap();
    assert!(user_json.contains("\"createdAt\":\"2026-10-08\""));
    assert!(!user_json.contains("\"created_at\""));

    let log = AuditLog {
        id: "1".into(),
        user_name: "Pratik".into(),
        action: "CREATE".into(),
        resource: "users".into(),
        record_id: "2".into(),
        details: "Created record".into(),
        timestamp: "2026-10-08 12:00:00".into(),
    };

    let log_json = serde_json::to_string(&log).unwrap();
    assert!(log_json.contains("\"userName\":\"Pratik\""));
    assert!(log_json.contains("\"recordId\":\"2\""));
}

struct PolicyTestResource;
impl Resource for PolicyTestResource {
    fn name(&self) -> &str { "Invoice" }
    fn plural_name(&self) -> &str { "Invoices" }
    fn slug(&self) -> &str { "invoices" }
    fn table(&self) -> Table {
        Table::new().column(Column::text("status"))
    }
    fn form(&self) -> Form { Form::new() }
    fn fetch_rows(&self, _query: &QueryState) -> (Vec<RowData>, usize) {
        let r1 = RowData::new("1").insert("status", "Draft");
        let r2 = RowData::new("2").insert("status", "Settled");
        (vec![r1, r2], 2)
    }

    #[allow(non_snake_case)]
    fn canEditRow(&self, user: &User, row: &RowData) -> bool {
        if !self.canEdit(user) {
            return false;
        }
        row.get("status") != Some("Settled")
    }

    #[allow(non_snake_case)]
    fn canDeleteRow(&self, user: &User, row: &RowData) -> bool {
        if !self.canDelete(user) {
            return false;
        }
        row.get("status") != Some("Settled")
    }
}

#[test]
fn test_record_level_authorization_hooks() {
    let res = PolicyTestResource;
    let admin_user = User {
        id: "1".into(),
        name: "Admin".into(),
        email: "admin@example.com".into(),
        role: Role::Superadmin,
        status: UserStatus::Active,
        created_at: "2026-10-08".into(),
        ..Default::default()
    };
    let member_user = User {
        id: "2".into(),
        name: "Member".into(),
        email: "member@example.com".into(),
        role: Role::Member,
        status: UserStatus::Active,
        created_at: "2026-10-08".into(),
        ..Default::default()
    };

    let draft_row = RowData::new("1").insert("status", "Draft");
    let settled_row = RowData::new("2").insert("status", "Settled");

    // Member cannot edit or delete invoices at all
    assert!(!res.canEdit(&member_user));
    assert!(!res.canEditRow(&member_user, &draft_row));
    assert!(!res.canDeleteRow(&member_user, &draft_row));

    // Admin has invoices.edit & invoices.delete, but Settled records are locked
    assert!(res.canEdit(&admin_user));
    assert!(res.canDelete(&admin_user));

    // Draft row is editable and deletable
    assert!(res.canEditRow(&admin_user, &draft_row));
    assert!(res.canDeleteRow(&admin_user, &draft_row));

    // Settled row is locked by record-level policy
    assert!(!res.canEditRow(&admin_user, &settled_row));
    assert!(!res.canDeleteRow(&admin_user, &settled_row));

    // Both camelCase and snake_case aliases work interchangeably
    assert!(!res.can_edit_row(&admin_user, &settled_row));
    assert!(!res.can_delete_row(&admin_user, &settled_row));
}

#[test]
fn test_policy_table_view_action_buttons() {
    let res = PolicyTestResource;
    let admin_user = User {
        id: "1".into(),
        name: "Admin".into(),
        email: "admin@example.com".into(),
        role: Role::Superadmin,
        status: UserStatus::Active,
        created_at: "2026-10-08".into(),
        ..Default::default()
    };

    let query = QueryState::default();
    let html = terminus::view::table_view::render_table_partial(&res, &query, &admin_user);

    // Row 1 (Draft) should have edit and delete dialog triggers
    assert!(html.contains("openEditDialog('invoices', '1'"));
    assert!(html.contains("openDeleteDialog('invoices', '1'"));

    // Row 2 (Settled) should NOT have edit or delete dialog triggers
    assert!(!html.contains("openEditDialog('invoices', '2'"));
    assert!(!html.contains("openDeleteDialog('invoices', '2'"));
    assert!(html.contains("—")); // Fallback dash for disabled row actions
}

#[test]
fn test_theme_and_brand_customization() {
    let theme = ThemeConfig::new()
        .brand_name("Acme Studio")
        .primary_color(PrimaryColor::Violet)
        .font_family(FontFamily::Inter);

    assert_eq!(theme.brand_name, "Acme Studio");
    assert_eq!(theme.primary_color.name(), "violet");
    assert_eq!(theme.primary_color.hex(), "#8b5cf6");
    assert_eq!(theme.font_family.font_family_css(), "'Inter', system-ui, -apple-system, sans-serif");
    assert!(theme.font_family.css_url().contains("family=Inter"));

    let user = User {
        id: "1".into(),
        name: "Admin".into(),
        email: "admin@acme.com".into(),
        role: Role::Superadmin,
        status: UserStatus::Active,
        created_at: "2026-10-08".into(),
        ..Default::default()
    };

    let html = terminus::view::layout::render_page(
        "Products",
        "products",
        &[],
        "<div>Content</div>",
        "",
        None,
        &user,
        &theme,
    );

    assert!(html.contains("<title>Products · Acme Studio</title>"));
    assert!(html.contains("Acme Studio"));
    assert!(html.contains("#8b5cf6"));
    assert!(html.contains("family=Inter"));
}

#[test]
fn test_job_queue_and_background_worker() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    let queue = JobQueue::new();
    let ran_flag = Arc::new(AtomicBool::new(false));
    let ran_flag_clone = ran_flag.clone();

    queue.dispatch_fn("send_slack_alert", move || {
        ran_flag_clone.store(true, Ordering::SeqCst);
        Ok(())
    });

    queue.dispatch_fn("generate_invoice_pdf", || {
        Ok(())
    });

    assert_eq!(queue.stats().pending, 2);
    assert_eq!(queue.stats().processed, 0);

    let count = queue.work_all();
    assert_eq!(count, 2);
    assert_eq!(queue.stats().pending, 0);
    assert_eq!(queue.stats().processed, 2);
    assert_eq!(queue.stats().failed, 0);
    assert!(ran_flag.load(Ordering::SeqCst));
}

#[test]
fn test_transactional_mailer() {
    let mailer = Mailer::new();
    let msg = MailMessage::to("founder@startup.io")
        .subject("Welcome to Terminus")
        .line("Your enterprise admin kit is ready.")
        .action("Access Panel", "/admin");

    assert_eq!(msg.to, "founder@startup.io");
    assert_eq!(msg.subject, "Welcome to Terminus");
    assert_eq!(msg.lines.len(), 1);

    msg.send_via(&mailer).unwrap();

    let outbox = mailer.sent_messages();
    assert_eq!(outbox.len(), 1);
    assert_eq!(outbox[0].to, "founder@startup.io");
    assert_eq!(outbox[0].subject, "Welcome to Terminus");
}

#[test]
fn test_fluent_query_builder_sql_and_bindings() {
    let query = Query::table("orders")
        .select(vec!["id", "customer", "amount", "status"])
        .where_eq("status", "Paid")
        .where_gt("amount", "100")
        .order_by("created_at", "DESC")
        .paginate(1, 10);

    let (sql, bindings) = query.to_sql();
    assert_eq!(
        sql,
        "SELECT id, customer, amount, status FROM orders WHERE status = ? AND amount > ? ORDER BY created_at DESC LIMIT 10 OFFSET 0"
    );
    assert_eq!(bindings, vec!["Paid", "100"]);

    // Test camelCase aliases
    let query_camel = Query::table("users")
        .whereEq("role", "admin")
        .whereLike("email", "%@company.com")
        .orderBy("id", "ASC")
        .limit(5);

    let (sql_camel, bindings_camel) = query_camel.toSql();
    assert_eq!(
        sql_camel,
        "SELECT * FROM users WHERE role = ? AND email LIKE ? ORDER BY id ASC LIMIT 5"
    );
    assert_eq!(bindings_camel, vec!["admin", "%@company.com"]);
}

#[test]
fn test_cache_remember_and_ttl_expiration() {
    let cache = MemoryCache::new();

    // Cache miss followed by remember
    let val1 = cache.remember("stats:daily_sales", 300, || {
        "42000".to_string()
    });
    assert_eq!(val1, "42000");

    // Cache hit: callback not invoked
    let val2 = cache.remember("stats:daily_sales", 300, || {
        "99999".to_string()
    });
    assert_eq!(val2, "42000");

    // Direct get
    assert_eq!(cache.get("stats:daily_sales"), Some("42000".to_string()));

    // Forget
    assert!(cache.forget("stats:daily_sales"));
    assert_eq!(cache.get("stats:daily_sales"), None);

    // Flush
    cache.put("key1", "val1", None);
    cache.put("key2", "val2", None);
    cache.flush();
    assert_eq!(cache.get("key1"), None);
    assert_eq!(cache.get("key2"), None);
}

#[test]
fn test_fluent_expectations() {
    use terminus::prelude::*;

    terminus_test("it validates user properties with fluent syntax", || {
        let user = User {
            id: "u_42".to_string(),
            name: "Engineer".to_string(),
            email: "dev@terminus.dev".to_string(),
            role: Role::Superadmin,
            status: UserStatus::Active,
            created_at: "2026-10-08".to_string(),
            ..Default::default()
        };

        // Chainable assertions
        expect(&user.name).to_be("Engineer")
            .and(&user.email).to_contain("@terminus.dev")
            .and(&user.role).to_be(&Role::Superadmin)
            .and(user.status.is_active()).to_be_true();

        // Comparison and collections
        expect(100).to_be_greater_than(50);
        expect(25).to_be_less_than(50);
        expect("").to_be_empty();
        expect("non-empty").to_not_be_empty();

        // Options and Results
        let some_val: Option<i32> = Some(42);
        let none_val: Option<i32> = None;
        expect(some_val).to_be_some();
        expect(none_val).to_be_none();

        let ok_val: Result<&str, &str> = Ok("success");
        let err_val: Result<&str, &str> = Err("failure");
        expect(ok_val).to_be_ok();
        expect(err_val).to_be_err();
    });
}

#[test]
fn test_debug_dump_macro() {
    use terminus::dump;
    let data = vec!["terminus", "admin", "fast"];
    // Verify dump! macro executes smoothly without errors
    dump!(data);
}

#[tokio::test]
async fn test_websocket_broadcaster_pubsub() {
    use terminus::prelude::*;

    let broadcaster = Broadcaster::new(16);
    assert_eq!(broadcaster.subscriber_count(), 0);

    // Subscribe client 1 & 2
    let mut rx1 = broadcaster.subscribe();
    let mut rx2 = broadcaster.subscribe();
    assert_eq!(broadcaster.subscriber_count(), 2);

    // Broadcast event
    let payload = serde_json::json!({
        "order_id": "ORD-999",
        "amount": 450,
        "status": "Paid"
    });
    let receiver_count = broadcaster.broadcast("orders", "order:created", payload.clone());
    assert_eq!(receiver_count, 2);

    // Both clients receive identical event
    let msg1 = rx1.recv().await.unwrap();
    let msg2 = rx2.recv().await.unwrap();

    expect(&msg1.channel).to_be("orders");
    expect(&msg1.event).to_be("order:created");
    expect(msg1.data["order_id"].as_str().unwrap()).to_be("ORD-999");

    expect(&msg2.channel).to_be("orders");
    expect(&msg2.event).to_be("order:created");
    expect(msg2.data["amount"].as_i64().unwrap()).to_be(450);
}

struct ActionTestResource {
    records: std::sync::Mutex<HashMap<String, HashMap<String, String>>>,
}

impl ActionTestResource {
    fn new() -> Self {
        let mut map = HashMap::new();
        let mut r1 = HashMap::new();
        r1.insert("name".to_string(), "MacBook Pro".to_string());
        r1.insert("price".to_string(), "1999".to_string());
        let mut r2 = HashMap::new();
        r2.insert("name".to_string(), "Studio Display".to_string());
        r2.insert("price".to_string(), "1599".to_string());
        map.insert("1".to_string(), r1);
        map.insert("2".to_string(), r2);
        Self {
            records: std::sync::Mutex::new(map),
        }
    }
}

impl Resource for ActionTestResource {
    fn name(&self) -> &str { "Product" }
    fn plural_name(&self) -> &str { "Products" }
    fn slug(&self) -> &str { "products" }

    fn table(&self) -> Table {
        Table::new()
            .columns(vec![
                Column::text("name").label("Product Name").sortable(),
                Column::numeric("price").label("Price").sortable(),
            ])
            .actions(vec![
                Action::view(),
                Action::edit(),
                Action::replicate(),
                Action::delete(),
                Action::make("publish")
                    .label("Publish Now")
                    .icon("check")
                    .color("emerald")
                    .requiresConfirmation()
                    .modalHeading("Publish Product")
                    .modalDescription("Are you sure you want to make this product live?"),
            ])
            .bulkActions(vec![
                BulkAction::export(),
                BulkAction::delete(),
                BulkAction::make("archive")
                    .label("Archive Selected")
                    .color("amber")
                    .requiresConfirmation()
                    .modalHeading("Archive Records")
                    .modalDescription("Archive all selected items?"),
            ])
            .emptyStateHeading("No products in inventory")
            .emptyStateDescription("Add your first product to get started.")
    }

    fn fetch_rows(&self, _query: &QueryState) -> (Vec<RowData>, usize) {
        let guard = self.records.lock().unwrap();
        let mut rows = Vec::new();
        let mut ids: Vec<_> = guard.keys().cloned().collect();
        ids.sort();
        for id in ids {
            if let Some(map) = guard.get(&id) {
                let mut row = RowData::new(&id);
                for (k, v) in map {
                    row.values.insert(k.clone(), v.clone());
                }
                rows.push(row);
            }
        }
        let len = rows.len();
        (rows, len)
    }

    fn get_row(&self, id: &str) -> Option<RowData> {
        let guard = self.records.lock().unwrap();
        guard.get(id).map(|map| {
            let mut row = RowData::new(id);
            for (k, v) in map {
                row.values.insert(k.clone(), v.clone());
            }
            row
        })
    }

    fn create_row(&self, values: HashMap<String, String>) -> Result<String, String> {
        let mut guard = self.records.lock().unwrap();
        let next_id = (guard.len() + 1).to_string();
        guard.insert(next_id.clone(), values);
        Ok(next_id)
    }

    fn delete_row(&self, id: &str) -> Result<(), String> {
        let mut guard = self.records.lock().unwrap();
        guard.remove(id);
        Ok(())
    }
}

#[test]
fn test_actions_and_replicate_row() {
    let res = ActionTestResource::new();

    // Verify Action builder with strict camelCase and fluent methods
    let custom_action = Action::make("duplicate")
        .label("Clone Product")
        .icon("copy")
        .color("indigo")
        .requiresConfirmation()
        .modalHeading("Clone this item")
        .modalDescription("Are you sure you want to clone this item?")
        .actionUrl("/custom/url");

    expect(&custom_action.id).to_be("duplicate");
    expect(&custom_action.label).to_be("Clone Product");
    expect(&custom_action.color).to_be("indigo");
    assert!(custom_action.requires_confirmation);
    expect(custom_action.modal_heading.as_deref().unwrap()).to_be("Clone this item");
    expect(custom_action.action_url.as_deref().unwrap()).to_be("/custom/url");

    // Test record replication hook
    let new_id = res.replicateRow("1").expect("Replication should succeed");
    let cloned_record = res.get_row(&new_id).expect("Cloned record should exist");
    expect(cloned_record.get("name").unwrap()).to_be("MacBook Pro (Copy)");
    expect(cloned_record.get("price").unwrap()).to_be("1999");

    // Test row action dispatcher
    let action_msg = res.handleAction("replicate", "2").expect("Dispatcher replicate should succeed");
    assert!(action_msg.contains("replicated successfully"));

    let (_, total) = res.fetch_rows(&QueryState::default());
    assert_eq!(total, 4); // 2 original + 2 replicated
}

#[test]
fn test_bulk_delete_and_streaming_export() {
    let res = ActionTestResource::new();

    // Verify BulkAction builder with strict camelCase
    let bulk_act = BulkAction::make("archive")
        .label("Archive Selected")
        .color("amber")
        .requiresConfirmation()
        .modalHeading("Confirm Archive")
        .modalDescription("Selected records will be archived.");

    expect(&bulk_act.id).to_be("archive");
    expect(&bulk_act.label).to_be("Archive Selected");
    expect(&bulk_act.color).to_be("amber");
    assert!(bulk_act.requires_confirmation);
    expect(bulk_act.modal_heading.as_deref().unwrap()).to_be("Confirm Archive");

    // High-performance streaming CSV export
    let csv_output = res.bulkExport(&["1".to_string(), "2".to_string()], "csv").expect("CSV export should succeed");
    assert!(csv_output.contains("name,price") || csv_output.contains("price,name"));
    assert!(csv_output.contains("MacBook Pro"));
    assert!(csv_output.contains("Studio Display"));

    // High-performance streaming JSON export
    let json_output = res.bulkExport(&["1".to_string(), "2".to_string()], "json").expect("JSON export should succeed");
    assert!(json_output.contains("MacBook Pro"));
    assert!(json_output.contains("Studio Display"));
    assert!(json_output.starts_with('[') && json_output.ends_with(']'));

    // High-concurrency in-process batch delete
    let deleted_count = res.bulkDelete(&["1".to_string(), "2".to_string()]).expect("Bulk delete should succeed");
    assert_eq!(deleted_count, 2);

    let (_, remaining) = res.fetch_rows(&QueryState::default());
    assert_eq!(remaining, 0);
}

#[test]
fn test_form_section_and_field_camel_case_builder() {
    // Section layout with columns and schema
    let section = Section::make("General Information")
        .description("Essential product attributes")
        .columns(2)
        .schema(vec![
            FormField::text("title")
                .helperText("Enter full product title")
                .prefix("https://")
                .suffix(".io")
                .columnSpan(2),
            FormField::number("inventory")
                .defaultValue("50")
                .min("0")
                .max("1000")
                .step("1"),
            FormField::password("secret_key")
                .helperText("API secret token")
                .disabled(),
            FormField::textarea("description")
                .rows(5)
                .placeholder("Markdown supported")
                .columnSpan(2),
            FormField::toggle("is_published")
                .helperText("Toggle live visibility on storefront"),
            FormField::datetime("scheduled_release")
                .readOnly(),
        ]);

    expect(section.title()).to_be("General Information");
    expect(section.description.as_deref().unwrap()).to_be("Essential product attributes");
    assert_eq!(section.columns, 2);
    assert_eq!(section.fields.len(), 6);

    // Verify field builders
    let title_field = &section.fields[0];
    expect(title_field.helper_text.as_deref().unwrap()).to_be("Enter full product title");
    expect(title_field.prefix.as_deref().unwrap()).to_be("https://");
    expect(title_field.suffix.as_deref().unwrap()).to_be(".io");
    assert_eq!(title_field.column_span, 2);

    let num_field = &section.fields[1];
    expect(num_field.default_value.as_deref().unwrap()).to_be("50");
    expect(num_field.min.as_deref().unwrap()).to_be("0");
    expect(num_field.max.as_deref().unwrap()).to_be("1000");

    let pwd_field = &section.fields[2];
    assert!(pwd_field.disabled);

    let desc_field = &section.fields[3];
    assert_eq!(desc_field.rows, Some(5));

    let dt_field = &section.fields[5];
    assert!(dt_field.read_only);
}

#[test]
fn test_table_view_renders_bulk_actions_and_empty_state() {
    let res = ActionTestResource::new();
    let admin_user = User {
        id: "1".into(),
        name: "Admin".into(),
        email: "admin@example.com".into(),
        role: Role::Superadmin,
        status: UserStatus::Active,
        created_at: "2026-10-08".into(),
        ..Default::default()
    };

    let query = QueryState::default();
    let html = terminus::view::table_view::render_table_partial(&res, &query, &admin_user);

    // Checkbox master and row selection
    assert!(html.contains("id=\"select-all-checkbox\""));
    assert!(html.contains("class=\"row-select-checkbox"));

    // Bulk actions bar
    assert!(html.contains("id=\"bulk-actions-toolbar\""));
    assert!(html.contains("executeBulkExport('products', 'csv')"));
    assert!(html.contains("executeBulkExport('products', 'json')"));
    assert!(html.contains("Delete Selected"));

    // Replicate action button
    assert!(html.contains("executeRowAction('products', 'replicate'"));

    // Custom row action
    assert!(html.contains("Publish Now"));

    // Empty state test
    let empty_query = QueryState {
        search: "Nonexistent".into(),
        ..Default::default()
    };
    struct EmptyResource;
    impl Resource for EmptyResource {
        fn name(&self) -> &str { "Order" }
        fn plural_name(&self) -> &str { "Orders" }
        fn slug(&self) -> &str { "orders" }
        fn table(&self) -> Table {
            Table::new()
                .emptyStateHeading("No orders found")
                .emptyStateDescription("No customer orders recorded yet.")
        }
        fn fetch_rows(&self, _query: &QueryState) -> (Vec<RowData>, usize) { (vec![], 0) }
    }
    let empty_html = terminus::view::table_view::render_table_partial(&EmptyResource, &empty_query, &admin_user);
    assert!(empty_html.contains("No orders found"));
    assert!(empty_html.contains("No customer orders recorded yet."));
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct TestProduct {
    id: String,
    title: String,
    price: String,
    category_id: String,
    created_at: String,
    updated_at: String,
}

impl Model for TestProduct {
    fn table_name() -> &'static str {
        "test_products"
    }

    fn get_id(&self) -> Option<&str> {
        Some(&self.id)
    }

    fn set_id(&mut self, id: String) {
        self.id = id;
    }

    fn to_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("id".to_string(), self.id.clone());
        map.insert("title".to_string(), self.title.clone());
        map.insert("price".to_string(), self.price.clone());
        map.insert("category_id".to_string(), self.category_id.clone());
        map.insert("created_at".to_string(), self.created_at.clone());
        map.insert("updated_at".to_string(), self.updated_at.clone());
        map
    }

    fn from_map(map: HashMap<String, String>) -> Self {
        Self {
            id: map.get("id").cloned().unwrap_or_default(),
            title: map.get("title").cloned().unwrap_or_default(),
            price: map.get("price").cloned().unwrap_or_default(),
            category_id: map.get("category_id").cloned().unwrap_or_default(),
            created_at: map.get("created_at").cloned().unwrap_or_default(),
            updated_at: map.get("updated_at").cloned().unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct TestCategory {
    id: String,
    name: String,
    created_at: String,
    updated_at: String,
}

impl Model for TestCategory {
    fn table_name() -> &'static str {
        "test_categories"
    }

    fn get_id(&self) -> Option<&str> {
        Some(&self.id)
    }

    fn set_id(&mut self, id: String) {
        self.id = id;
    }

    fn to_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("id".to_string(), self.id.clone());
        map.insert("name".to_string(), self.name.clone());
        map.insert("created_at".to_string(), self.created_at.clone());
        map.insert("updated_at".to_string(), self.updated_at.clone());
        map
    }

    fn from_map(map: HashMap<String, String>) -> Self {
        Self {
            id: map.get("id").cloned().unwrap_or_default(),
            name: map.get("name").cloned().unwrap_or_default(),
            created_at: map.get("created_at").cloned().unwrap_or_default(),
            updated_at: map.get("updated_at").cloned().unwrap_or_default(),
        }
    }
}

#[test]
fn test_token_issuance_and_abilities() {
    use terminus::prelude::*;

    let store = MemoryTokenStore::new();
    let user = User {
        id: "usr_100".to_string(),
        name: "Developer".to_string(),
        email: "dev@terminus.rs".to_string(),
        role: Role::Admin,
        status: UserStatus::Active,
        created_at: "2026-10-10".to_string(),
        ..Default::default()
    };

    // Issue token with specific abilities
    let new_token = user.create_token(&store, "cli-app", vec!["orders:read", "orders:create"]).unwrap();
    assert!(new_token.plain_text_token.contains('|'));

    // Verify abilities
    assert!(new_token.access_token.can("orders:read"));
    assert!(new_token.access_token.can("orders:create"));
    assert!(new_token.access_token.cant("orders:delete"));
    assert!(!new_token.access_token.can("users:manage"));

    // Find token by plain text secret
    let retrieved = store.find_token(&new_token.plain_text_token);
    assert!(retrieved.is_some());
    let token = retrieved.unwrap();
    assert_eq!(token.name, "cli-app");
    assert_eq!(token.tokenable_id, "usr_100");

    // Wildcard ability token
    let full_access = user.create_token(&store, "root-agent", vec!["*"]).unwrap();
    assert!(full_access.access_token.can("any:ability"));
    assert!(full_access.access_token.can("database:destroy"));

    // Expiration test
    let expired_token = user.create_token_with_expiry(
        &store,
        "temporary",
        vec!["*"],
        Some("2020-01-01T00:00:00Z".to_string()),
    ).unwrap();
    assert!(expired_token.access_token.is_expired());
    assert!(store.find_token(&expired_token.plain_text_token).is_none());
}

#[test]
fn test_sqlite_token_store() {
    use terminus::prelude::*;
    use std::sync::{Arc, Mutex};

    let conn = Arc::new(Mutex::new(rusqlite::Connection::open_in_memory().unwrap()));
    let store = SqliteTokenStore::new(conn).unwrap();

    let new_token = store.create_token("User", "u_42", "mobile-app", vec!["read".into(), "write".into()], None).unwrap();
    let plain_secret = &new_token.plain_text_token;

    let found = store.find_token(plain_secret);
    assert!(found.is_some());
    let pat = found.unwrap();
    assert_eq!(pat.name, "mobile-app");
    assert_eq!(pat.tokenable_id, "u_42");
    assert!(pat.can("read"));
    assert!(pat.can("write"));
    assert!(pat.cant("delete"));

    // Touch last used
    store.touch_last_used(&pat.id).unwrap();
    let list = store.list_for("User", "u_42");
    assert_eq!(list.len(), 1);
    assert!(list[0].last_used_at.is_some());

    // Revoke
    store.revoke_token(&pat.id).unwrap();
    assert!(store.find_token(plain_secret).is_none());
}

#[test]
fn test_orm_active_record_and_query_builder() {
    use terminus::prelude::*;

    let conn = rusqlite::Connection::open_in_memory().unwrap();

    // 1. Schema migration builder
    Schema::create(&conn, "test_categories", |table| {
        table.id();
        table.string("name");
        table.timestamps();
    }).unwrap();

    Schema::create(&conn, "test_products", |table| {
        table.id();
        table.string("title");
        table.string("price");
        table.string("category_id");
        table.timestamps();
    }).unwrap();

    // 2. Dynamic Model Active Record dirty tracking
    let mut dyn_model = DynamicModel::new("test_products");
    dyn_model.set("title", "Mechanical Keyboard");
    dyn_model.set("price", "150");
    dyn_model.set("category_id", "cat_1");

    assert!(dyn_model.is_dirty());
    assert_eq!(dyn_model.get_dirty().get("title").map(|s| s.as_str()), Some("Mechanical Keyboard"));

    dyn_model.save(&conn).unwrap();
    assert!(!dyn_model.is_dirty());
    assert!(dyn_model.get("id").is_some());

    // 3. Typed Model Active Record
    let mut cat_attrs = HashMap::new();
    cat_attrs.insert("id", "cat_1");
    cat_attrs.insert("name", "Hardware");
    TestCategory::create(cat_attrs, &conn).unwrap();

    let mut p1_attrs = HashMap::new();
    p1_attrs.insert("id", "prod_1");
    p1_attrs.insert("title", "Rust Programming");
    p1_attrs.insert("price", "45");
    p1_attrs.insert("category_id", "cat_1");
    TestProduct::create(p1_attrs, &conn).unwrap();

    let mut p2_attrs = HashMap::new();
    p2_attrs.insert("id", "prod_2");
    p2_attrs.insert("title", "Axum Deep Dive");
    p2_attrs.insert("price", "99");
    p2_attrs.insert("category_id", "cat_1");
    TestProduct::create(p2_attrs, &conn).unwrap();

    // Model::all
    let all_prods = TestProduct::all(&conn).unwrap();
    assert_eq!(all_prods.len(), 3); // 1 dynamic + 2 typed

    // Model::find
    let found = TestProduct::find("prod_1", &conn).unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().title, "Rust Programming");

    // ModelQuery builder
    let expensive = TestProduct::query()
        .where_eq("category_id", "cat_1")
        .where_gt("price", "40")
        .order_by("price", "desc")
        .get(&conn)
        .unwrap();
    assert!(expensive.len() >= 2);

    let count = TestProduct::query().where_eq("category_id", "cat_1").count(&conn).unwrap();
    assert_eq!(count, 3);

    let first = TestProduct::query().where_eq("id", "prod_2").first(&conn).unwrap();
    assert!(first.is_some());
    assert_eq!(first.unwrap().title, "Axum Deep Dive");

    // 4. HasMany and BelongsTo relationships
    let has_many_products = HasMany::<TestProduct>::new("cat_1", "category_id");
    let cat_products = has_many_products.get(&conn).unwrap();
    assert_eq!(cat_products.len(), 3);

    let belongs_to_cat = BelongsTo::<TestCategory>::new("cat_1", "id");
    let parent_cat = belongs_to_cat.get(&conn).unwrap();
    assert!(parent_cat.is_some());
    assert_eq!(parent_cat.unwrap().name, "Hardware");
}

#[test]
fn test_terminus_prompts_terminal_table_and_callouts() {
    use terminus::prelude::*;

    // Spin closure execution
    let computed = prompt::spin("Calculating database metrics", || {
        let mut sum = 0;
        for i in 1..=100 {
            sum += i;
        }
        sum
    });
    assert_eq!(computed, 5050);

    // Prompt table
    let headers = vec!["ID", "NAME", "ROLE"];
    let rows = vec![
        vec!["1".into(), "Alice".into(), "Admin".into()],
        vec!["2".into(), "Bob".into(), "Editor".into()],
    ];
    prompt::table(&headers, &rows);

    // Callout messages
    prompt::info("Terminus engine initialized");
    prompt::warning("Cache nearing capacity");
    prompt::error("Database connection timeout");
    prompt::note("Tip", "Run 'terminus migrate' to apply pending schemas");
}

#[tokio::test]
async fn test_enterprise_app_module_and_api_responses() {
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use axum::routing::get;
    use terminus::prelude::*;

    // 1. ValidatedRequest
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct CreateInvoiceRequest {
        customer: String,
        amount: i64,
    }

    impl ValidatedRequest for CreateInvoiceRequest {
        fn validate(&self) -> Result<(), Vec<String>> {
            let mut errs = Vec::new();
            if self.customer.trim().is_empty() {
                errs.push("Customer cannot be empty".into());
            }
            if self.amount <= 0 {
                errs.push("Amount must be greater than zero".into());
            }
            if errs.is_empty() {
                Ok(())
            } else {
                Err(errs)
            }
        }
    }

    let bad_req = CreateInvoiceRequest {
        customer: "".into(),
        amount: -10,
    };
    assert!(bad_req.validate().is_err());
    assert_eq!(bad_req.validate().unwrap_err().len(), 2);

    let good_req = CreateInvoiceRequest {
        customer: "Acme Corp".into(),
        amount: 500,
    };
    assert!(good_req.validate().is_ok());

    // 2. ApiResponse envelope and HTTP status codes
    let ok_resp = ApiResponse::ok("invoice_created");
    assert_eq!(ok_resp.into_response().status(), StatusCode::OK);

    let err_resp = ApiResponse::error("Payment gateway failure");
    assert_eq!(err_resp.into_response().status(), StatusCode::BAD_REQUEST);

    let val_err_resp = ApiResponse::validation_errors(vec!["Invalid currency".into()]);
    assert_eq!(
        val_err_resp.into_response().status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );

    // 3. AppModule and AdminPanel integration
    struct InvoiceResource;
    impl Resource for InvoiceResource {
        fn name(&self) -> &str {
            "Invoice"
        }
        fn plural_name(&self) -> &str {
            "Invoices"
        }
        fn slug(&self) -> &str {
            "invoices"
        }
        fn table(&self) -> Table {
            Table::new()
        }
        fn form(&self) -> Form {
            Form::new()
        }
        fn fetch_rows(&self, _q: &QueryState) -> (Vec<RowData>, usize) {
            (vec![], 0)
        }
    }

    struct BillingModule;
    impl AppModule for BillingModule {
        fn name(&self) -> &str {
            "Billing"
        }
        fn slug(&self) -> &str {
            "billing"
        }
        fn description(&self) -> Option<&str> {
            Some("Enterprise invoice and billing management")
        }
        fn routes(&self) -> axum::Router {
            axum::Router::new().route("/summary", get(|| async { ApiResponse::ok("summary_ok") }))
        }
        fn resource(&self) -> Option<std::sync::Arc<dyn Resource>> {
            Some(std::sync::Arc::new(InvoiceResource))
        }
    }

    let panel = AdminPanel::new().module(BillingModule);
    assert_eq!(panel.get_modules().len(), 1);
    assert_eq!(panel.get_modules()[0].name(), "Billing");
    assert_eq!(panel.get_modules()[0].slug(), "billing");

    // Router mounts both the admin UI routes and the nested /api/billing module routes
    let router = panel.into_router();
    let _ = router;

    // 4. DynamicModule programmatic builder
    let dyn_mod = DynamicModule::new("Shipping", "shipping")
        .with_description("Logistics tracking module")
        .with_routes(axum::Router::new().route(
            "/tracking",
            get(|| async { ApiResponse::ok("in_transit") }),
        ));

    assert_eq!(dyn_mod.name(), "Shipping");
    assert_eq!(dyn_mod.slug(), "shipping");
    assert_eq!(dyn_mod.description(), Some("Logistics tracking module"));
}

#[test]
fn test_cpanel_scaffolding_templates_and_configs() {
    use terminus::cpanel::*;

    // 1. Test .htaccess generation for different strategies
    let htaccess_hybrid = generate_htaccess(3000, "hybrid");
    assert!(htaccess_hybrid.contains("RewriteEngine On"));
    assert!(htaccess_hybrid.contains("http://127.0.0.1:3000/$1 [P,L]"));
    assert!(htaccess_hybrid.contains("RewriteRule ^(.*)$ index.php [QSA,L]"));
    assert!(htaccess_hybrid.contains("<FilesMatch \"\\.(db|sqlite|sqlite3|env|sh|log|pid)$\">"));
    assert!(htaccess_hybrid.contains("Require all denied"));

    let htaccess_php = generate_htaccess(8080, "php");
    assert!(htaccess_php.contains("RewriteRule ^(.*)$ index.php [QSA,L]"));
    assert!(!htaccess_php.contains("mod_proxy.c"));

    let htaccess_proxy = generate_htaccess(4000, "htaccess");
    assert!(htaccess_proxy.contains("http://127.0.0.1:4000/$1 [P,L]"));
    assert!(!htaccess_proxy.contains("RewriteRule ^(.*)$ index.php"));

    // 2. Test PHP reverse proxy & supervisor bridge
    let php_bridge = generate_php_bridge(3000, "my_app");
    assert!(php_bridge.contains("$appPort    = 3000;"));
    assert!(php_bridge.contains("$binaryName = 'my_app';"));
    assert!(php_bridge.contains("function isPortOpen("));
    assert!(php_bridge.contains("fsockopen"));
    assert!(php_bridge.contains("curl_init"));
    assert!(php_bridge.contains("X-Forwarded-For"));
    assert!(php_bridge.contains("X-Forwarded-Proto"));
    assert!(php_bridge.contains("http_response_code($httpCode);"));

    // 3. Test Cron watchdog script
    let watchdog = generate_watchdog(3000, "my_app");
    assert!(watchdog.contains("BIN_NAME=\"my_app\""));
    assert!(watchdog.contains("PORT=3000"));
    assert!(watchdog.contains("curl -s -o /dev/null -w \"%{http_code}\""));
    assert!(watchdog.contains("nohup \"$BIN_PATH\" >> \"$LOG_PATH\" 2>&1 &"));

    // 4. Test Build script & README
    let build_sh = generate_build_script("my_app");
    assert!(build_sh.contains("APP_NAME=\"my_app\""));
    assert!(build_sh.contains("x86_64-unknown-linux-musl"));

    let readme = generate_readme(3000, "my_app");
    assert!(readme.contains("# Deploying Terminus on cPanel Shared Hosting"));
    assert!(readme.contains("127.0.0.1:3000"));

    // 5. Test bundle scaffolding to temporary directory
    let temp_dir = std::env::temp_dir().join(format!("terminus_cpanel_test_{}", std::process::id()));
    let created = scaffold_cpanel_bundle(&temp_dir, 3000, "test_app", "hybrid").unwrap();
    assert_eq!(created.len(), 6);
    assert!(temp_dir.join("public_html/.htaccess").exists());
    assert!(temp_dir.join("public_html/index.php").exists());
    assert!(temp_dir.join("app/watchdog.sh").exists());
    assert!(temp_dir.join("app/.env.example").exists());
    assert!(temp_dir.join("build.sh").exists());
    assert!(temp_dir.join("README.md").exists());

    // Clean up temporary test directory
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_studio_tui_rendering_and_telemetry() {
    use terminus::studio::StudioApp;

    let mut app = StudioApp::new("terminus.db");

    // 1. Header and tabs
    let header = app.render_header();
    assert!(header.contains("TERMINUS STUDIO"));
    assert!(header.contains("HOST:"));

    let tabs = app.render_tabs();
    assert!(tabs.contains("Overview"));
    assert!(tabs.contains("Database"));
    assert!(tabs.contains("Cache & Redis"));
    assert!(tabs.contains("Queue"));
    assert!(tabs.contains("API Tokens"));
    assert!(tabs.contains("Mail Outbox"));
    assert!(tabs.contains("Audit Logs"));

    // 2. Individual screens
    let overview = app.render_overview();
    assert!(overview.contains("SYSTEM TELEMETRY"));
    assert!(overview.contains("DATABASE (SQLite)"));

    let db_screen = app.render_database();
    assert!(db_screen.contains("DATABASE EXPLORER"));

    let cache_screen = app.render_cache();
    assert!(cache_screen.contains("CACHE & REDIS TELEMETRY"));
    assert!(cache_screen.contains("MEMORY CACHE STORE"));

    let queue_screen = app.render_queue();
    assert!(queue_screen.contains("BACKGROUND JOB QUEUE"));

    let tokens_screen = app.render_tokens();
    assert!(tokens_screen.contains("PERSONAL ACCESS TOKENS"));

    let mail_screen = app.render_mail();
    assert!(mail_screen.contains("TRANSACTIONAL MAIL OUTBOX"));

    let audit_screen = app.render_audit();
    assert!(audit_screen.contains("SECURITY AUDIT TRAIL LOGS"));

    // 3. Tab switching and full screen rendering
    for tab in 1..=7 {
        app.active_tab = tab;
        let screen = app.render_screen();
        assert!(screen.contains("TERMINUS STUDIO"));
        assert!(screen.contains("Studio:"));
    }
}

#[test]
fn test_rich_form_fields_and_media_builders() {
    use terminus::form::{FieldType, Form, FormField};
    use terminus::table::{Column, ColumnType};

    let rt1 = FormField::rich_text("content");
    let rt2 = FormField::richText("body");
    assert_eq!(rt1.field_type, FieldType::RichText);
    assert_eq!(rt2.field_type, FieldType::RichText);

    let img = FormField::image("cover");
    let gal = FormField::gallery("photos");
    assert_eq!(img.field_type, FieldType::Image);
    assert_eq!(gal.field_type, FieldType::Gallery);

    let barcode = FormField::barcode("sku");
    assert_eq!(barcode.field_type, FieldType::Barcode);

    let color = FormField::color("accent_color");
    assert_eq!(color.field_type, FieldType::Color);

    let kv1 = FormField::key_value("metadata");
    let kv2 = FormField::keyValue("attributes");
    assert_eq!(kv1.field_type, FieldType::KeyValue);
    assert_eq!(kv2.field_type, FieldType::KeyValue);

    let col_img = Column::image("avatar");
    assert_eq!(col_img.name, "avatar");
    assert_eq!(col_img.column_type, ColumnType::Image);

    let form = Form::new()
        .field(rt1)
        .field(img)
        .field(barcode)
        .field(color);
    assert_eq!(form.fields.len(), 4);
}

#[test]
fn test_rich_fields_and_image_columns_rendering() {
    use terminus::form::{Form, FormField};
    use terminus::table::Column;

    struct ProductResource;
    impl terminus::resource::Resource for ProductResource {
        fn name(&self) -> &'static str { "Product" }
        fn plural_name(&self) -> &'static str { "Products" }
        fn slug(&self) -> &'static str { "products" }
        fn table(&self) -> terminus::table::Table {
            terminus::table::Table::new()
                .column(Column::text("title"))
                .column(Column::image("thumbnail"))
        }
        fn form(&self) -> Form {
            Form::new()
                .field(FormField::text("title"))
                .field(FormField::rich_text("description"))
                .field(FormField::image("thumbnail"))
                .field(FormField::barcode("sku"))
                .field(FormField::color("theme_color"))
        }
        fn fetch_rows(&self, _: &terminus::resource::QueryState) -> (Vec<terminus::resource::RowData>, usize) {
            (
                vec![
                    terminus::resource::RowData::new("1")
                        .insert("title", "Ergonomic Mechanical Keyboard")
                        .insert("thumbnail", "https://images.unsplash.com/photo-1587829741301-dc798b83add3")
                        .insert("sku", "KB-9002-RGB")
                        .insert("theme_color", "#3b82f6"),
                ],
                1,
            )
        }
    }

    let resource = ProductResource;
    let user = User::default_admin();

    // 1. Test table view rendering for ColumnType::Image
    let query = terminus::resource::QueryState::default();
    let table_html = terminus::view::table_view::render_table_partial(&resource, &query, &user);
    assert!(table_html.contains("img src=\"https://images.unsplash.com/photo-1587829741301-dc798b83add3\""));
    assert!(table_html.contains("alt=\"Thumbnail\""));

    // 2. Test dialog form rendering for rich fields
    let dialogs_html = terminus::view::dialogs::render_dialogs(&resource);
    assert!(dialogs_html.contains("formatRichText('create-field-description'"));
    assert!(dialogs_html.contains("title=\"Bold\">B</button>"));
    assert!(dialogs_html.contains("title=\"Code\">&lt;&gt;</button>"));
    assert!(dialogs_html.contains("updateImagePreview('create-field-thumbnail'"));
    assert!(dialogs_html.contains("placeholder=\"Scan barcode or enter SKU...\""));
    assert!(dialogs_html.contains("type=\"color\" id=\"picker-create-field-theme_color\""));

    // 3. Test full-page form rendering for rich fields with populated values
    let mut values = std::collections::HashMap::new();
    values.insert("title".to_string(), "Ergonomic Mechanical Keyboard".to_string());
    values.insert("description".to_string(), "## Specs\n- Hot-swappable switches".to_string());
    values.insert("thumbnail".to_string(), "https://images.unsplash.com/photo-1587829741301-dc798b83add3".to_string());
    values.insert("sku".to_string(), "KB-9002-RGB".to_string());
    values.insert("theme_color".to_string(), "#3b82f6".to_string());

    let page_html = terminus::view::form_page::render_form_page(&resource, Some("1"), Some(&values));
    assert!(page_html.contains("## Specs\n- Hot-swappable switches"));
    assert!(page_html.contains("formatRichText('page-field-description'"));
    assert!(page_html.contains("value=\"https://images.unsplash.com/photo-1587829741301-dc798b83add3\""));
    assert!(page_html.contains("value=\"KB-9002-RGB\""));
    assert!(page_html.contains("value=\"#3b82f6\""));
}

#[test]
fn test_global_helpers_and_ergonomics() {
    use chrono::Datelike;
    use terminus::prelude::*;

    // 1. Without authenticated user
    set_current_user(None);
    assert!(auth().is_none());
    assert!(user().is_none());
    assert!(!can("orders.view"));
    assert!(cannot("orders.view"));

    // 2. With with_user scoped helper
    let admin = User::new("u_99", "Sarah Admin", "sarah@example.com", Role::Admin);
    with_user(admin.clone(), || {
        assert!(auth().is_some());
        assert_eq!(user_id(), Some("u_99".to_string()));
        assert_eq!(user().unwrap().name, "Sarah Admin");
        assert!(can("orders.view"));
        assert!(cannot("unauthorized.action"));
        assert!(can_any(&["orders.view", "unknown"]));
        assert!(!can_all(&["orders.view", "unknown"]));
        assert!(is_admin());
        assert!(!is_superadmin());
    });

    // 3. Thread/Task context restoration
    assert!(auth().is_none());

    // 4. Request inspection helpers
    let req_info = RequestInfo::new("GET", "/admin/orders")
        .with_query("search", "Mechanical")
        .with_query("page", "2")
        .with_header("Accept", "application/json")
        .with_header("X-Requested-With", "XMLHttpRequest")
        .with_ip("192.168.1.50");

    with_request(req_info, || {
        assert_eq!(request().method(), "GET");
        assert_eq!(request().path(), "/admin/orders");
        assert_eq!(request().input("search"), Some("Mechanical".to_string()));
        assert_eq!(request().input("page"), Some("2".to_string()));
        assert!(request().has("search"));
        assert!(!request().has("missing_filter"));
        assert_eq!(request().ip(), "192.168.1.50");
        assert!(request().wants_json());
        assert!(request().is_ajax());
    });

    // 5. Abort flow control helpers
    assert!(abort_if(false, 403, "Forbidden").is_ok());
    assert!(abort_if(true, 403, "Forbidden").is_err());
    assert_eq!(abort_if(true, 403, "Access Denied").unwrap_err().status, 403);
    assert!(abort_unless(true, 404, "Not Found").is_ok());
    assert!(abort_unless(false, 404, "Not Found").is_err());
    assert_eq!(abort_404("Missing Record").unwrap_err().status, 404);
    assert_eq!(abort_403("Forbidden Area").unwrap_err().status, 403);

    // 6. DateTime and config utilities
    assert!(!today().is_empty());
    assert!(today().contains('-'));
    assert!(timestamp() > 1_700_000_000);
    let n = now();
    assert_eq!(n.year(), 2026);

    config_set("app.name", "Terminus Platform");
    assert_eq!(config("app.name"), Some("Terminus Platform".to_string()));
}

#[test]
fn test_app_bootstrap_and_routing() {
    use terminus::prelude::*;

    let app = App::new()
        .name("Terminus Shop")
        .port(8080)
        .host("0.0.0.0")
        .cors(true);

    assert_eq!(app.name, "Terminus Shop");
    assert_eq!(app.port, 8080);
    assert_eq!(app.host, "0.0.0.0");
    assert!(app.cors_enabled);

    let router = app.build();
    // Verify router builds without panic
    let _ = router;

    let res = json_response(serde_json::json!({"status": "ok", "app": "Terminus"}));
    assert_eq!(res.status(), axum::http::StatusCode::OK);

    let html = html_response("<h1>Welcome</h1>");
    assert!(html.0.contains("Welcome"));

    let red = redirect_to("/admin");
    let _ = red;
}

#[test]
fn test_advanced_filters_and_presets() {
    use terminus::prelude::*;
    use std::collections::HashMap;

    let filter_date = Filter::date_range("created_at").label("Creation Date");
    assert_eq!(filter_date.key, "created_at");
    assert_eq!(filter_date.label, "Creation Date");

    let filter_slider = Filter::range_slider("price", 0.0, 500.0, 10.0);
    assert_eq!(filter_slider.key, "price");

    let filter_multi = Filter::multi_select("status", vec![("paid", "Paid"), ("pending", "Pending")]);
    assert_eq!(filter_multi.key, "status");

    let preset = FilterPreset::make("vip_orders", "VIP High Spend Orders")
        .with("total_min", "500")
        .with("status", "paid");
    assert_eq!(preset.criteria.get("total_min"), Some(&"500".to_string()));

    // Test filter evaluation engine
    let mut row_values = HashMap::new();
    row_values.insert("price".to_string(), "150".to_string());
    row_values.insert("status".to_string(), "paid".to_string());
    row_values.insert("created_at".to_string(), "2026-05-10".to_string());

    let mut active_filters = HashMap::new();
    active_filters.insert("price_min".to_string(), "100".to_string());
    active_filters.insert("price_max".to_string(), "200".to_string());
    active_filters.insert("status".to_string(), "paid,shipped".to_string());
    active_filters.insert("created_at_from".to_string(), "2026-01-01".to_string());

    assert!(matches_filters(&row_values, &active_filters));

    // Test non-matching filter
    let mut non_matching = active_filters.clone();
    non_matching.insert("price_max".to_string(), "120".to_string());
    assert!(!matches_filters(&row_values, &non_matching));
}

#[test]
fn test_excel_and_pdf_document_generation() {
    use terminus::prelude::*;

    // 1. Excel OpenXML & CSV
    let mut excel = ExcelDocument::new("Orders")
        .headers(vec!["Order ID", "Customer", "Total", "Status"]);

    excel.add_row(vec!["ORD-101", "Alice", "$199.00", "Completed"]);
    excel.add_row(vec!["ORD-102", "Bob", "$89.50", "Pending"]);

    let csv = excel.to_csv();
    assert!(csv.contains("Order ID,Customer,Total,Status"));
    assert!(csv.contains("ORD-101,Alice,$199.00,Completed"));

    let xml = excel.to_excel_xml();
    assert!(xml.contains("urn:schemas-microsoft-com:office:spreadsheet"));
    assert!(xml.contains("<Worksheet ss:Name=\"Orders\">"));
    assert!(xml.contains("<Data ss:Type=\"String\">ORD-101</Data>"));

    // 2. Native PDF Document
    let pdf = PdfDocument::new("INVOICE #9021")
        .company("Terminus Tech", "100 Market St, SF")
        .recipient("Acme Corp", "200 Broadway, NY")
        .table(
            vec!["Item", "Quantity", "Rate", "Amount"],
            vec![
                vec!["Terminus Cloud License", "1", "$299.00", "$299.00"],
                vec!["Support SLA", "1", "$99.00", "$99.00"],
            ],
        )
        .total("$398.00")
        .notes("Payment due within 14 days.");

    let pdf_bytes = pdf.to_pdf_bytes();
    assert!(pdf_bytes.starts_with(b"%PDF-1.4"));
    let pdf_str = String::from_utf8_lossy(&pdf_bytes);
    assert!(pdf_str.contains("INVOICE #9021"));
    assert!(pdf_str.contains("Terminus Tech"));
    assert!(pdf_str.contains("TOTAL AMOUNT: $398.00"));
    assert!(pdf_str.contains("%%EOF"));
}

#[test]
fn test_passkeys_oauth_and_payments() {
    use terminus::prelude::*;

    // 1. Passkey Challenge
    let challenge = PasskeyChallenge::generate("usr_10", "pratik", "terminus.dev");
    assert!(challenge.challenge.starts_with("passkey_challenge_usr_10_"));
    assert_eq!(challenge.user_name, "pratik");
    assert_eq!(challenge.relying_party, "terminus.dev");

    // 2. OAuth Provider
    let github = OAuthProvider::GitHub {
        client_id: "gh_client_123".to_string(),
        client_secret: "gh_secret_456".to_string(),
    };
    let auth_url = github.authorization_url("https://terminus.dev/auth/callback", "state_abc");
    assert!(auth_url.contains("https://github.com/login/oauth/authorize"));
    assert!(auth_url.contains("client_id=gh_client_123"));
    assert!(auth_url.contains("state=state_abc"));

    // 3. Payment Webhook verification & parsing
    let raw_event = serde_json::json!({
        "type": "payment_intent.succeeded",
        "data": {
            "id": "pi_12345",
            "amount": 4900,
            "currency": "usd",
            "customer_email": "buyer@example.com"
        }
    }).to_string();

    assert!(PaymentWebhook::verify_signature(&raw_event, "test_sig", "whsec_abc"));
    let event = PaymentWebhook::parse_event(&raw_event).unwrap();
    match event {
        PaymentEvent::PaymentSucceeded { transaction_id, amount_cents, currency, customer_email } => {
            assert_eq!(transaction_id, "pi_12345");
            assert_eq!(amount_cents, 4900);
            assert_eq!(currency, "usd");
            assert_eq!(customer_email, "buyer@example.com");
        }
        _ => panic!("Expected PaymentSucceeded"),
    }
}

#[test]
fn test_multi_platform_mobile_iot_desktop() {
    use terminus::prelude::*;

    // 1. Mobile Push Notification
    let push = PushNotification::new("New Order Alert", "Order #4092 received", "fcm_device_token_xyz")
        .with_data("order_id", "4092");

    let fcm = push.to_fcm_payload();
    assert_eq!(fcm["message"]["token"], "fcm_device_token_xyz");
    assert_eq!(fcm["message"]["notification"]["title"], "New Order Alert");
    assert_eq!(fcm["message"]["data"]["order_id"], "4092");

    // 2. IoT Telemetry Packet
    let telemetry = DeviceTelemetry::new("sensor_hub_01")
        .metric("temperature", 24.5)
        .metric("humidity", 58.2)
        .battery(94.0);

    assert_eq!(telemetry.device_id, "sensor_hub_01");
    assert_eq!(telemetry.metrics.get("temperature"), Some(&24.5));
    assert_eq!(telemetry.battery_percent, Some(94.0));

    // 3. Desktop IPC Message
    let desktop_msg = DesktopMessage::new("window_focus", serde_json::json!({"focused": true}));
    assert_eq!(desktop_msg.command, "window_focus");
}

#[test]
fn test_sortable_tables_and_draggable_blocks() {
    use terminus::prelude::*;

    // 1. Editable Columns & Sortable Rows in Table
    let table = Table::new()
        .column(Column::text("title").editable())
        .column(Column::text("status").editable_select(vec!["draft", "published", "archived"]))
        .sortable_rows();

    assert!(table.reorderable);
    assert!(table.columns[0].editable);
    assert!(table.columns[1].editable);
    assert_eq!(
        table.columns[1].editable_options,
        Some(vec!["draft".to_string(), "published".to_string(), "archived".to_string()])
    );

    // 2. Draggable UI blocks in Form
    let field = FormField::draggable_blocks("page_content", vec!["hero", "features", "pricing", "cta"]);
    match field.field_type {
        FieldType::DraggableBlocks { block_types } => {
            assert_eq!(block_types.len(), 4);
            assert_eq!(block_types[0], "hero");
            assert_eq!(block_types[2], "pricing");
        }
        _ => panic!("Expected DraggableBlocks"),
    }
}

#[test]
fn test_platform_adapters_and_sdk_generator() {
    use terminus::prelude::*;

    // 1. Platform detection from user-agent
    assert_eq!(Platform::from_user_agent("Mozilla/5.0 (Linux; Android 14)"), Platform::Android);
    assert_eq!(Platform::from_user_agent("Mozilla/5.0 (iPhone; CPU iPhone OS 17)"), Platform::Ios);
    assert_eq!(Platform::from_user_agent("Tauri/2.0 Desktop"), Platform::Desktop);
    assert_eq!(Platform::from_user_agent("ESP32-Scanner-Client"), Platform::Iot);
    assert_eq!(Platform::from_user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X)"), Platform::Web);

    // 2. Push Dispatcher with APNs and delivery
    let dispatcher = PushDispatcher::new();
    let n1 = PushNotification::new("Order Shipped", "Package #901 has dispatched", "tok_1").badge(2);
    let apns = n1.to_apns_payload();
    assert_eq!(apns["aps"]["badge"], 2);
    assert_eq!(apns["aps"]["sound"], "default");

    dispatcher.enqueue(n1);
    assert_eq!(dispatcher.pending_count(), 1);
    let delivered = dispatcher.deliver_all();
    assert_eq!(delivered.len(), 1);
    assert_eq!(dispatcher.pending_count(), 0);

    // 3. Typed SDK Generator (TypeScript & Dart)
    let gen = SdkGenerator::new("Acme Logistics", "1.0")
        .type_schema("Item", vec![("id", "string"), ("stock", "number")])
        .endpoint("GET", "/items", None, "Item[]", "Fetch items")
        .endpoint("POST", "/items", Some("Item"), "Item", "Create item");

    let ts = gen.to_typescript();
    assert!(ts.contains("export interface Item"));
    assert!(ts.contains("export class AcmeLogisticsClient"));
    assert!(ts.contains("public async get_items(): Promise<Item[]>"));
    assert!(ts.contains("public async post_items(data: Item): Promise<Item>"));

    let dart = gen.to_dart();
    assert!(dart.contains("class AcmeLogisticsClient"));
    assert!(dart.contains("import 'package:http/http.dart'"));

    // 4. Desktop Offline Sync Queue
    let sync_queue = OfflineSyncQueue::new();
    let tx1 = sync_queue.enqueue(
        "update_inventory",
        "/api/inventory/update",
        serde_json::json!({"sku": "SKU_1", "qty": 5}),
        "idemp_key_1",
    );
    assert_eq!(sync_queue.pending_count(), 1);
    let pending = sync_queue.pending_records();
    assert_eq!(pending[0].id, tx1);
    assert_eq!(pending[0].action, "update_inventory");

    assert!(sync_queue.mark_synced(&tx1));
    assert_eq!(sync_queue.pending_count(), 0);

    // 5. IoT Device Registry & Hardware Scanner
    let registry = DeviceRegistry::new();
    registry.register("dev_scanner_1", "Warehouse Handheld 1", "psk_secret_key_123", "scanner");
    assert!(registry.is_registered("dev_scanner_1"));
    assert!(!registry.is_registered("unknown_dev"));

    assert!(registry.verify_signature("dev_scanner_1", "payload_data", "psk_secret_key_123"));
    assert!(!registry.verify_signature("dev_scanner_1", "payload_data", "wrong_secret"));
    assert!(registry.heartbeat("dev_scanner_1"));

    let scan = BarcodeScanEvent::new("dev_scanner_1", "079357318921", "UPC-A")
        .location("Aisle-4-Shelf-2");
    assert_eq!(scan.device_id, "dev_scanner_1");
    assert_eq!(scan.barcode, "079357318921");
    assert_eq!(scan.symbology, "UPC-A");
    assert_eq!(scan.location, Some("Aisle-4-Shelf-2".to_string()));
}


