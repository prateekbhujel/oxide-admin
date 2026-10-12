//! Terminus Project Scaffolder & Installation Wizard
//!
//! Interactive project bootstrapping and template scaffolding
//! supporting Fullstack (with React/Vue/HTMX), Backend API (headless),
//! and Universal (Terminus Web, Mobile, Desktop & IoT) architecture presets.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Architecture presets available during project creation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectPreset {
    /// Universal Terminus ecosystem: Web Admin UI, Mobile APIs, Desktop SQLite sync, IoT scanner streams
    Universal,
    /// Fullstack: Backend API + Built-in Admin Dashboard UI
    Fullstack,
    /// Headless Backend API: Pure high-throughput JSON API, OpenAPI 3.0 specs, zero frontend bloat
    Backend,
}

impl ProjectPreset {
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "universal" | "all" | "u" => Some(Self::Universal),
            "fullstack" | "admin" | "dashboard" | "f" => Some(Self::Fullstack),
            "backend" | "api" | "headless" | "b" => Some(Self::Backend),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Universal => "universal",
            Self::Fullstack => "fullstack",
            Self::Backend => "backend",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Universal => "Universal (Terminus Ecosystem: Web, Mobile APIs, Desktop Sync, IoT)",
            Self::Fullstack => "Fullstack (Backend API + Built-in Admin Dashboard)",
            Self::Backend => "Backend API (High-throughput JSON API + OpenAPI 3.0 Specs)",
        }
    }
}

/// Frontend client choices
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrontendChoice {
    /// Headless API only (no frontend scaffolding)
    None,
    /// React 19 + TypeScript (Vite & Tailwind CSS)
    React,
    /// Vue 3 + TypeScript (Vite & Tailwind CSS)
    Vue,
    /// HTML5 + HTMX + Tailwind CSS (Zero JavaScript build step, SSR)
    Htmx,
}

impl FrontendChoice {
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "none" | "no" | "n" | "api" => Some(Self::None),
            "react" | "r" | "react-ts" => Some(Self::React),
            "vue" | "v" | "vue-ts" => Some(Self::Vue),
            "htmx" | "h" | "html" => Some(Self::Htmx),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::React => "react",
            Self::Vue => "vue",
            Self::Htmx => "htmx",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::None => "None (Headless API only)",
            Self::React => "React 19 + TypeScript (Vite & Tailwind CSS)",
            Self::Vue => "Vue 3 + TypeScript (Vite & Tailwind CSS)",
            Self::Htmx => "HTMX + Tailwind CSS (Zero build step, SSR)",
        }
    }
}

/// Database engine choices
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseChoice {
    /// SQLite (Embedded, zero configuration single-file database)
    Sqlite,
    /// PostgreSQL (Enterprise relational database)
    Postgres,
}

impl DatabaseChoice {
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "sqlite" | "sqlite3" | "sq" => Some(Self::Sqlite),
            "postgres" | "postgresql" | "pg" => Some(Self::Postgres),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sqlite => "sqlite",
            Self::Postgres => "postgres",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Sqlite => "SQLite (Zero configuration single-file database)",
            Self::Postgres => "PostgreSQL (Enterprise relational database)",
        }
    }
}

/// Configuration options for a new Terminus project
#[derive(Debug, Clone)]
pub struct ProjectConfig {
    pub name: String,
    pub preset: ProjectPreset,
    pub frontend: FrontendChoice,
    pub database: DatabaseChoice,
}

impl ProjectConfig {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            preset: ProjectPreset::Universal,
            frontend: FrontendChoice::None,
            database: DatabaseChoice::Sqlite,
        }
    }

    pub fn preset(mut self, preset: ProjectPreset) -> Self {
        self.preset = preset;
        self
    }

    pub fn frontend(mut self, frontend: FrontendChoice) -> Self {
        self.frontend = frontend;
        self
    }

    pub fn database(mut self, database: DatabaseChoice) -> Self {
        self.database = database;
        self
    }
}

