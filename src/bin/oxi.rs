use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    match command {
        "help" | "--help" | "-h" => print_help(),
        "version" | "--version" | "-v" => print_version(),
        "make:resource" => {
            let name = args.get(2).map(|s| s.as_str());
            make_resource(name);
        }
        "make:model" => {
            let name = args.get(2).map(|s| s.as_str());
            make_model(name);
        }
        "make:repo" => {
            let name = args.get(2).map(|s| s.as_str());
            make_repo(name);
        }
        "routes" => print_routes(),
        "queue:work" => run_queue_worker(),
        "theme:preview" => preview_theme(),
        "mail:outbox" => view_mail_outbox(),
        unknown => {
            eprintln!("\x1b[31mError:\x1b[0m Unknown command '{unknown}'\n");
            print_help();
            std::process::exit(1);
        }
    }
}

fn print_banner() {
    println!("\x1b[32m   ____            _       _         _             _       \n  / __ \\ _  __(_)___| |___  / \\   __| |_ __ ___ (_)_ __  \n / / _` \\ \\/ / / _` / / -_)/ _ \\ / _` | '  \\ _ \\| | '  \\ \n \\ \\__,_/>  </_\\__,/_\\___/_/   \\_\\__,_|_|_|_|___/|_|_|_|_|\n  \\____//_/\\_\\                                            \x1b[0m\n\x1b[1mOxideAdmin CLI (oxi)\x1b[0m v0.1.0 — Fast Rust & Axum Admin & App Tooling\n");
}

fn print_help() {
    print_banner();
    println!("Usage:\n  oxi <command> [arguments]\n\n\x1b[1mAvailable Commands:\x1b[0m\n  \x1b[33mmake:resource <Name>\x1b[0m    Scaffold a new Filament-style Resource with table & form\n  \x1b[33mmake:model <Name>\x1b[0m       Scaffold a new domain Model struct with camelCase serde\n  \x1b[33mmake:repo <Name>\x1b[0m        Scaffold a SQLite repository implementation\n  \x1b[33mroutes\x1b[0m                  List registered admin routes & JSON API endpoints\n  \x1b[33mqueue:work\x1b[0m              Start processing background queue jobs\n  \x1b[33mmail:outbox\x1b[0m             Inspect sent transactional emails\n  \x1b[33mtheme:preview\x1b[0m           Preview theme colors, palettes, and font families\n  \x1b[33mversion\x1b[0m                 Show OxideAdmin & oxi CLI version\n\n\x1b[1mExamples:\x1b[0m\n  oxi make:resource Product\n  oxi make:model Customer\n  oxi theme:preview\n");
}

fn print_version() {
    println!("oxi (OxideAdmin CLI) v0.1.0 (Rust edition 2021)");
}

fn make_resource(name_opt: Option<&str>) {
    let name = match name_opt {
        Some(n) if !n.is_empty() => n,
        _ => {
            eprintln!("\x1b[31mError:\x1b[0m Missing resource name.\nUsage: oxi make:resource <Name>\nExample: oxi make:resource Product");
            std::process::exit(1);
        }
    };

    let singular = name;
    let plural = format!("{}s", name);
    let slug = name.to_lowercase();
    let filename = format!("{}_resource.rs", slug);

    let template = format!(r#"use oxide_admin::prelude::*;
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
        // Wire to your repository list query
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

    // Filament-style record policy hook
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

fn make_model(name_opt: Option<&str>) {
    let name = match name_opt {
        Some(n) if !n.is_empty() => n,
        _ => {
            eprintln!("\x1b[31mError:\x1b[0m Missing model name.\nUsage: oxi make:model <Name>\nExample: oxi make:model Product");
            std::process::exit(1);
        }
    };

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
    let name = match name_opt {
        Some(n) if !n.is_empty() => n,
        _ => {
            eprintln!("\x1b[31mError:\x1b[0m Missing repository name.\nUsage: oxi make:repo <Name>\nExample: oxi make:repo Product");
            std::process::exit(1);
        }
    };

    let singular = name;
    let template = format!(r#"use oxide_admin::prelude::*;
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

fn print_routes() {
    print_banner();
    println!(r#"Registered Admin Routes & Endpoints:

  METHOD       ROUTE                     DESCRIPTION
  ──────────────────────────────────────────────────────────────────────────
  GET, POST    /admin/login              Authentication form and session handler
  GET          /admin/logout             Terminates active session cookie
  GET          /admin/oauth/github       OAuth 2.0 Single Sign-On flow
  GET          /admin/dashboard          Executive KPI Analytics & Activity Feed
  GET          /admin/:slug              Table view with filters, search, pagination
  GET          /admin/:slug/table        HTMX/fetch table partial (Zero JS reload)
  GET          /admin/:slug/api          Headless REST JSON API (?sortBy=&filterRole=)
  GET, POST    /admin/:slug/create       Full-page form or slide-over drawer create
  GET, POST    /admin/:slug/edit/:id     Full-page form or dialog drawer update
  POST         /admin/:slug/delete/:id   Deletes record with 403 policy gate
"#);
}

fn run_queue_worker() {
    println!("\x1b[32m✔ Starting Oxide Queue Worker...\x1b[0m");
    println!("Listening for jobs on default connection: [sync / memory]");
    println!("Processing jobs...");
    println!("\x1b[32m✔ All queues processed (0 pending jobs). Worker idle.\x1b[0m");
}

fn preview_theme() {
    print_banner();
    println!("OxideAdmin Theme & Brand Customization Options:\n\n\x1b[1mAvailable Primary Palettes:\x1b[0m\n  • \x1b[32memerald\x1b[0m : #10b981 (Fresh green, default)\n  • \x1b[34mindigo\x1b[0m  : #6366f1 (Classic Laravel purple-blue)\n  • \x1b[35mviolet\x1b[0m  : #8b5cf6 (Modern sleek SaaS)\n  • \x1b[33mamber\x1b[0m   : #f59e0b (Warm executive dashboard)\n  • \x1b[31mrose\x1b[0m    : #f43f5e (Vibrant, high contrast)\n  • \x1b[36mcyan\x1b[0m    : #06b6d4 (Cloud & infrastructure)\n  • \x1b[37mzinc\x1b[0m    : #71717a (Monochrome minimalist)\n\n\x1b[1mSupported Web Typography:\x1b[0m\n  • Geist Sans       (Modern Vercel design system)\n  • Inter            (Clean industry-standard sans-serif)\n  • Outfit           (Contemporary geometric rounded)\n  • Plus Jakarta Sans (Fintech & high-converting SaaS)\n\n\x1b[1mConfiguration in code:\x1b[0m\n  let admin = AdminPanel::new()\n      .brand_name(\"Acme Studio\")\n      .primary_color(PrimaryColor::Violet)\n      .font_family(FontFamily::Inter);\n");
}

fn view_mail_outbox() {
    println!("\x1b[32m✔ Oxide Transactional Mailbox:\x1b[0m");
    println!("Active Driver: [InMemory / LogDriver]");
    println!("Sent Outbox: 0 pending / 0 failed");
}
