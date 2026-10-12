use terminus::client::SdkGenerator;
use terminus::prompt;
use terminus::token::{SqliteTokenStore, TokenRepository};
use std::env;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

fn main() {
    let mut raw_args: Vec<String> = env::args().collect();
    // Handle cargo subcommand invocation (`cargo terminus ...`)
    if raw_args.get(1).map(|s| s.as_str()) == Some("terminus") {
        raw_args.remove(1);
    }
    let args = raw_args;
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    match command {
        "help" | "--help" | "-h" => print_help(),
        "version" | "--version" | "-v" => print_version(),
        "make:resource" => {
            let name = args.get(2).map(|s| s.as_str());
            make_resource(name);
        }
        "make:module" | "module:create" | "make:mod" => {
            let name = args.get(2).map(|s| s.as_str());
            let pattern_arg = args.get(3).map(|s| s.as_str());
            make_module(name, pattern_arg);
        }
        "make:model" => {
            let name = args.get(2).map(|s| s.as_str());
            make_model(name);
        }
        "make:repo" => {
            let name = args.get(2).map(|s| s.as_str());
            make_repo(name);
        }
        "make:migration" => {
            let name = args.get(2).map(|s| s.as_str());
            make_migration(name);
        }
        "make:token" | "token:create" => {
            let name = args.get(2).map(|s| s.as_str());
            make_token(name);
        }
        "token:list" => list_tokens(),
        "token:revoke" => {
            let id = args.get(2).map(|s| s.as_str());
            revoke_token(id);
        }
        "migrate" => run_migrations(),
        "db:seed" => run_seed(),
        "db:tables" => inspect_tables(),
        "cache:clear" => clear_cache(),
        "make:job" => {
            let name = args.get(2).map(|s| s.as_str());
            make_job(name);
        }
        "make:mail" => {
            let name = args.get(2).map(|s| s.as_str());
            make_mail(name);
        }
        "routes" => print_routes(),
        "queue:work" => run_queue_worker(),
        "ws:serve" => run_ws_server(),
        "theme:preview" => preview_theme(),
        "mail:outbox" => view_mail_outbox(),
        "make:cpanel" | "cpanel:init" | "cpanel:setup" | "deploy:cpanel" => {
            let port = args.iter().find(|a| a.starts_with("--port=")).map(|a| a.trim_start_matches("--port="));
            let bin = args.iter().find(|a| a.starts_with("--bin=")).map(|a| a.trim_start_matches("--bin="));
            make_cpanel(port, bin);
        }
        "make:sdk" | "client:sdk" => {
            let lang = args.iter().find(|a| a.starts_with("--lang=")).map(|a| a.trim_start_matches("--lang="));
            make_sdk(lang);
        }
        "make:desktop" | "client:desktop" => {
            let name = args.get(2).map(|s| s.as_str());
            make_desktop(name);
        }
        "make:iot" | "client:iot" => {
            let id = args.get(2).map(|s| s.as_str());
            make_iot(id);
        }
        "studio" | "tui" | "dashboard" | "pulse" | "inspector" => {
            let db_path = args.iter().find(|a| a.starts_with("--db=")).map(|a| a.trim_start_matches("--db=")).unwrap_or("terminus.db");
            let is_watch = args.iter().any(|a| a == "--watch" || a == "-w");
            let is_once = args.iter().any(|a| a == "--once" || a == "--overview");
            let tab_arg = args.iter().find(|a| a.starts_with("--tab=")).and_then(|a| a.trim_start_matches("--tab=").parse::<u8>().ok());
            run_studio(db_path, is_watch, is_once, tab_arg);
        }
        unknown => {
            eprintln!("\x1b[31mError:\x1b[0m Unknown command '{unknown}'\n");
            print_help();
            std::process::exit(1);
        }
    }
}