/// Scaffolds a complete Terminus project into the specified directory
pub fn scaffold_project(target_dir: &Path, config: &ProjectConfig) -> io::Result<Vec<PathBuf>> {
    let mut created_files = Vec::new();

    fs::create_dir_all(target_dir)?;

    // 1. Cargo.toml
    let cargo_toml = generate_cargo_toml(config);
    let cargo_path = target_dir.join("Cargo.toml");
    fs::write(&cargo_path, cargo_toml)?;
    created_files.push(cargo_path);

    // 2. src/main.rs
    let src_dir = target_dir.join("src");
    fs::create_dir_all(&src_dir)?;
    let main_rs = generate_main_rs(config);
    let main_path = src_dir.join("main.rs");
    fs::write(&main_path, main_rs)?;
    created_files.push(main_path);

    // 3. .env.example
    let env_example = generate_env_example(config);
    let env_path = target_dir.join(".env.example");
    fs::write(&env_path, env_example)?;
    created_files.push(env_path);

    // 4. README.md
    let readme = generate_readme(config);
    let readme_path = target_dir.join("README.md");
    fs::write(&readme_path, readme)?;
    created_files.push(readme_path);

    // 5. Frontend scaffolding (if selected)
    match config.frontend {
        FrontendChoice::React => {
            let fe_files = scaffold_react_frontend(&target_dir.join("frontend"), config)?;
            created_files.extend(fe_files);
        }
        FrontendChoice::Vue => {
            let fe_files = scaffold_vue_frontend(&target_dir.join("frontend"), config)?;
            created_files.extend(fe_files);
        }
        FrontendChoice::Htmx => {
            let public_dir = target_dir.join("public");
            fs::create_dir_all(&public_dir)?;
            let htmx_html = generate_htmx_index(config);
            let htmx_path = public_dir.join("index.html");
            fs::write(&htmx_path, htmx_html)?;
            created_files.push(htmx_path);
        }
        FrontendChoice::None => {}
    }

    Ok(created_files)
}

/// Generates Cargo.toml for the new application
pub fn generate_cargo_toml(config: &ProjectConfig) -> String {
    format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[dependencies]
terminus-framework = "0.1"
tokio = {{ version = "1", features = ["full"] }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
axum = {{ version = "0.7", features = ["macros"] }}
chrono = "0.4"
"#,
        name = config.name
    )
}

