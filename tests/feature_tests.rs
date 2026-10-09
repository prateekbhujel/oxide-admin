use oxide_admin::prelude::*;
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
        vec![("1", "Pratik Bhujel"), ("2", "Dharma Raj"), ("3", "Lasta Chaudhary")],
    );

    assert_eq!(field.name, "assignee");
    assert_eq!(field.label, "Assignee");
    match field.field_type {
        oxide_admin::form::FieldType::SearchableSelect { options } => {
            assert_eq!(options.len(), 3);
            assert_eq!(options[0], ("1".to_string(), "Pratik Bhujel".to_string()));
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

    let row2 = RowData::new("2").insert("user_name", "Pratik Bhujel");
    assert_eq!(row2.get("userName"), Some("Pratik Bhujel"));
    assert_eq!(row2.get("user_name"), Some("Pratik Bhujel"));
}

#[test]
fn test_laravel_dx_camel_case_methods() {
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
        name: "Pratik Bhujel".into(),
        email: "pratik@example.com".into(),
        role: Role::Superadmin,
        status: UserStatus::Active,
        created_at: "2026-10-08".into(),
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

struct FilamentPolicyResource;
impl Resource for FilamentPolicyResource {
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
fn test_filament_record_level_authorization_hooks() {
    let res = FilamentPolicyResource;
    let admin_user = User {
        id: "1".into(),
        name: "Admin".into(),
        email: "admin@example.com".into(),
        role: Role::Superadmin,
        status: UserStatus::Active,
        created_at: "2026-10-08".into(),
    };
    let member_user = User {
        id: "2".into(),
        name: "Member".into(),
        email: "member@example.com".into(),
        role: Role::Member,
        status: UserStatus::Active,
        created_at: "2026-10-08".into(),
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
fn test_filament_policy_table_view_action_buttons() {
    let res = FilamentPolicyResource;
    let admin_user = User {
        id: "1".into(),
        name: "Admin".into(),
        email: "admin@example.com".into(),
        role: Role::Superadmin,
        status: UserStatus::Active,
        created_at: "2026-10-08".into(),
    };

    let query = QueryState::default();
    let html = oxide_admin::view::table_view::render_table_partial(&res, &query, &admin_user);

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
    };

    let html = oxide_admin::view::layout::render_page(
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
        .subject("Welcome to OxideAdmin")
        .line("Your enterprise admin kit is ready.")
        .action("Access Panel", "/admin");

    assert_eq!(msg.to, "founder@startup.io");
    assert_eq!(msg.subject, "Welcome to OxideAdmin");
    assert_eq!(msg.lines.len(), 1);

    msg.send_via(&mailer).unwrap();

    let outbox = mailer.sent_messages();
    assert_eq!(outbox.len(), 1);
    assert_eq!(outbox[0].to, "founder@startup.io");
    assert_eq!(outbox[0].subject, "Welcome to OxideAdmin");
}

#[test]
fn test_eloquent_query_builder_sql_and_bindings() {
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
fn test_pest_style_fluent_expectations() {
    use oxide_admin::prelude::*;

    pest_test("it validates user properties with Pest syntax", || {
        let user = User {
            id: "u_42".to_string(),
            name: "Taylor".to_string(),
            email: "taylor@laravel.com".to_string(),
            role: Role::Superadmin,
            status: UserStatus::Active,
            created_at: "2026-10-08".to_string(),
        };

        // Pest-style chainable assertions
        expect(&user.name).to_be("Taylor")
            .and(&user.email).to_contain("@laravel.com")
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
    use oxide_admin::dump;
    let data = vec!["oxide", "admin", "fast"];
    // Verify dump! macro executes smoothly without errors
    dump!(data);
}

#[tokio::test]
async fn test_websocket_broadcaster_pubsub() {
    use oxide_admin::prelude::*;

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

struct FilamentParityResource {
    records: std::sync::Mutex<HashMap<String, HashMap<String, String>>>,
}

impl FilamentParityResource {
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

impl Resource for FilamentParityResource {
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
fn test_filament_actions_and_replicate_row() {
    let res = FilamentParityResource::new();

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

    // Test record replication hook (Filament replicate)
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
    let res = FilamentParityResource::new();

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
    // Filament-grade Section layout with columns and schema
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
    let res = FilamentParityResource::new();
    let admin_user = User {
        id: "1".into(),
        name: "Admin".into(),
        email: "admin@example.com".into(),
        role: Role::Superadmin,
        status: UserStatus::Active,
        created_at: "2026-10-08".into(),
    };

    let query = QueryState::default();
    let html = oxide_admin::view::table_view::render_table_partial(&res, &query, &admin_user);

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
    let empty_html = oxide_admin::view::table_view::render_table_partial(&EmptyResource, &empty_query, &admin_user);
    assert!(empty_html.contains("No orders found"));
    assert!(empty_html.contains("No customer orders recorded yet."));
}