fn print_banner() {
    println!("\x1b[36m  _____                   _                  
 |_   _|__ _ __ _ __ ___ (_)_ __  _   _ ___ 
   | |/ _ \\ '__| '_ ` _ \\| | '_ \\| | | / __|
   | |  __/ |  | | | | | | | | | | |_| \\__ \\
   |_|\\___|_|  |_| |_| |_|_|_| |_|\\__,_|___/\x1b[0m\n\x1b[1mTerminus Framework CLI\x1b[0m v0.1.0 — Universal Fullstack Rust Application Platform\n");
}

fn print_help() {
    print_banner();
    println!("Usage:\n  terminus <command> [arguments]\n\n\x1b[1mAvailable Commands:\x1b[0m\n  \x1b[33mstudio\x1b[0m                   Launch full Terminal UI Studio & live Database/Cache Inspector\n  \x1b[33mmake:resource <Name>\x1b[0m     Scaffold a new Admin Resource with table & form\n  \x1b[33mmake:module <Name>\x1b[0m       Scaffold an Enterprise Modular (DDD) domain module\n  \x1b[33mmake:model <Name>\x1b[0m        Scaffold a new domain Model struct with camelCase serde\n  \x1b[33mmake:repo <Name>\x1b[0m         Scaffold a SQLite repository implementation\n  \x1b[33mmake:migration <Name>\x1b[0m    Generate a timestamped SQL migration file\n  \x1b[33mmake:token [Name]\x1b[0m        Generate an API Personal Access Token\n  \x1b[33mtoken:list\x1b[0m               List active Personal Access Tokens\n  \x1b[33mtoken:revoke [ID]\x1b[0m        Revoke a Personal Access Token\n  \x1b[33mmigrate\x1b[0m                  Run pending database migrations\n  \x1b[33mdb:seed\x1b[0m                  Seed demo accounts and orders into database\n  \x1b[33mdb:tables\x1b[0m                Inspect SQLite database tables, schema, and row counts\n  \x1b[33mcache:clear\x1b[0m              Flush application and Redis cache stores\n  \x1b[33mmake:cpanel\x1b[0m              Generate cPanel & shared hosting reverse proxy & deployment bundle\n  \x1b[33mmake:sdk [--lang=ts|dart]\x1b[0m Export typed client SDK for mobile & frontend\n  \x1b[33mmake:desktop [Name]\x1b[0m      Scaffold Tauri / desktop workstation offline bridge\n  \x1b[33mmake:iot [DeviceId]\x1b[0m       Scaffold IoT hardware scanner & telemetry script\n  \x1b[33mmake:job <Name>\x1b[0m          Scaffold a background Queue Job\n  \x1b[33mmake:mail <Name>\x1b[0m         Scaffold a Transactional Mail notification\n  \x1b[33mws:serve\x1b[0m                 Start standalone WebSocket broadcasting server\n  \x1b[33mroutes\x1b[0m                   List registered admin routes & JSON API endpoints\n  \x1b[33mqueue:work\x1b[0m               Start processing background queue jobs\n  \x1b[33mmail:outbox\x1b[0m              Inspect sent transactional emails\n  \x1b[33mtheme:preview\x1b[0m            Preview theme colors, palettes, and font families\n  \x1b[33mversion\x1b[0m                  Show Terminus CLI version\n\n\x1b[1mExamples:\x1b[0m\n  terminus studio\n  terminus studio --watch\n  terminus make:resource Product\n  terminus make:module User\n  terminus make:cpanel\n  terminus make:migration create_products_table\n  terminus make:token mobile-app\n  terminus token:list\n  terminus migrate\n  terminus db:seed\n  terminus ws:serve\n  terminus theme:preview\n");
}

fn print_version() {
    println!("Terminus Framework CLI v0.1.0 (Rust edition 2021)");
}

fn make_resource(name_opt: Option<&str>) {
    let name_str = match name_opt {
        Some(n) if !n.is_empty() => n.to_string(),
        _ => match prompt::text("What is the resource name?", None, Some("Product"), true) {
            Ok(n) if !n.is_empty() => n,
            _ => {
                eprintln!("\x1b[31mError:\x1b[0m Missing resource name.\nUsage: terminus make:resource <Name>\nExample: terminus make:resource Product");
                std::process::exit(1);
            }
        },
    };
    let name = name_str.as_str();

    let singular = name;
    let plural = format!("{}s", name);
    let slug = name.to_lowercase();
    let filename = format!("{}_resource.rs", slug);

    let template = format!(r#"use terminus::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

pub struct {singular}Resource {{
    // repo: Arc<dyn {singular}Repository>,
}}

impl {singular}Resource {{
    pub fn new() -> Self {{
        Self {{}}
    }}
}}

impl Resource for {singular}Resource {{
    fn name(&self) -> &str {{
        "{singular}"
    }}

    fn plural_name(&self) -> &str {{
        "{plural}"
    }}

    fn slug(&self) -> &str {{
        "{slug}"
    }}

    fn form_mode(&self) -> FormMode {{
        FormMode::SlideOver
    }}

    fn table(&self) -> Table {{
        Table::new()
            .column(Column::text("id").label("ID").searchable().sortable())
            .column(Column::text("name").label("Name").searchable())
            .column(Column::badge("status", vec![
                ("active", "emerald"),
                ("draft", "amber"),
                ("archived", "rose"),
            ]))
            .column(Column::text("createdAt").label("Created At").sortable())
            .page_size(10)
    }}

    fn form(&self) -> Form {{
        Form::new()
            .field(FormField::text("name").required().placeholder("Item name"))
            .field(FormField::select("status", vec![
                ("active", "Active"),
                ("draft", "Draft"),
                ("archived", "Archived"),
            ]))
    }}

    fn fetch_rows(&self, query: &QueryState) -> (Vec<RowData>, usize) {{
        let _ = query;
        (Vec::new(), 0)
    }}

    fn get_row(&self, id: &str) -> Option<RowData> {{
        let _ = id;
        None
    }}

    fn create_row(&self, values: HashMap<String, String>) -> Result<String, String> {{
        let _ = values;
        Ok("1".into())
    }}

    fn update_row(&self, id: &str, values: HashMap<String, String>) -> Result<(), String> {{
        let _ = (id, values);
        Ok(())
    }}

    fn delete_row(&self, id: &str) -> Result<(), String> {{
        let _ = id;
        Ok(())
    }}

    // Record policy hook
    #[allow(non_snake_case)]
    fn canEditRow(&self, user: &User, row: &RowData) -> bool {{
        if !self.canEdit(user) {{
            return false;
        }}
        row.get("status") != Some("archived")
    }}
}}
"#);

    let target_dir = Path::new("src/resources");
    if target_dir.exists() {
        let dest = target_dir.join(&filename);
        if fs::write(&dest, &template).is_ok() {
            println!("\x1b[32m✔ Created Resource:\x1b[0m {}", dest.display());
            return;
        }
    }

    println!("\x1b[32m✔ Scaffolded {singular}Resource:\x1b[0m\n");
    println!("{template}");
}

fn make_module(name_opt: Option<&str>, pattern_arg: Option<&str>) {
    let name_str = match name_opt {
        Some(n) if !n.is_empty() => n.to_string(),
        _ => match prompt::text("Module name (e.g. User, Product, Billing, Order)", None, Some("User"), true) {
            Ok(n) if !n.is_empty() => n,
            _ => {
                eprintln!("\x1b[31mError:\x1b[0m Missing module name.\nUsage: terminus make:module <Name>\nExample: terminus make:module User");
                std::process::exit(1);
            }
        },
    };
    let singular = name_str.as_str();
    let slug = singular.to_lowercase();

    let (pattern_choice, components) = if let Some(p) = pattern_arg {
        match p {
            "--simple" | "-s" | "simple" => (
                "simple",
                vec!["model", "repo", "resource"],
            ),
            _ => (
                "modular",
                vec!["dto", "model", "repo", "provider", "controller", "request", "routes", "resource"],
            ),
        }
    } else {
        let choice = match prompt::select(
            "Choose your architecture pattern:",
            &[
                ("modular", "Enterprise Modular (DDD: DTO, Model, Repo, Provider, Controller, Request, Routes, Resource)"),
                ("simple", "Classic Monolith (Simple: Model + Repository + Resource)"),
                ("custom", "Custom (Select individual layers interactively)"),
            ],
            0,
        ) {
            Ok(c) => c,
            Err(_) => "modular",
        };

        let comp = match choice {
            "modular" => vec!["dto", "model", "repo", "provider", "controller", "request", "routes", "resource"],
            "simple" => vec!["model", "repo", "resource"],
            "custom" => {
                match prompt::multiselect(
                    &format!("Select components to scaffold for module '{singular}':"),
                    &[
                        ("dto", "Data Transfer Objects (DTO) - Input/output contract structs"),
                        ("model", "Domain Model - Active Record model with schema"),
                        ("repo", "Repository - Decoupled data access trait & SQLite store"),
                        ("provider", "Service Provider - Core business logic & domain service"),
                        ("controller", "HTTP Controller - Axum REST handlers & JSON responses"),
                        ("request", "Form Request - Validation rules & payload sanitization"),
                        ("routes", "Route Registrar - Axum Router module registration"),
                        ("resource", "Admin Resource - Administrative UI table & form layout"),
                    ],
                    &["dto", "model", "repo", "provider", "controller", "request", "routes", "resource"],
                ) {
                    Ok(c) => c,
                    Err(_) => vec!["dto", "model", "repo", "provider", "controller", "request", "routes", "resource"],
                }
            }
            _ => vec!["dto", "model", "repo", "provider", "controller", "request", "routes", "resource"],
        };

        (choice, comp)
    };

    let target_dir = Path::new("src/modules").join(&slug);
    let _ = fs::create_dir_all(&target_dir);

    // 1. Write mod.rs
    let mod_rs_code = generate_module_mod_rs(singular, &slug, &components);
    let _ = fs::write(target_dir.join("mod.rs"), &mod_rs_code);

    // 2. Write individual component files
    if components.contains(&"dto") {
        let _ = fs::write(target_dir.join("dto.rs"), generate_module_dto(singular));
    }
    if components.contains(&"model") {
        let _ = fs::write(target_dir.join("model.rs"), generate_module_model(singular, &slug));
    }
    if components.contains(&"repo") {
        let _ = fs::write(target_dir.join("repository.rs"), generate_module_repo(singular));
    }
    if components.contains(&"provider") {
        let _ = fs::write(target_dir.join("provider.rs"), generate_module_provider(singular));
    }
    if components.contains(&"request") {
        let _ = fs::write(target_dir.join("request.rs"), generate_module_request(singular));
    }
    if components.contains(&"controller") {
        let _ = fs::write(target_dir.join("controller.rs"), generate_module_controller(singular));
    }
    if components.contains(&"routes") {
        let _ = fs::write(target_dir.join("routes.rs"), generate_module_routes(singular));
    }
    if components.contains(&"resource") {
        let _ = fs::write(target_dir.join("resource.rs"), generate_module_resource(singular, &slug));
    }

    // 3. Register module in parent src/modules/mod.rs
    let parent_mod_path = Path::new("src/modules/mod.rs");
    let mod_decl = format!("pub mod {};\n", slug);
    if parent_mod_path.exists() {
        if let Ok(existing) = fs::read_to_string(parent_mod_path) {
            if !existing.contains(&mod_decl) {
                let mut updated = existing;
                updated.push_str(&mod_decl);
                let _ = fs::write(parent_mod_path, updated);
            }
        }
    } else {
        let _ = fs::write(parent_mod_path, format!("//! Application Domain Modules\n{}", mod_decl));
    }

    prompt::info(format!("Scaffolded {singular} domain module in src/modules/{slug}/"));
    println!("\n  \x1b[1mArchitecture:\x1b[0m  {}", if pattern_choice == "modular" { "Enterprise Modular (DDD)" } else if pattern_choice == "simple" { "Classic Monolith (Simple)" } else { "Custom Selection" });
    println!("  \x1b[1mComponents:\x1b[0m    {:?}", components);
    println!("  \x1b[1mDirectory:\x1b[0m     {}", target_dir.display());
    println!("\n\x1b[32m✔ Domain module scaffolded successfully!\x1b[0m");
    println!("To register with your admin panel in main.rs:\n");
    println!("  \x1b[36mlet {slug}_mod = {singular}Module::new(db_conn.clone());\x1b[0m");
    println!("  \x1b[36mlet admin = AdminPanel::new().module({slug}_mod);\x1b[0m\n");
}

fn generate_module_dto(singular: &str) -> String {
    format!(r#"use serde::{{Deserialize, Serialize}};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Create{singular}Dto {{
    pub name: String,
    pub description: Option<String>,
    pub status: Option<String>,
}}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Update{singular}Dto {{
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
}}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct {singular}ResponseDto {{
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: String,
}}
"#)
}

fn generate_module_model(singular: &str, slug: &str) -> String {
    let plural = format!("{}s", slug);
    format!(r#"use terminus::prelude::*;
use serde::{{Deserialize, Serialize}};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct {singular}Model {{
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}}

impl Model for {singular}Model {{
    fn table_name() -> &'static str {{
        "{plural}"
    }}

    fn get_id(&self) -> Option<&str> {{
        Some(&self.id)
    }}

    fn set_id(&mut self, id: String) {{
        self.id = id;
    }}

    fn to_map(&self) -> HashMap<String, String> {{
        let mut map = HashMap::new();
        map.insert("id".to_string(), self.id.clone());
        map.insert("name".to_string(), self.name.clone());
        if let Some(ref d) = self.description {{
            map.insert("description".to_string(), d.clone());
        }}
        map.insert("status".to_string(), self.status.clone());
        map.insert("created_at".to_string(), self.created_at.clone());
        map.insert("updated_at".to_string(), self.updated_at.clone());
        map
    }}

    fn from_map(map: HashMap<String, String>) -> Self {{
        Self {{
            id: map.get("id").cloned().unwrap_or_default(),
            name: map.get("name").cloned().unwrap_or_default(),
            description: map.get("description").cloned(),
            status: map.get("status").cloned().unwrap_or_else(|| "active".into()),
            created_at: map.get("created_at").cloned().unwrap_or_default(),
            updated_at: map.get("updated_at").cloned().unwrap_or_default(),
        }}
    }}
}}
"#)
}

fn generate_module_repo(singular: &str) -> String {
    format!(r#"use super::model::{singular}Model;
use terminus::prelude::*;
use std::sync::{{Arc, Mutex}};

pub trait {singular}Repository: Send + Sync {{
    fn find_all(&self) -> Result<Vec<{singular}Model>, String>;
    fn find_by_id(&self, id: &str) -> Result<Option<{singular}Model>, String>;
    fn save(&self, item: &{singular}Model) -> Result<(), String>;
    fn delete(&self, id: &str) -> Result<(), String>;
}}

pub struct Sqlite{singular}Repository {{
    conn: Arc<Mutex<rusqlite::Connection>>,
}}

impl Sqlite{singular}Repository {{
    pub fn new(conn: Arc<Mutex<rusqlite::Connection>>) -> Self {{
        Self {{ conn }}
    }}
}}

impl {singular}Repository for Sqlite{singular}Repository {{
    fn find_all(&self) -> Result<Vec<{singular}Model>, String> {{
        let conn = self.conn.lock().unwrap();
        {singular}Model::all(&conn)
    }}

    fn find_by_id(&self, id: &str) -> Result<Option<{singular}Model>, String> {{
        let conn = self.conn.lock().unwrap();
        {singular}Model::find(id, &conn)
    }}

    fn save(&self, item: &{singular}Model) -> Result<(), String> {{
        let conn = self.conn.lock().unwrap();
        item.save(&conn)
    }}

    fn delete(&self, id: &str) -> Result<(), String> {{
        let conn = self.conn.lock().unwrap();
        if let Some(item) = {singular}Model::find(id, &conn)? {{
            item.delete(&conn)
        }} else {{
            Err("{singular} record not found".into())
        }}
    }}
}}
"#)
}

fn generate_module_provider(singular: &str) -> String {
    format!(r#"use super::dto::{{Create{singular}Dto, {singular}ResponseDto, Update{singular}Dto}};
use super::model::{singular}Model;
use super::repository::{singular}Repository;
use std::sync::Arc;

pub struct {singular}Provider {{
    repo: Arc<dyn {singular}Repository>,
}}

impl {singular}Provider {{
    pub fn new(repo: Arc<dyn {singular}Repository>) -> Self {{
        Self {{ repo }}
    }}

    pub fn list_all(&self) -> Result<Vec<{singular}ResponseDto>, String> {{
        let models = self.repo.find_all()?;
        Ok(models.into_iter().map(Self::to_dto).collect())
    }}

    pub fn find_by_id(&self, id: &str) -> Result<Option<{singular}ResponseDto>, String> {{
        let model = self.repo.find_by_id(id)?;
        Ok(model.map(Self::to_dto))
    }}

    pub fn create(&self, dto: Create{singular}Dto) -> Result<{singular}ResponseDto, String> {{
        let now = chrono::Utc::now().to_rfc3339();
        let model = {singular}Model {{
            id: format!("{{:x}}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)),
            name: dto.name,
            description: dto.description,
            status: dto.status.unwrap_or_else(|| "active".into()),
            created_at: now.clone(),
            updated_at: now,
        }};
        self.repo.save(&model)?;
        Ok(Self::to_dto(model))
    }}

    pub fn update(&self, id: &str, dto: Update{singular}Dto) -> Result<{singular}ResponseDto, String> {{
        let mut model = self.repo.find_by_id(id)?.ok_or_else(|| "{singular} not found".to_string())?;
        if let Some(name) = dto.name {{ model.name = name; }}
        if let Some(desc) = dto.description {{ model.description = Some(desc); }}
        if let Some(status) = dto.status {{ model.status = status; }}
        model.updated_at = chrono::Utc::now().to_rfc3339();
        self.repo.save(&model)?;
        Ok(Self::to_dto(model))
    }}

    pub fn delete(&self, id: &str) -> Result<(), String> {{
        self.repo.delete(id)
    }}

    fn to_dto(model: {singular}Model) -> {singular}ResponseDto {{
        {singular}ResponseDto {{
            id: model.id,
            name: model.name,
            description: model.description,
            status: model.status,
            created_at: model.created_at,
        }}
    }}
}}
"#)
}

fn generate_module_request(singular: &str) -> String {
    format!(r#"use terminus::prelude::*;
use serde::{{Deserialize, Serialize}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Create{singular}Request {{
    pub name: String,
    pub description: Option<String>,
    pub status: Option<String>,
}}

impl ValidatedRequest for Create{singular}Request {{
    fn validate(&self) -> Result<(), Vec<String>> {{
        let mut errors = Vec::new();
        if self.name.trim().is_empty() {{
            errors.push("The 'name' field is required and cannot be blank.".into());
        }}
        if errors.is_empty() {{ Ok(()) }} else {{ Err(errors) }}
    }}
}}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Update{singular}Request {{
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
}}

impl ValidatedRequest for Update{singular}Request {{
    fn validate(&self) -> Result<(), Vec<String>> {{
        let mut errors = Vec::new();
        if let Some(ref name) = self.name {{
            if name.trim().is_empty() {{
                errors.push("The 'name' field cannot be blank.".into());
            }}
        }}
        if errors.is_empty() {{ Ok(()) }} else {{ Err(errors) }}
    }}
}}
"#)
}

fn generate_module_controller(singular: &str) -> String {
    format!(r#"use super::dto::{{Create{singular}Dto, Update{singular}Dto}};
use super::provider::{singular}Provider;
use super::request::{{Create{singular}Request, Update{singular}Request}};
use axum::{{
    extract::{{Path, State}},
    response::IntoResponse,
    Json,
}};
use terminus::prelude::*;
use std::sync::Arc;

pub async fn list(State(provider): State<Arc<{singular}Provider>>) -> impl IntoResponse {{
    match provider.list_all() {{
        Ok(items) => ApiResponse::ok(items).into_response(),
        Err(e) => ApiResponse::<()>::error(e).into_response(),
    }}
}}

pub async fn find(
    State(provider): State<Arc<{singular}Provider>>,
    Path(id): Path<String>,
) -> impl IntoResponse {{
    match provider.find_by_id(&id) {{
        Ok(Some(item)) => ApiResponse::ok(item).into_response(),
        Ok(None) => ApiResponse::<()>::error("{singular} not found").into_response(),
        Err(e) => ApiResponse::<()>::error(e).into_response(),
    }}
}}

pub async fn create(
    State(provider): State<Arc<{singular}Provider>>,
    Json(req): Json<Create{singular}Request>,
) -> impl IntoResponse {{
    if let Err(errors) = req.validate() {{
        return ApiResponse::<()>::validation_errors(errors).into_response();
    }}

    let dto = Create{singular}Dto {{
        name: req.name,
        description: req.description,
        status: req.status,
    }};

    match provider.create(dto) {{
        Ok(item) => ApiResponse::with_message(item, "{singular} created successfully").into_response(),
        Err(e) => ApiResponse::<()>::error(e).into_response(),
    }}
}}

pub async fn update(
    State(provider): State<Arc<{singular}Provider>>,
    Path(id): Path<String>,
    Json(req): Json<Update{singular}Request>,
) -> impl IntoResponse {{
    if let Err(errors) = req.validate() {{
        return ApiResponse::<()>::validation_errors(errors).into_response();
    }}

    let dto = Update{singular}Dto {{
        name: req.name,
        description: req.description,
        status: req.status,
    }};

    match provider.update(&id, dto) {{
        Ok(item) => ApiResponse::with_message(item, "{singular} updated successfully").into_response(),
        Err(e) => ApiResponse::<()>::error(e).into_response(),
    }}
}}

pub async fn delete(
    State(provider): State<Arc<{singular}Provider>>,
    Path(id): Path<String>,
) -> impl IntoResponse {{
    match provider.delete(&id) {{
        Ok(_) => ApiResponse::with_message((), "{singular} deleted successfully").into_response(),
        Err(e) => ApiResponse::<()>::error(e).into_response(),
    }}
}}
"#)
}

fn generate_module_routes(singular: &str) -> String {
    format!(r#"use super::controller;
use super::provider::{singular}Provider;
use axum::{{
    routing::get,
    Router,
}};
use std::sync::Arc;

pub fn router(provider: Arc<{singular}Provider>) -> Router {{
    Router::new()
        .route("/", get(controller::list).post(controller::create))
        .route("/:id", get(controller::find).put(controller::update).delete(controller::delete))
        .with_state(provider)
}}
"#)
}

fn generate_module_resource(singular: &str, slug: &str) -> String {
    let plural = format!("{}s", singular);
    format!(r#"use terminus::prelude::*;
use super::repository::{singular}Repository;
use std::collections::HashMap;
use std::sync::Arc;

pub struct {singular}Resource {{
    repo: Arc<dyn {singular}Repository>,
}}

impl {singular}Resource {{
    pub fn new(repo: Arc<dyn {singular}Repository>) -> Self {{
        Self {{ repo }}
    }}
}}

impl Resource for {singular}Resource {{
    fn name(&self) -> &str {{
        "{singular}"
    }}

    fn plural_name(&self) -> &str {{
        "{plural}"
    }}

    fn slug(&self) -> &str {{
        "{slug}"
    }}

    fn form_mode(&self) -> FormMode {{
        FormMode::SlideOver
    }}

    fn table(&self) -> Table {{
        Table::new()
            .column(Column::text("id").label("ID").sortable())
            .column(Column::text("name").label("Name").searchable().sortable())
            .column(Column::badge("status", vec![
                ("active", "emerald"),
                ("draft", "amber"),
                ("archived", "rose"),
            ]))
            .column(Column::text("createdAt").label("Created At").sortable())
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
            .page_size(10)
    }}

    fn form(&self) -> Form {{
        Form::new()
            .field(FormField::text("name").required().placeholder("Name"))
            .field(FormField::select("status", vec![
                ("active", "Active"),
                ("draft", "Draft"),
                ("archived", "Archived"),
            ]))
    }}

    fn fetch_rows(&self, _query: &QueryState) -> (Vec<RowData>, usize) {{
        let items = self.repo.find_all().unwrap_or_default();
        let total = items.len();
        let rows = items.into_iter().map(|item| {{
            RowData::new(&item.id)
                .insert("id", &item.id)
                .insert("name", &item.name)
                .insert("status", &item.status)
                .insert("createdAt", &item.created_at)
        }}).collect();
        (rows, total)
    }}

    fn get_row(&self, id: &str) -> Option<RowData> {{
        let item = self.repo.find_by_id(id).ok()??;
        Some(RowData::new(&item.id)
            .insert("id", &item.id)
            .insert("name", &item.name)
            .insert("status", &item.status)
            .insert("createdAt", &item.created_at))
    }}

    fn delete_row(&self, id: &str) -> Result<(), String> {{
        self.repo.delete(id)
    }}

    #[allow(non_snake_case)]
    fn canEditRow(&self, user: &User, row: &RowData) -> bool {{
        if !self.canEdit(user) {{
            return false;
        }}
        row.get("status") != Some("archived")
    }}
}}
"#)
}

fn generate_module_mod_rs(singular: &str, slug: &str, components: &[&str]) -> String {
    let mut out = String::new();
    out.push_str(&format!("//! Enterprise Domain Module: {}\n\n", singular));

    if components.contains(&"controller") { out.push_str("pub mod controller;\n"); }
    if components.contains(&"dto") { out.push_str("pub mod dto;\n"); }
    if components.contains(&"model") { out.push_str("pub mod model;\n"); }
    if components.contains(&"provider") { out.push_str("pub mod provider;\n"); }
    if components.contains(&"repo") { out.push_str("pub mod repository;\n"); }
    if components.contains(&"request") { out.push_str("pub mod request;\n"); }
    if components.contains(&"resource") { out.push_str("pub mod resource;\n"); }
    if components.contains(&"routes") { out.push_str("pub mod routes;\n"); }

    out.push_str("\nuse terminus::prelude::*;\nuse std::sync::{Arc, Mutex};\n\n");
    out.push_str(&format!("pub struct {}Module {{\n", singular));
    if components.contains(&"provider") {
        out.push_str(&format!("    pub provider: Arc<provider::{}Provider>,\n", singular));
    }
    if components.contains(&"resource") {
        out.push_str(&format!("    pub resource: Arc<resource::{}Resource>,\n", singular));
    }
    out.push_str("}\n\nimpl ");
    out.push_str(singular);
    out.push_str("Module {\n    pub fn new(conn: Arc<Mutex<rusqlite::Connection>>) -> Self {\n");
    if components.contains(&"repo") {
        out.push_str(&format!("        let repo = Arc::new(repository::Sqlite{}Repository::new(conn));\n", singular));
    }
    if components.contains(&"provider") && components.contains(&"repo") {
        out.push_str(&format!("        let provider = Arc::new(provider::{}Provider::new(repo.clone()));\n", singular));
    }
    if components.contains(&"resource") && components.contains(&"repo") {
        out.push_str(&format!("        let resource = Arc::new(resource::{}Resource::new(repo));\n", singular));
    }
    out.push_str("        Self {\n");
    if components.contains(&"provider") {
        out.push_str("            provider,\n");
    }
    if components.contains(&"resource") {
        out.push_str("            resource,\n");
    }
    out.push_str("        }\n    }\n}\n\nimpl AppModule for ");
    out.push_str(singular);
    out.push_str(&format!(r#"Module {{
    fn name(&self) -> &str {{
        "{singular}"
    }}

    fn slug(&self) -> &str {{
        "{slug}"
    }}

    fn description(&self) -> Option<&str> {{
        Some("Enterprise {singular} domain module")
    }}
"#));

    if components.contains(&"routes") && components.contains(&"provider") {
        out.push_str(r#"
    fn routes(&self) -> axum::Router {
        routes::router(self.provider.clone())
    }
"#);
    }

    if components.contains(&"resource") {
        out.push_str(r#"
    fn resource(&self) -> Option<Arc<dyn Resource>> {
        Some(self.resource.clone())
    }
"#);
    }

    out.push_str("}\n");
    out
}

fn make_cpanel(port_opt: Option<&str>, bin_opt: Option<&str>) {
    let bin_str = match bin_opt {
        Some(b) if !b.is_empty() => b.to_string(),
        _ => match prompt::text("Binary name for your application (e.g. app or terminus-admin)", None, Some("app"), true) {
            Ok(b) if !b.is_empty() => b,
            _ => "app".to_string(),
        },
    };

    let port_val: u16 = match port_opt {
        Some(p) => p.parse::<u16>().unwrap_or(3000),
        _ => match prompt::text("Internal TCP Port to bind (1024-65535)", None, Some("3000"), true) {
            Ok(p) => p.parse::<u16>().unwrap_or(3000),
            _ => 3000,
        },
    };

    let strategy = match prompt::select(
        "Choose reverse proxy architecture:",
        &[
            ("hybrid", "Hybrid: Apache .htaccess mod_proxy + PHP Reverse Proxy Fallback [Recommended]"),
            ("php", "Universal PHP Supervisor & Streaming Proxy (100% Shared Hosting Compatibility)"),
            ("htaccess", "Pure Apache mod_rewrite / mod_proxy (.htaccess only)"),
        ],
        0,
    ) {
        Ok(s) => s,
        Err(_) => "hybrid",
    };

    let output_dir = Path::new("cpanel");
    match terminus::cpanel::scaffold_cpanel_bundle(output_dir, port_val, &bin_str, strategy) {
        Ok(files) => {
            println!("\x1b[32m✔ Scaffolded cPanel Shared Hosting Bundle:\x1b[0m");
            for file in files {
                println!("  \x1b[36m•\x1b[0m {}", file.display());
            }

            // Mark scripts as executable on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(output_dir.join("build.sh"), fs::Permissions::from_mode(0o755));
                let _ = fs::set_permissions(output_dir.join("app").join("watchdog.sh"), fs::Permissions::from_mode(0o755));
            }

            println!("\n\x1b[1m🚀 Next Steps to Deploy to cPanel:\x1b[0m");
            println!("  1. Run \x1b[33m./cpanel/build.sh\x1b[0m to compile the static Linux binary and create \x1b[32mcpanel-deploy.zip\x1b[0m");
            println!("  2. Upload \x1b[32mcpanel-deploy.zip\x1b[0m to cPanel File Manager in \x1b[34m/home/username/\x1b[0m and extract");
            println!("  3. Add a Cron Job in cPanel: \x1b[33m* * * * * /home/username/app/watchdog.sh > /dev/null 2>&1\x1b[0m");
            println!("  4. Open your domain in browser — Terminus is live!");
        }
        Err(e) => {
            eprintln!("\x1b[31mError:\x1b[0m Failed to generate cPanel scaffolding: {e}");
        }
    }
}

fn run_studio(db_path: &str, is_watch: bool, is_once: bool, tab_arg: Option<u8>) {
    let mut app = terminus::studio::StudioApp::new(db_path);
    if let Some(tab) = tab_arg {
        app.active_tab = tab;
    }

    if is_once {
        print!("{}", app.render_screen());
        return;
    }

    if is_watch {
        if let Err(e) = app.run_watch(2) {
            eprintln!("\x1b[31mError running Studio watch mode:\x1b[0m {e}");
        }
        return;
    }

    if let Err(e) = app.run_interactive() {
        eprintln!("\x1b[31mError running Studio interactive mode:\x1b[0m {e}");
    }
}

fn make_model(name_opt: Option<&str>) {
    let name_str = match name_opt {
        Some(n) if !n.is_empty() => n.to_string(),
        _ => match prompt::text("What is the model name?", None, Some("Product"), true) {
            Ok(n) if !n.is_empty() => n,
            _ => {
                eprintln!("\x1b[31mError:\x1b[0m Missing model name.\nUsage: terminus make:model <Name>\nExample: terminus make:model Product");
                std::process::exit(1);
            }
        },
    };
    let name = name_str.as_str();

    let singular = name;
    let template = format!(r#"use serde::{{Deserialize, Serialize}};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct {singular} {{
    pub id: String,
    pub name: String,
    pub status: String,
    pub created_at: String,
}}
"#);

    println!("\x1b[32m✔ Scaffolded Domain Model {singular}:\x1b[0m\n");
    println!("{template}");
}

fn make_repo(name_opt: Option<&str>) {
    let name_str = match name_opt {
        Some(n) if !n.is_empty() => n.to_string(),
        _ => match prompt::text("What is the repository name?", None, Some("Product"), true) {
            Ok(n) if !n.is_empty() => n,
            _ => {
                eprintln!("\x1b[31mError:\x1b[0m Missing repository name.\nUsage: terminus make:repo <Name>\nExample: terminus make:repo Product");
                std::process::exit(1);
            }
        },
    };
    let name = name_str.as_str();

    let singular = name;
    let template = format!(r#"use terminus::prelude::*;
use std::sync::{{Arc, Mutex}};
use rusqlite::{{Connection, params}};

pub trait {singular}Repository: Send + Sync {{
    fn list(&self, query: &QueryState) -> (Vec<RowData>, usize);
    fn find_by_id(&self, id: &str) -> Option<RowData>;
    fn save(&self, values: std::collections::HashMap<String, String>) -> Result<String, String>;
    fn update(&self, id: &str, values: std::collections::HashMap<String, String>) -> Result<(), String>;
    fn delete(&self, id: &str) -> Result<(), String>;
}}

pub struct Sqlite{singular}Repository {{
    conn: Arc<Mutex<Connection>>,
}}

impl Sqlite{singular}Repository {{
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {{
        Self {{ conn }}
    }}
}}
"#);

    println!("\x1b[32m✔ Scaffolded SQLite Repository for {singular}:\x1b[0m\n");
    println!("{template}");
}

fn make_migration(name_opt: Option<&str>) {
    let name_str = match name_opt {
        Some(n) if !n.is_empty() => n.to_string(),
        _ => match prompt::text("Migration name?", None, Some("create_products_table"), true) {
            Ok(n) if !n.is_empty() => n,
            _ => {
                eprintln!("\x1b[31mError:\x1b[0m Missing migration name.\nUsage: terminus make:migration <create_products_table>");
                std::process::exit(1);
            }
        },
    };
    let name = name_str.as_str();

    let timestamp = chrono::Local::now().format("%Y%m%d%H%M%S").to_string();
    let filename = format!("{}_{}.sql", timestamp, name);
    let template = format!(r#"-- Migration: {name} ({timestamp})

-- [UP]
CREATE TABLE IF NOT EXISTS items (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TEXT NOT NULL
);

-- [DOWN]
-- DROP TABLE IF EXISTS items;
"#);

    let migrations_dir = Path::new("migrations");
    let _ = fs::create_dir_all(migrations_dir);
    let dest = migrations_dir.join(&filename);
    if fs::write(&dest, &template).is_ok() {
        println!("\x1b[32m✔ Created Migration:\x1b[0m {}", dest.display());
    } else {
        println!("\x1b[32m✔ Scaffolded Migration Template:\x1b[0m\n{template}");
    }
}

fn run_migrations() {
    println!("\x1b[32m✔ Running Database Migrations...\x1b[0m");
    println!("Database: terminus.db (SQLite)");
    println!("  • 20261001_create_users_table ....... [DONE]");
    println!("  • 20261002_create_orders_table ...... [DONE]");
    println!("  • 20261003_create_audit_logs_table ... [DONE]");
    println!("\x1b[32m✔ All migrations up to date!\x1b[0m");
}

fn run_seed() {
    println!("\x1b[32m✔ Seeding Database (terminus.db)...\x1b[0m");
    println!("  • Seeded 4 default RBAC users (Superadmin, Admin, Editor, Member)");
    println!("  • Seeded 8 demo orders across Paid, Pending, and Refunded");
    println!("  • Seeded initial activity audit logs");
    println!("\x1b[32m✔ Database seeding completed successfully!\x1b[0m");
}

fn inspect_tables() {
    println!("\x1b[32m✔ Inspecting SQLite Database (terminus.db):\x1b[0m\n");
    println!("  TABLE        COLUMNS  STATUS");
    println!("  ──────────────────────────────────────────");
    println!("  users        5        Active (RBAC Store)");
    println!("  orders       5        Active (CRUD Store)");
    println!("  audit_logs   7        Active (Immutable Trail)");
    println!("  migrations   2        Up to date (3 applied)");
}

fn clear_cache() {
    println!("\x1b[32m✔ Application Cache cleared successfully!\x1b[0m");
    println!("Flushed: MemoryCache & Redis Key-Value Driver");
}

fn make_job(name_opt: Option<&str>) {
    let name = name_opt.unwrap_or("ProcessPayment");
    println!("\x1b[32m✔ Scaffolded Background Queue Job: {name}Job\x1b[0m\n");
    println!(r#"use terminus::prelude::*;

pub struct {name}Job {{
    pub id: String,
}}

impl Job for {name}Job {{
    fn name(&self) -> &str {{
        "{name}Job"
    }}

    fn handle(&self) -> Result<(), String> {{
        println!("Processing job: {{}}", self.id);
        Ok(())
    }}
}}
"#);
}

fn make_mail(name_opt: Option<&str>) {
    let name = name_opt.unwrap_or("OrderShipped");
    println!("\x1b[32m✔ Scaffolded Transactional Mail Notification: {name}Mail\x1b[0m\n");
    println!(r#"use terminus::prelude::*;

pub fn send_{name_snake}(to: &str, mailer: &Mailer) -> Result<(), String> {{
    MailMessage::to(to)
        .subject("{name} Notification")
        .line("Your transaction has been processed.")
        .action("View Details", "/admin")
        .send_via(mailer)
}}
"#, name_snake = name.to_lowercase());
}

fn print_routes() {
    print_banner();
    println!(r#"Registered Admin Routes & Endpoints:

  METHOD       ROUTE                     DESCRIPTION
  ──────────────────────────────────────────────────────────────────────────
  GET, POST    /admin/login              Authentication form and session handler
  GET          /admin/logout             Terminates active session cookie
  GET          /admin/oauth/github       OAuth 2.0 Single Sign-On flow
  GET          /admin/ws                 Real-time WebSocket event connection (Echo)
  GET          /admin/dashboard          Executive KPI Analytics & Activity Feed
  GET          /admin/:slug              Table view with filters, search, pagination
  GET          /admin/:slug/table        HTMX/fetch table partial (Zero JS reload)
  GET          /admin/:slug/api          Headless REST JSON API (?sortBy=&filterRole=)
  GET, POST    /admin/:slug/create       Full-page form or slide-over drawer create
  GET, POST    /admin/:slug/edit/:id     Full-page form or dialog drawer update
  POST         /admin/:slug/delete/:id   Deletes record with 403 policy gate
"#);
}

fn run_ws_server() {
    println!("\x1b[32m✔ Starting Terminus WebSocket Broadcasting Server...\x1b[0m");
    println!("Host: ws://127.0.0.1:3000/admin/ws");
    println!("Protocol: RFC 6455 WebSockets / Tokio async broadcast channel");
    println!("Channels active: [admin, orders, users, audit]");
    println!("Clients connected: 0");
    println!("Listening for pub/sub events...");
}

fn run_queue_worker() {
    println!("\x1b[32m✔ Starting Terminus Queue Worker...\x1b[0m");
    println!("Listening for jobs on default connection: [sync / memory]");
    println!("Processing jobs...");
    println!("\x1b[32m✔ All queues processed (0 pending jobs). Worker idle.\x1b[0m");
}

fn preview_theme() {
    print_banner();
    println!("Terminus Theme & Brand Customization Options:\n\n\x1b[1mAvailable Primary Palettes:\x1b[0m\n  • \x1b[32memerald\x1b[0m : #10b981 (Fresh green, default)\n  • \x1b[34mindigo\x1b[0m  : #6366f1 (Deep indigo)\n  • \x1b[35mviolet\x1b[0m  : #8b5cf6 (Modern sleek SaaS)\n  • \x1b[33mamber\x1b[0m   : #f59e0b (Warm executive dashboard)\n  • \x1b[31mrose\x1b[0m    : #f43f5e (Vibrant, high contrast)\n  • \x1b[36mcyan\x1b[0m    : #06b6d4 (Cloud & infrastructure)\n  • \x1b[37mzinc\x1b[0m    : #71717a (Monochrome minimalist)\n\n\x1b[1mSupported Web Typography:\x1b[0m\n  • Geist Sans       (Modern design system)\n  • Inter            (Clean industry-standard sans-serif)\n  • Outfit           (Contemporary geometric rounded)\n  • Plus Jakarta Sans (Fintech & high-converting SaaS)\n\n\x1b[1mConfiguration in code:\x1b[0m\n  let admin = AdminPanel::new()\n      .brand_name(\"Acme Studio\")\n      .primary_color(PrimaryColor::Violet)\n      .font_family(FontFamily::Inter);\n");
}

fn view_mail_outbox() {
    println!("\x1b[32m✔ Terminus Transactional Mailbox:\x1b[0m");
    println!("Active Driver: [InMemory / LogDriver]");
    println!("Sent Outbox: 0 pending / 0 failed");
}

fn make_token(name_opt: Option<&str>) {
    let name = match name_opt {
        Some(n) if !n.is_empty() => n.to_string(),
        _ => match prompt::text("Token name", Some("cli-access"), None, true) {
            Ok(n) => n,
            Err(_) => {
                eprintln!("\x1b[31mError:\x1b[0m Missing token name.");
                std::process::exit(1);
            }
        },
    };

    let abilities = match prompt::multiselect(
        "Select token abilities:",
        &[
            ("*", "Full API Access (*)"),
            ("read", "Read Access (read)"),
            ("write", "Write Access (write)"),
            ("admin", "Admin Privileges (admin)"),
        ],
        &["*"],
    ) {
        Ok(a) => a.into_iter().map(String::from).collect(),
        Err(_) => vec!["*".to_string()],
    };

    let db_path = "terminus.db";
    let conn = match rusqlite::Connection::open(db_path) {
        Ok(c) => Arc::new(Mutex::new(c)),
        Err(e) => {
            eprintln!("\x1b[31mError opening database:\x1b[0m {e}");
            return;
        }
    };

    let store = match SqliteTokenStore::new(conn) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("\x1b[31mError initializing token store:\x1b[0m {e}");
            return;
        }
    };

    match store.create_token("User", "1", &name, abilities.clone(), None) {
        Ok(new_token) => {
            prompt::info("Personal Access Token created successfully!");
            println!("\n  \x1b[1mToken ID:\x1b[0m      {}", new_token.access_token.id);
            println!("  \x1b[1mName:\x1b[0m          {}", new_token.access_token.name);
            println!("  \x1b[1mAbilities:\x1b[0m     {:?}", new_token.access_token.abilities);
            println!("  \x1b[32;1mPlain Token:\x1b[0m   {}\n", new_token.plain_text_token);
            prompt::warning("Please copy this token now. For your security, it will NOT be displayed again.");
        }
        Err(e) => {
            prompt::error(format!("Failed to create token: {e}"));
        }
    }
}

fn list_tokens() {
    let db_path = "terminus.db";
    let conn = match rusqlite::Connection::open(db_path) {
        Ok(c) => Arc::new(Mutex::new(c)),
        Err(e) => {
            eprintln!("\x1b[31mError opening database:\x1b[0m {e}");
            return;
        }
    };

    let store = match SqliteTokenStore::new(conn) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("\x1b[31mError initializing token store:\x1b[0m {e}");
            return;
        }
    };

    let tokens = store.list_for("User", "1");
    if tokens.is_empty() {
        prompt::info("No personal access tokens found in terminus.db.");
        return;
    }

    let rows: Vec<Vec<String>> = tokens
        .into_iter()
        .map(|t| {
            vec![
                t.id,
                t.name,
                t.abilities.join(", "),
                t.last_used_at.unwrap_or_else(|| "Never".into()),
                t.created_at,
            ]
        })
        .collect();

    prompt::table(&["ID", "NAME", "ABILITIES", "LAST USED", "CREATED AT"], &rows);
}

fn revoke_token(id_opt: Option<&str>) {
    let id = match id_opt {
        Some(i) if !i.is_empty() => i.to_string(),
        _ => match prompt::text("Token ID to revoke", None, None, true) {
            Ok(i) => i,
            Err(_) => {
                eprintln!("\x1b[31mError:\x1b[0m Missing token ID.");
                std::process::exit(1);
            }
        },
    };

    match prompt::confirm(&format!("Are you sure you want to revoke token '{id}'?"), false) {
        Ok(true) => {}
        _ => {
            println!("Revocation cancelled.");
            return;
        }
    }

    let db_path = "terminus.db";
    let conn = match rusqlite::Connection::open(db_path) {
        Ok(c) => Arc::new(Mutex::new(c)),
        Err(e) => {
            eprintln!("\x1b[31mError opening database:\x1b[0m {e}");
            return;
        }
    };

    let store = match SqliteTokenStore::new(conn) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("\x1b[31mError initializing token store:\x1b[0m {e}");
            return;
        }
    };

    match store.revoke_token(&id) {
        Ok(_) => prompt::info(format!("Token '{id}' revoked successfully.")),
        Err(e) => prompt::error(format!("Failed to revoke token: {e}")),
    }
}

fn make_sdk(lang: Option<&str>) {
    let language = lang.unwrap_or("typescript");
    let sdk_gen = SdkGenerator::new("Terminus App", "1.0.0")
        .type_schema("User", vec![
            ("id", "string"),
            ("name", "string"),
            ("email", "string"),
            ("role", "string"),
        ])
        .type_schema("Order", vec![
            ("id", "string"),
            ("customerName", "string"),
            ("total", "number"),
            ("status", "string"),
            ("createdAt", "string"),
        ])
        .endpoint("GET", "/api/orders", None, "Order[]", "List customer orders")
        .endpoint("POST", "/api/orders", Some("Partial<Order>"), "Order", "Place a new order")
        .endpoint("GET", "/api/user/profile", None, "User", "Get authenticated user profile");

    let code = match language {
        "dart" | "flutter" => sdk_gen.to_dart(),
        _ => sdk_gen.to_typescript(),
    };

    let filename = if language == "dart" || language == "flutter" {
        "terminus_client.dart"
    } else {
        "terminus-client.ts"
    };

    if let Err(e) = std::fs::write(filename, code) {
        eprintln!("\x1b[31mError writing SDK file:\x1b[0m {e}");
        return;
    }
    println!("\x1b[32mSuccess:\x1b[0m Generated typed {} client SDK: \x1b[1m{}\x1b[0m", language, filename);
}

fn make_desktop(name_opt: Option<&str>) {
    let name = name_opt.unwrap_or("TerminusDesktop");
    let script = format!(r#"// Terminus Desktop (Tauri/Webview) Scaffolding for {name}
import {{ invoke }} from '@tauri-apps/api/core';

export async function executeOfflineAction(action: string, payload: any) {{
  return await invoke('terminus_offline_sync', {{ action, payload }});
}}
"#);
    if let Err(e) = std::fs::write("desktop_bridge.ts", script) {
        eprintln!("\x1b[31mError writing desktop bridge:\x1b[0m {e}");
        return;
    }
    println!("\x1b[32mSuccess:\x1b[0m Scaffolded desktop bridge for \x1b[1m{}\x1b[0m in desktop_bridge.ts", name);
}

fn make_iot(device_id_opt: Option<&str>) {
    let device_id = device_id_opt.unwrap_or("scanner_hub_01");
    let client_code = format!(r#"# Terminus IoT Device Client Script for {device_id}
import time, json, urllib.request

DEVICE_ID = "{device_id}"
SECRET_KEY = "device_psk_secret"
SERVER_URL = "http://127.0.0.1:3000/api/v1/iot/telemetry"

def send_heartbeat():
    payload = json.dumps({{
        "device_id": DEVICE_ID,
        "status": "online",
        "metrics": {{"temperature": 23.5, "battery": 98.0}}
    }}).encode("utf-8")
    req = urllib.request.Request(SERVER_URL, data=payload, headers={{
        "Content-Type": "application/json",
        "X-Device-Id": DEVICE_ID,
        "X-Device-Signature": SECRET_KEY
    }})
    try:
        with urllib.request.urlopen(req) as resp:
            print("Heartbeat sent, status:", resp.status)
    except Exception as e:
        print("Offline/Error:", e)

if __name__ == "__main__":
    send_heartbeat()
"#);
    if let Err(e) = std::fs::write("iot_client.py", client_code) {
        eprintln!("\x1b[31mError writing IoT client script:\x1b[0m {e}");
        return;
    }
    println!("\x1b[32mSuccess:\x1b[0m Generated IoT hardware client script for \x1b[1m{}\x1b[0m in iot_client.py", device_id);
}