/// Generates src/main.rs tailored to the chosen preset and frontend
pub fn generate_main_rs(config: &ProjectConfig) -> String {
    let app_name = &config.name;

    match config.preset {
        ProjectPreset::Universal => {
            let tmpl = r#"use terminus::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize thread-safe database connection
    let db = Database::open("{{NAME}}.db")?;
    db.execute(
        "CREATE TABLE IF NOT EXISTS items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'active',
            created_at TEXT NOT NULL
        )",
        [],
    )?;

    // 2. Multi-platform SDK & OpenAPI specification generator
    let sdk = SdkGenerator::new("{{NAME}}")
        .base_url("http://localhost:3000")
        .endpoint(SdkEndpoint::get("getItems", "/api/v1/items", "Fetch active inventory items"))
        .endpoint(SdkEndpoint::post("createItem", "/api/v1/items", "Create a new inventory item"));

    println!("⚡ Terminus Engine [{{NAME}}] ready across 4 touchpoints:");
    println!("  -> Web Admin UI   : http://localhost:3000/admin");
    println!("  -> OpenAPI Specs  : http://localhost:3000/api/openapi.json");
    println!("  -> Mobile SDK API : http://localhost:3000/api/v1");
    println!("  -> Terminal TUI   : run `cargo terminus studio` to inspect telemetry");

    // 3. Bootstrap fluent, zero-boilerplate application engine
    App::new()
        .port(3000)
        .database(db)
        .openapi("/api/openapi.json", sdk)
        .get("/", || async {
            json_response(serde_json::json!({
                "app": "{{NAME}}",
                "status": "online",
                "preset": "universal",
                "docs": "/api/openapi.json"
            }))
        })
        .group("/api/v1", |api| {
            api.get("/items", || async {
                json_response(serde_json::json!([
                    { "id": 1, "title": "Universal Gateway", "status": "active" },
                    { "id": 2, "title": "Mobile Telemetry Node", "status": "active" }
                ]))
            })
            .post("/items", || async {
                json_response(serde_json::json!({ "created": true, "id": 3 }))
            })
        })
        .run()
        .await?;

    Ok(())
}
"#;
            tmpl.replace("{{NAME}}", app_name)
        }

        ProjectPreset::Fullstack => {
            let tmpl = r#"use terminus::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::open("{{NAME}}.db")?;

    println!("⚡ Terminus Fullstack [{{NAME}}] starting...");
    println!("  -> Web Application : http://localhost:3000");
    println!("  -> API Endpoints   : http://localhost:3000/api/v1");

    App::new()
        .port(3000)
        .database(db)
        .static_dir("/assets", "public")
        .get("/", || async {
            html_response("<!DOCTYPE html><html><head><title>{{NAME}}</title><script src=\"https://cdn.tailwindcss.com\"></script></head><body class=\"bg-slate-950 text-slate-100 flex min-h-screen items-center justify-center p-6\"><div class=\"max-w-md w-full bg-slate-900 border border-slate-800 rounded-xl p-8 text-center\"><h1 class=\"text-2xl font-bold text-white mb-2\">{{NAME}}</h1><p class=\"text-slate-400 text-sm mb-6\">Fullstack application powered by Rust & Axum.</p><div class=\"bg-slate-950 p-3 rounded border border-slate-800 text-xs font-mono text-emerald-400\">Connected to Terminus</div></div></body></html>")
        })
        .group("/api/v1", |api| {
            api.get("/health", || async {
                json_response(serde_json::json!({ "status": "healthy", "service": "{{NAME}}" }))
            })
        })
        .run()
        .await?;

    Ok(())
}
"#;
            tmpl.replace("{{NAME}}", app_name)
        }

        ProjectPreset::Backend => {
            let tmpl = r#"use terminus::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::open("{{NAME}}.db")?;

    let sdk = SdkGenerator::new("{{NAME}}")
        .base_url("http://localhost:3000")
        .endpoint(SdkEndpoint::get("getHealth", "/api/v1/health", "Service health check"))
        .endpoint(SdkEndpoint::get("getRecords", "/api/v1/records", "List domain records"));

    println!("⚡ Terminus Headless API [{{NAME}}] running on http://localhost:3000");
    println!("  -> OpenAPI Documentation : http://localhost:3000/api/openapi.json");

    App::new()
        .port(3000)
        .database(db)
        .openapi("/api/openapi.json", sdk)
        .group("/api/v1", |api| {
            api.get("/health", || async {
                json_response(serde_json::json!({
                    "service": "{{NAME}}",
                    "status": "ok",
                    "timestamp": now()
                }))
            })
            .get("/records", || async {
                json_response(serde_json::json!([
                    { "id": 1, "name": "Primary Database Node", "latency_ms": 0.4 },
                    { "id": 2, "name": "Telemetry Ingestion Worker", "latency_ms": 0.8 }
                ]))
            })
        })
        .run()
        .await?;

    Ok(())
}
"#;
            tmpl.replace("{{NAME}}", app_name)
        }
    }
}

/// Generates .env.example
pub fn generate_env_example(config: &ProjectConfig) -> String {
    format!(
        r#"# {name} Environment Configuration
APP_NAME="{name}"
APP_ENV=local
PORT=3000

# Database
DATABASE_URL={db_url}

# Authentication & Security
APP_KEY=terminus_secret_key_change_in_production
"#,
        name = config.name,
        db_url = match config.database {
            DatabaseChoice::Sqlite => format!("{}.db", config.name),
            DatabaseChoice::Postgres => "postgres://postgres:postgres@localhost:5432/terminus_db".to_string(),
        }
    )
}

/// Generates README.md for the new project
pub fn generate_readme(config: &ProjectConfig) -> String {
    let fe_instructions = match config.frontend {
        FrontendChoice::React | FrontendChoice::Vue => format!(
            r#"
### Frontend Development ({fe})

1. Navigate to the frontend directory:
   ```bash
   cd frontend
   ```
2. Install dependencies:
   ```bash
   npm install
   ```
3. Start Vite dev server:
   ```bash
   npm run dev
   ```
   The frontend automatically proxies `/api/*` requests to the Terminus backend on port 3000.
"#,
            fe = config.frontend.display_name()
        ),
        FrontendChoice::Htmx => r#"
### Frontend (HTMX)
Zero build steps required! Static HTMX assets are served directly from `./public/index.html`.
"#.to_string(),
        FrontendChoice::None => "".to_string(),
    };

    format!(
        r#"# {name}

Powered by **Terminus Framework** — The universal fullstack application framework for Rust & Axum.

- **Architecture Preset**: `{preset}`
- **Frontend Choice**: `{frontend}`
- **Database Engine**: `{database}`

---

## Getting Started

### 1. Run the Backend Application

```bash
cargo run
```
The server will start listening at `http://localhost:3000`.

### 2. Launch the Terminus Studio TUI

Terminus includes an interactive terminal UI dashboard to inspect your database, queues, cache, and telemetry:

```bash
cargo terminus studio
```
{fe_instructions}
## Project Structure

```
├── Cargo.toml          # Rust dependencies & package configuration
├── src/
│   └── main.rs         # Terminus application bootstrap & routing
├── .env.example        # Environment variables template
{fe_tree}└── README.md
```
"#,
        name = config.name,
        preset = config.preset.as_str(),
        frontend = config.frontend.as_str(),
        database = config.database.as_str(),
        fe_instructions = fe_instructions,
        fe_tree = if matches!(config.frontend, FrontendChoice::React | FrontendChoice::Vue) {
            "├── frontend/           # Modern TypeScript client (Vite + Tailwind)\n"
        } else if matches!(config.frontend, FrontendChoice::Htmx) {
            "├── public/             # Static HTML & HTMX assets\n"
        } else {
            ""
        }
    )
}

/// Generates HTML5 + HTMX template
pub fn generate_htmx_index(config: &ProjectConfig) -> String {
    let tmpl = r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>{{NAME}} — Terminus + HTMX</title>
  <script src="https://cdn.tailwindcss.com"></script>
  <script src="https://unpkg.com/htmx.org@1.9.12"></script>
</head>
<body class="bg-slate-950 text-slate-100 flex min-h-screen flex-col items-center justify-center p-6">
  <div class="max-w-lg w-full bg-slate-900 border border-slate-800 rounded-xl p-8 shadow-2xl">
    <div class="flex items-center justify-between mb-4">
      <h1 class="text-2xl font-bold text-white">{{NAME}}</h1>
      <span class="px-2.5 py-0.5 rounded-full text-xs font-semibold bg-emerald-950 text-emerald-400 border border-emerald-800">
        HTMX Active
      </span>
    </div>
    <p class="text-slate-400 text-sm mb-6">Zero-build interactive frontend powered by Terminus & Axum.</p>
    <button hx-get="/api/v1/items" hx-target="#results" hx-swap="innerHTML"
      class="w-full bg-sky-600 hover:bg-sky-500 text-white font-medium py-2.5 px-4 rounded-lg transition-colors">
      Load Items from Terminus Backend
    </button>
    <div id="results" class="mt-4 text-xs font-mono text-slate-300"></div>
  </div>
</body>
</html>"##;
    tmpl.replace("{{NAME}}", &config.name)
}

/// Scaffolds a React 19 + TypeScript + Vite + Tailwind frontend
fn scaffold_react_frontend(dest_dir: &Path, config: &ProjectConfig) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    fs::create_dir_all(dest_dir.join("src"))?;

    // package.json
    let pkg_json = format!(
        r#"{{
  "name": "{name}-frontend",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {{
    "dev": "vite",
    "build": "tsc -b && vite build",
    "preview": "vite preview"
  }},
  "dependencies": {{
    "react": "^19.0.0",
    "react-dom": "^19.0.0"
  }},
  "devDependencies": {{
    "@types/react": "^19.0.0",
    "@types/react-dom": "^19.0.0",
    "@vitejs/plugin-react": "^4.3.0",
    "typescript": "^5.6.0",
    "vite": "^6.0.0",
    "tailwindcss": "^3.4.0",
    "autoprefixer": "^10.4.0",
    "postcss": "^8.4.0"
  }}
}}
"#,
        name = config.name
    );
    let p_path = dest_dir.join("package.json");
    fs::write(&p_path, pkg_json)?;
    files.push(p_path);

    // vite.config.ts with API proxy to port 3000
    let vite_config = r#"import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:3000',
        changeOrigin: true,
      },
    },
  },
});
"#;
    let v_path = dest_dir.join("vite.config.ts");
    fs::write(&v_path, vite_config)?;
    files.push(v_path);

    // tsconfig.json
    let tsconfig = r#"{
  "compilerOptions": {
    "target": "ES2022",
    "useDefineForClassFields": true,
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true
  },
  "include": ["src"]
}
"#;
    let ts_path = dest_dir.join("tsconfig.json");
    fs::write(&ts_path, tsconfig)?;
    files.push(ts_path);

    // index.html
    let index_html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>{name} — Terminus + React</title>
  </head>
  <body class="bg-slate-950 text-slate-100 antialiased">
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
"#,
        name = config.name
    );
    let idx_path = dest_dir.join("index.html");
    fs::write(&idx_path, index_html)?;
    files.push(idx_path);

    // src/main.tsx
    let main_tsx = r#"import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App.tsx';

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
"#;
    let m_path = dest_dir.join("src/main.tsx");
    fs::write(&m_path, main_tsx)?;
    files.push(m_path);

    // src/App.tsx
    let app_tsx = format!(
        r#"import {{ useState, useEffect }} from 'react';

export default function App() {{
  const [data, setData] = useState<any>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {{
    fetch('/api/v1/items')
      .then((res) => res.json())
      .then((json) => {{
        setData(json);
        setLoading(false);
      }})
      .catch((err) => {{
        console.error(err);
        setLoading(false);
      }});
  }}, []);

  return (
    <div className="flex min-h-screen flex-col items-center justify-center p-6">
      <div className="w-full max-w-lg rounded-2xl border border-slate-800 bg-slate-900 p-8 shadow-2xl">
        <div className="mb-4 flex items-center justify-between">
          <h1 className="text-2xl font-bold tracking-tight text-white">{name}</h1>
          <span className="rounded-full border border-sky-800 bg-sky-950 px-3 py-1 text-xs font-medium text-sky-400">
            React 19 + TypeScript
          </span>
        </div>
        <p className="mb-6 text-sm text-slate-400">
          Connected to Terminus Axum backend on port 3000.
        </p>

        <div className="rounded-xl border border-slate-800 bg-slate-950 p-4">
          <p className="mb-2 text-xs font-semibold uppercase tracking-wider text-slate-500">
            API Response (/api/v1/items)
          </p>
          {{loading ? (
            <p className="text-sm text-slate-500">Fetching live data...</p>
          ) : (
            <pre className="overflow-x-auto text-xs text-emerald-400 font-mono">
              {{JSON.stringify(data, null, 2)}}
            </pre>
          )}}
        </div>
      </div>
    </div>
  );
}}
"#,
        name = config.name
    );
    let app_path = dest_dir.join("src/App.tsx");
    fs::write(&app_path, app_tsx)?;
    files.push(app_path);

    Ok(files)
}

/// Scaffolds a Vue 3 + TypeScript + Vite + Tailwind frontend
fn scaffold_vue_frontend(dest_dir: &Path, config: &ProjectConfig) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    fs::create_dir_all(dest_dir.join("src"))?;

    // package.json
    let pkg_json = format!(
        r#"{{
  "name": "{name}-frontend",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {{
    "dev": "vite",
    "build": "vue-tsc && vite build",
    "preview": "vite preview"
  }},
  "dependencies": {{
    "vue": "^3.5.0"
  }},
  "devDependencies": {{
    "@vitejs/plugin-vue": "^5.2.0",
    "typescript": "^5.6.0",
    "vite": "^6.0.0",
    "vue-tsc": "^2.1.0",
    "tailwindcss": "^3.4.0",
    "autoprefixer": "^10.4.0",
    "postcss": "^8.4.0"
  }}
}}
"#,
        name = config.name
    );
    let p_path = dest_dir.join("package.json");
    fs::write(&p_path, pkg_json)?;
    files.push(p_path);

    // vite.config.ts with API proxy to port 3000
    let vite_config = r#"import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';

export default defineConfig({
  plugins: [vue()],
  server: {
    port: 5173,
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:3000',
        changeOrigin: true,
      },
    },
  },
});
"#;
    let v_path = dest_dir.join("vite.config.ts");
    fs::write(&v_path, vite_config)?;
    files.push(v_path);

    // tsconfig.json
    let tsconfig = r#"{
  "compilerOptions": {
    "target": "ES2022",
    "useDefineForClassFields": true,
    "module": "ESNext",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "preserve",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true
  },
  "include": ["src/**/*.ts", "src/**/*.d.ts", "src/**/*.tsx", "src/**/*.vue"]
}
"#;
    let ts_path = dest_dir.join("tsconfig.json");
    fs::write(&ts_path, tsconfig)?;
    files.push(ts_path);

    // index.html
    let index_html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>{name} — Terminus + Vue</title>
  </head>
  <body class="bg-slate-950 text-slate-100 antialiased">
    <div id="app"></div>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
"#,
        name = config.name
    );
    let idx_path = dest_dir.join("index.html");
    fs::write(&idx_path, index_html)?;
    files.push(idx_path);

    // src/main.ts
    let main_ts = r#"import { createApp } from 'vue';
import App from './App.vue';

createApp(App).mount('#app');
"#;
    let m_path = dest_dir.join("src/main.ts");
    fs::write(&m_path, main_ts)?;
    files.push(m_path);

    // src/App.vue
    let app_vue = format!(
        r#"<script setup lang="ts">
import {{ ref, onMounted }} from 'vue';

const data = ref<any>(null);
const loading = ref(true);

onMounted(async () => {{
  try {{
    const res = await fetch('/api/v1/items');
    data.value = await res.json();
  }} catch (err) {{
    console.error(err);
  }} finally {{
    loading.value = false;
  }}
}});
</script>

<template>
  <div class="flex min-h-screen flex-col items-center justify-center p-6">
    <div class="w-full max-w-lg rounded-2xl border border-slate-800 bg-slate-900 p-8 shadow-2xl">
      <div class="mb-4 flex items-center justify-between">
        <h1 class="text-2xl font-bold tracking-tight text-white">{name}</h1>
        <span class="rounded-full border border-emerald-800 bg-emerald-950 px-3 py-1 text-xs font-medium text-emerald-400">
          Vue 3 + TypeScript
        </span>
      </div>
      <p class="mb-6 text-sm text-slate-400">
        Connected to Terminus Axum backend on port 3000.
      </p>

      <div class="rounded-xl border border-slate-800 bg-slate-950 p-4">
        <p class="mb-2 text-xs font-semibold uppercase tracking-wider text-slate-500">
          API Response (/api/v1/items)
        </p>
        <p v-if="loading" class="text-sm text-slate-500">Fetching live data...</p>
        <pre v-else class="overflow-x-auto text-xs text-emerald-400 font-mono">{{{{ JSON.stringify(data, null, 2) }}}}</pre>
      </div>
    </div>
  </div>
</template>
"#,
        name = config.name
    );
    let app_path = dest_dir.join("src/App.vue");
    fs::write(&app_path, app_vue)?;
    files.push(app_path);

    Ok(files)
}
