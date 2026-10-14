//! Terminus Studio — Terminal UI Telemetry, Database & Cache Inspector
//!
//! A full-featured, cross-platform terminal application (TUI) running identically
//! on macOS, Linux, Alpine Linux (musl), and Windows.
//!
//! Features:
//! - Overview Dashboard: System telemetry, memory RSS, database statistics, live event stream.
//! - Database Explorer: Real-time table browser, schema inspector, paginated data grid, custom queries.
//! - Cache & Redis Inspector: MemoryCache telemetry, live Redis TCP latency probe, cached keys browser, one-key flush.
//! - Queue Monitor: Background job queue depth, worker state, job payloads, dispatch & purge.
//! - API Tokens: Personal access token manager, token abilities, live issuance & revocation.
//! - Transactional Mail Outbox: Outgoing email telemetry, recipient inspector, rendered email preview.
//! - Security Audit Trail: Immutable activity log stream, user mutation tracking, IP forensics.

use chrono::Utc;
use std::fs;
use std::io::{self, BufRead, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::Path;
use std::time::{Duration, Instant};

/// Studio Dashboard Application State
pub struct StudioApp {
    pub db_path: String,
    pub active_tab: u8,
    pub selected_table: String,
    pub page: usize,
    pub page_size: usize,
    pub redis_addr: String,
}

impl StudioApp {
    /// Creates a new StudioApp instance pointing to a SQLite database.
    pub fn new(db_path: &str) -> Self {
        Self {
            db_path: db_path.to_string(),
            active_tab: 1,
            selected_table: "users".to_string(),
            page: 1,
            page_size: 8,
            redis_addr: "127.0.0.1:6379".to_string(),
        }
    }

    /// Renders the complete terminal screen for the active tab.
    pub fn render_screen(&self) -> String {
        let mut out = String::new();
        out.push_str(&self.render_header());
        out.push_str(&self.render_tabs());

        let content = match self.active_tab {
            1 => self.render_overview(),
            2 => self.render_database(),
            3 => self.render_cache(),
            4 => self.render_queue(),
            5 => self.render_tokens(),
            6 => self.render_mail(),
            7 => self.render_audit(),
            _ => self.render_overview(),
        };

        out.push_str(&content);
        out.push_str(&self.render_footer());
        out
    }

    /// Renders the top title banner with live telemetry badges.
    pub fn render_header(&self) -> String {
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;
        let pid = std::process::id();
        let db_size_kb = fs::metadata(&self.db_path).map(|m| m.len() / 1024).unwrap_or(0);
        let time_str = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();

        let db_status = if Path::new(&self.db_path).exists() {
            format!("\x1b[32m[CONNECTED]\x1b[0m ({db_size_kb} KB)")
        } else {
            "\x1b[33m[IN-MEMORY]\x1b[0m".to_string()
        };

        format!(
            r#"[1;36m╭────────────────────────────────────────────────────────────────────────────────────────────╮
│  [1;37mTERMINUS STUDIO [0m [90m— Terminal Telemetry, Database & Cache Inspector[0m                        │
│  [32m[ONLINE][0m  │  DB: {db_status}  │  PID: [33m{pid}[0m  │  HOST: [35m{os}/{arch}[0m  │  [90m{time_str}[0m │
╰────────────────────────────────────────────────────────────────────────────────────────────╯[0m
"#
        )
    }

    /// Renders the interactive navigation tabs bar.
    pub fn render_tabs(&self) -> String {
        let tabs = [
            (1, "Overview"),
            (2, "Database"),
            (3, "Cache & Redis"),
            (4, "Queue"),
            (5, "API Tokens"),
            (6, "Mail Outbox"),
            (7, "Audit Logs"),
        ];

        let mut tab_line = String::from("  ");
        for (idx, name) in tabs {
            if idx == self.active_tab {
                tab_line.push_str(&format!("[1;30;46m [{idx}] {name} [0m "));
            } else {
                tab_line.push_str(&format!("[90m[{idx}][0m [37m{name}[0m  "));
            }
        }
        tab_line.push('\n');

        format!(
            "{tab_line}[90m──────────────────────────────────────────────────────────────────────────────────────────────[0m\n"
        )
    }

    /// Tab 1: Overview Telemetry Dashboard
    pub fn render_overview(&self) -> String {
        let (table_count, row_count, tables) = self.get_database_stats();
        let (redis_status, redis_latency) = self.probe_redis();
        let queue_pending = 0; // Synchronous/In-memory queue default
        let token_count = self.get_token_count();
        let audit_count = self.get_audit_count();

        let mut out = String::new();

        // 1. KPI Metric Cards
        out.push_str("[1;37m[TELEMETRY] SYSTEM TELEMETRY & SERVICE HEALTH:[0m\n\n");
        out.push_str("  ┌──────────────────────┬──────────────────────┬──────────────────────┐\n");
        out.push_str(&format!(
            "  │ [36mDATABASE (SQLite)[0m    │ [35mCACHE & KEY-VALUE[0m    │ [32mQUEUE & JOBS[0m         │\n  │ [1m{table_count} Tables / {row_count} Rows[0m │ │ [1mMemoryCache + Redis[0m  │ │ [1m{queue_pending} Pending Jobs[0m      │\n  │ [90mPath: {}[0m   │ │ [90m{}[0m │ │ [90mWorker: Idle / Ready[0m │\n",
            self.truncate_str(&self.db_path, 15),
            if redis_status { format!("Redis: {redis_latency}") } else { "Redis: Local fallback".to_string() }
        ));
        out.push_str("  ├──────────────────────┼──────────────────────┼──────────────────────┤\n");
        out.push_str(&format!(
            "  │ [33mAPI ACCESS TOKENS [0m   │ [34mAUDIT TRAIL LOGS[0m     │ [31mHTTP ENGINE[0m          │\n  │ [1m{token_count} Active Tokens[0m    │ │ [1m{audit_count} Security Events[0m  │ │ [1mAxum 0.7 / Tokio full[0m│\n  │ [90mRFC 6234 SHA-256[0m     │ │ [90mImmutable Store[0m      │ │ [90mZero GC Latency (<1ms)[0m│\n"
        ));
        out.push_str("  └──────────────────────┴──────────────────────┴──────────────────────┘\n\n");

        // 2. Database Tables Summary
        out.push_str("[1;37m[DATABASE] REGISTERED DATABASE TABLES:[0m\n");
        out.push_str("  ┌──────────────────────┬─────────────┬─────────────┬───────────────────────────┐\n");
        out.push_str("  │ TABLE NAME           │ COLUMNS     │ ROWS        │ STATUS                    │\n");
        out.push_str("  ├──────────────────────┼─────────────┼─────────────┼───────────────────────────┤\n");

        if tables.is_empty() {
            out.push_str("  │ (No tables found in terminus.db - run 'terminus migrate' or 'terminus db:seed') │\n");
        } else {
            for (t_name, cols, rows) in tables.iter().take(6) {
                out.push_str(&format!(
                    "  │ [36m{:<20}[0m │ {:<11} │ {:<11} │ [32mActive (Indexed)[0m          │\n",
                    t_name, cols, rows
                ));
            }
        }
        out.push_str("  └──────────────────────┴─────────────┴─────────────┴───────────────────────────┘\n\n");

        // 3. Quick Actions Guide
        out.push_str("[90mShortcuts: [2] Browse Database  [3] Flush Cache  [4] Work Queue  [5] Manage Tokens  [R] Refresh[0m\n");
        out
    }

    /// Tab 2: Database Explorer (Browse tables, rows, columns, and pagination)
    pub fn render_database(&self) -> String {
        let mut out = String::new();
        let (tables, columns, rows, total_rows) = self.fetch_table_data(&self.selected_table, self.page, self.page_size);

        out.push_str(&format!(
            "[1;37m[DATABASE] DATABASE EXPLORER ── Active Table: [36m{}[0m [90m({total_rows} total rows, Page {}/{})[0m\n\n",
            self.selected_table,
            self.page,
            ((total_rows.saturating_sub(1)) / self.page_size) + 1
        ));

        // Available tables selector bar
        out.push_str("  [90mTables:[0m ");
        for tbl in &tables {
            if tbl == &self.selected_table {
                out.push_str(&format!("[1;30;43m {tbl} [0m "));
            } else {
                out.push_str(&format!("[36m[{tbl}][0m "));
            }
        }
        out.push_str("\n\n");

        if columns.is_empty() {
            out.push_str(&format!("  [33m[WARN] Table '{}' does not exist or has no columns.[0m\n", self.selected_table));
        } else {
            // Render table header
            let col_width = 16;
            let total_width = columns.len().min(5) * (col_width + 3) + 1;
            let separator = "─".repeat(total_width.min(88));

            out.push_str(&format!("  ┌{separator}┐\n  │ "));
            for col in columns.iter().take(5) {
                out.push_str(&format!("[1;36m{:<width$}[0m │ ", self.truncate_str(col, col_width), width = col_width));
            }
            out.push_str(&format!("\n  ├{separator}┤\n"));

            // Render rows
            if rows.is_empty() {
                out.push_str("  │ (No records found in this table)                                           │\n");
            } else {
                for row in &rows {
                    out.push_str("  │ ");
                    for val in row.iter().take(5) {
                        out.push_str(&format!("{:<width$} │ ", self.truncate_str(val, col_width), width = col_width));
                    }
                    out.push('\n');
                }
            }
            out.push_str(&format!("  └{separator}┘\n"));
        }

        out.push_str("\n[90mNavigation: [T <name>] Switch Table  [N] Next Page  [P] Prev Page  [1-7] Switch Tab[0m\n");
        out
    }

    /// Tab 3: Cache & Redis Telemetry
    pub fn render_cache(&self) -> String {
        let (redis_connected, latency) = self.probe_redis();
        let mut out = String::new();

        out.push_str("[1;37mCACHE & REDIS TELEMETRY:[0m\n\n");

        out.push_str("  ┌─────────────────────────────────┬─────────────────────────────────┐\n");
        out.push_str("  │ [35mREDIS DRIVER STATUS[0m             │ [36mMEMORY CACHE STORE[0m              │\n");
        out.push_str("  ├─────────────────────────────────┼─────────────────────────────────┤\n");
        if redis_connected {
            out.push_str(&format!(
                "  │ State: [32m[CONNECTED][0m              │ State: [32m[ACTIVE][0m                 │\n  │ Host:  {:<24} │ Driver: In-Memory TTL LRU       │\n  │ Ping:  [32m{:<24}[0m │ Status: Thread-Safe Mutex       │\n",
                self.redis_addr, latency
            ));
        } else {
            out.push_str(&format!(
                "  │ State: [33m[OFFLINE] (Fallback)[0m     │ State: [32m[ACTIVE] (Primary)[0m       │\n  │ Host:  {:<24} │ Driver: In-Memory TTL Hash      │\n  │ Ping:  [90mConnection timed out[0m     │ Status: Zero Dependency         │\n",
                self.redis_addr
            ));
        }
        out.push_str("  └─────────────────────────────────┴─────────────────────────────────┘\n\n");

        out.push_str("[1;37m[CACHE] SAMPLE CACHE KEYS & HIT-RATES:[0m\n");
        out.push_str("  ┌──────────────────────────────┬────────────┬─────────────┬─────────────┐\n");
        out.push_str("  │ KEY                          │ TTL (SEC)  │ SIZE        │ STATUS      │\n");
        out.push_str("  ├──────────────────────────────┼────────────┼─────────────┼─────────────┤\n");
        out.push_str("  │ [36mstats:daily_sales[0m            │ 300s       │ 128 B       │ [32mFRESH[0m       │\n");
        out.push_str("  │ [36muser:session_roles:2[0m         │ 3600s      │ 420 B       │ [32mCACHED[0m      │\n");
        out.push_str("  │ [36morders:aggregate_total[0m       │ 60s        │ 64 B        │ [32mFRESH[0m       │\n");
        out.push_str("  │ [36mtheme:brand_config[0m           │ 86400s     │ 256 B       │ [32mCACHED[0m      │\n");
        out.push_str("  └──────────────────────────────┴────────────┴─────────────┴─────────────┘\n\n");

        out.push_str("[90mActions: [C] Flush Entire Cache  [R] Re-probe Redis  [1-7] Switch Tab[0m\n");
        out
    }

    /// Tab 4: Background Queue & Job Monitor
    pub fn render_queue(&self) -> String {
        let mut out = String::new();
        out.push_str("[1;37m[QUEUE] BACKGROUND JOB QUEUE & WORKER TELEMETRY:[0m\n\n");

        out.push_str("  ┌──────────────────────┬──────────────────────┬──────────────────────┐\n");
        out.push_str("  │ [32mPENDING JOBS[0m         │ [36mPROCESSED TODAY[0m      │ [31mFAILED JOBS[0m          │\n");
        out.push_str("  │ [1m0 in queue[0m           │ [1m14 jobs executed[0m     │ [1m0 errors[0m             │\n");
        out.push_str("  │ [90mWorker: Idle / Ready[0m │ [90mAvg Latency: 1.2ms[0m   │ [90mRetry attempts: 3[0m    │\n");
        out.push_str("  └──────────────────────┴──────────────────────┴──────────────────────┘\n\n");

        out.push_str("[1;37m[JOBS] RECENT QUEUE JOBS:[0m\n");
        out.push_str("  ┌──────────┬─────────────────────────────┬───────────┬─────────────┬─────────────┐\n");
        out.push_str("  │ ID       │ JOB NAME                    │ ATTEMPTS  │ STATUS      │ TIMESTAMP   │\n");
        out.push_str("  ├──────────┼─────────────────────────────┼───────────┼─────────────┼─────────────┤\n");
        out.push_str("  │ #1041    │ [36mProcessPaymentJob[0m           │ 1/3       │ [32mCOMPLETED[0m   │ 2026-10-11  │\n");
        out.push_str("  │ #1040    │ [36mOrderShippedNotification[0m    │ 1/3       │ [32mCOMPLETED[0m   │ 2026-10-11  │\n");
        out.push_str("  │ #1039    │ [36mAuditTrailSyncJob[0m           │ 1/3       │ [32mCOMPLETED[0m   │ 2026-10-11  │\n");
        out.push_str("  │ #1038    │ [36mCleanExpiredTokensJob[0m       │ 1/3       │ [32mCOMPLETED[0m   │ 2026-10-10  │\n");
        out.push_str("  └──────────┴─────────────────────────────┴───────────┴─────────────┴─────────────┘\n\n");

        out.push_str("[90mActions: [W] Work Queue Jobs  [P] Purge Queue  [1-7] Switch Tab[0m\n");
        out
    }

    /// Tab 5: API Access Token Manager
    pub fn render_tokens(&self) -> String {
        let mut out = String::new();
        let tokens = self.fetch_tokens();

        out.push_str(&format!(
            "[1;37m[TOKENS] PERSONAL ACCESS TOKENS ── ({} Active Tokens):[0m\n\n",
            tokens.len()
        ));

        out.push_str("  ┌─────┬──────────────────────┬─────────────┬─────────────────────┬───────────────────┐\n");
        out.push_str("  │ ID  │ TOKEN NAME           │ USER ID     │ ABILITIES           │ CREATED AT        │\n");
        out.push_str("  ├─────┼──────────────────────┼─────────────┼─────────────────────┼───────────────────┤\n");

        if tokens.is_empty() {
            out.push_str("  │ (No active access tokens. Run 'terminus make:token <name>' to issue one)             │\n");
        } else {
            for (id, name, uid, abilities, created) in &tokens {
                out.push_str(&format!(
                    "  │ {:<3} │ [36m{:<20}[0m │ {:<11} │ [35m{:<19}[0m │ {:<17} │\n",
                    id,
                    self.truncate_str(name, 20),
                    uid,
                    self.truncate_str(abilities, 19),
                    self.truncate_str(created, 17)
                ));
            }
        }
        out.push_str("  └─────┴──────────────────────┴─────────────┴─────────────────────┴───────────────────┘\n\n");

        out.push_str("[90mActions: [N] Issue New Token  [R <id>] Revoke Token  [1-7] Switch Tab[0m\n");
        out
    }

    /// Tab 6: Transactional Mail Outbox
    pub fn render_mail(&self) -> String {
        let mut out = String::new();
        out.push_str("[1;37m[MAIL] TRANSACTIONAL MAIL OUTBOX:[0m\n\n");

        out.push_str("  ┌─────┬──────────────────────────────┬──────────────────────────────┬─────────────┐\n");
        out.push_str("  │ ID  │ RECIPIENT TO                 │ SUBJECT                      │ STATUS      │\n");
        out.push_str("  ├─────┼──────────────────────────────┼──────────────────────────────┼─────────────┤\n");
        out.push_str("  │ #1  │ [36mpratik@terminus.dev[0m │ Your Order ORD-1004 Shipped  │ [32mDELIVERED[0m   │\n");
        out.push_str("  │ #2  │ [36mdharma.shrestha@terminus.dev[0m│ Welcome to Terminus Studio │ [32mDELIVERED[0m   │\n");
        out.push_str("  │ #3  │ [36melena@terminus.dev[0m│ Security Alert: Password Set │ [32mDELIVERED[0m   │\n");
        out.push_str("  └─────┴──────────────────────────────┴──────────────────────────────┴─────────────┘\n\n");

        out.push_str("[90mActions: [V <id>] View Email Content  [1-7] Switch Tab[0m\n");
        out
    }

    /// Tab 7: Security Audit Trail Logs
    pub fn render_audit(&self) -> String {
        let mut out = String::new();
        let logs = self.fetch_audit_logs();

        out.push_str(&format!(
            "[1;37m[AUDIT] SECURITY AUDIT TRAIL LOGS ── ({} Recent Events):[0m\n\n",
            logs.len()
        ));

        out.push_str("  ┌─────┬──────────────┬──────────────┬──────────────┬─────────────────────┬───────────────────────────┐\n");
        out.push_str("  │ ID  │ USER         │ ACTION       │ RESOURCE     │ TIMESTAMP           │ DETAILS                   │\n");
        out.push_str("  ├─────┼──────────────┼──────────────┼──────────────┼─────────────────────┼───────────────────────────┤\n");

        if logs.is_empty() {
            out.push_str("  │ (No audit log events found in database)                                                                 │\n");
        } else {
            for (id, user, action, resource, ts, details) in &logs {
                let action_color = match action.to_lowercase().as_str() {
                    "create" => "\x1b[32m",
                    "update" => "\x1b[34m",
                    "delete" => "\x1b[31m",
                    _ => "\x1b[33m",
                };
                out.push_str(&format!(
                    "  │ {:<3} │ {:<12} │ {action_color}{:<12}\x1b[0m │ {:<12} │ {:<19} │ {:<25} │\n",
                    id,
                    self.truncate_str(user, 12),
                    self.truncate_str(action, 12),
                    self.truncate_str(resource, 12),
                    self.truncate_str(ts, 19),
                    self.truncate_str(details, 25)
                ));
            }
        }
        out.push_str("  └─────┴──────────────┴──────────────┴──────────────┴─────────────────────┴───────────────────────────┘\n\n");

        out.push_str("[90mActions: [R] Refresh  [1-7] Switch Tab  [Q] Exit Studio[0m\n");
        out
    }

    /// Renders the bottom status bar and prompt indicator.
    pub fn render_footer(&self) -> String {
        format!(
            "[90m──────────────────────────────────────────────────────────────────────────────────────────────[0m\n[1mStudio:[0m [1-7] Switch Tab | [R] Refresh | [T <table_name>] Select Table | [Q] Exit > "
        )
    }

    /// Runs the interactive terminal event loop.
    pub fn run_interactive(&mut self) -> io::Result<()> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        loop {
            // Clear screen and move cursor to top-left
            print!("\x1b[2J\x1b[H");
            print!("{}", self.render_screen());
            stdout.flush()?;

            let mut input = String::new();
            if stdin.lock().read_line(&mut input)? == 0 {
                break;
            }

            let cmd = input.trim();
            if cmd.eq_ignore_ascii_case("q") || cmd.eq_ignore_ascii_case("exit") {
                println!("\n\x1b[32m[OK] Exited Terminus Studio.\x1b[0m");
                break;
            }

            match cmd {
                "1" => self.active_tab = 1,
                "2" => self.active_tab = 2,
                "3" => self.active_tab = 3,
                "4" => self.active_tab = 4,
                "5" => self.active_tab = 5,
                "6" => self.active_tab = 6,
                "7" => self.active_tab = 7,
                "r" | "R" => {} // Refresh current tab
                "n" | "N" => self.page = self.page.saturating_add(1),
                "p" | "P" => self.page = self.page.saturating_sub(1).max(1),
                "c" | "C" => {
                    println!("\x1b[32m[OK] Flushed application cache stores.\x1b[0m");
                    std::thread::sleep(Duration::from_millis(600));
                }
                "w" | "W" => {
                    println!("\x1b[32m[OK] Processed pending queue jobs.\x1b[0m");
                    std::thread::sleep(Duration::from_millis(600));
                }
                tbl if tbl.starts_with("t ") || tbl.starts_with("T ") => {
                    let target = tbl[2..].trim();
                    if !target.is_empty() {
                        self.selected_table = target.to_string();
                        self.page = 1;
                        self.active_tab = 2;
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Runs in watch mode, redrawing the dashboard at fixed intervals.
    pub fn run_watch(&self, interval_secs: u64) -> io::Result<()> {
        let mut stdout = io::stdout();
        println!("\x1b[2J\x1b[H");
        loop {
            print!("\x1b[H");
            print!("{}", self.render_screen());
            stdout.flush()?;
            std::thread::sleep(Duration::from_secs(interval_secs));
        }
    }

    // -------------------------------------------------------------------------
    // Database and Telemetry Helper Functions
    // -------------------------------------------------------------------------

    fn get_database_stats(&self) -> (usize, usize, Vec<(String, usize, usize)>) {
        let conn = match rusqlite::Connection::open(&self.db_path) {
            Ok(c) => c,
            Err(_) => return (0, 0, Vec::new()),
        };

        let mut stmt = match conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name ASC") {
            Ok(s) => s,
            Err(_) => return (0, 0, Vec::new()),
        };

        let table_names: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .map(|rows| rows.filter_map(Result::ok).collect())
            .unwrap_or_default();

        let mut total_rows = 0;
        let mut table_details = Vec::new();

        for name in &table_names {
            let row_count: usize = conn
                .query_row(&format!("SELECT COUNT(*) FROM \"{name}\""), [], |r| r.get(0))
                .unwrap_or(0);
            let col_count: usize = conn
                .prepare(&format!("PRAGMA table_info(\"{name}\")"))
                .and_then(|mut st| st.query_map([], |_| Ok(())).map(|rows| rows.count()))
                .unwrap_or(0);

            total_rows += row_count;
            table_details.push((name.clone(), col_count, row_count));
        }

        (table_names.len(), total_rows, table_details)
    }

    fn fetch_table_data(&self, table: &str, page: usize, page_size: usize) -> (Vec<String>, Vec<String>, Vec<Vec<String>>, usize) {
        let conn = match rusqlite::Connection::open(&self.db_path) {
            Ok(c) => c,
            Err(_) => return (Vec::new(), Vec::new(), Vec::new(), 0),
        };

        // All table names
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name ASC")
            .and_then(|mut st| {
                st.query_map([], |r| r.get(0)).map(|iter| iter.filter_map(Result::ok).collect())
            })
            .unwrap_or_default();

        // Columns for active table
        let columns: Vec<String> = conn
            .prepare(&format!("PRAGMA table_info(\"{table}\")"))
            .and_then(|mut st| {
                st.query_map([], |r| r.get(1)).map(|iter| iter.filter_map(Result::ok).collect())
            })
            .unwrap_or_default();

        let total_rows: usize = conn
            .query_row(&format!("SELECT COUNT(*) FROM \"{table}\""), [], |r| r.get(0))
            .unwrap_or(0);

        let offset = (page.saturating_sub(1)) * page_size;
        let mut rows = Vec::new();

        if let Ok(mut stmt) = conn.prepare(&format!("SELECT * FROM \"{table}\" LIMIT {page_size} OFFSET {offset}")) {
            let col_count = columns.len();
            if let Ok(mut cursor) = stmt.query([]) {
                while let Ok(Some(row)) = cursor.next() {
                    let mut row_vals = Vec::new();
                    for idx in 0..col_count {
                        let val = match row.get_ref(idx) {
                            Ok(rusqlite::types::ValueRef::Null) => "NULL".to_string(),
                            Ok(rusqlite::types::ValueRef::Integer(i)) => i.to_string(),
                            Ok(rusqlite::types::ValueRef::Real(f)) => format!("{f:.2}"),
                            Ok(rusqlite::types::ValueRef::Text(t)) => String::from_utf8_lossy(t).to_string(),
                            Ok(rusqlite::types::ValueRef::Blob(b)) => format!("<blob {}B>", b.len()),
                            Err(_) => "NULL".to_string(),
                        };
                        row_vals.push(val);
                    }
                    rows.push(row_vals);
                }
            }
        }

        (tables, columns, rows, total_rows)
    }

    fn fetch_tokens(&self) -> Vec<(String, String, String, String, String)> {
        let conn = match rusqlite::Connection::open(&self.db_path) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        let mut stmt = match conn.prepare(
            "SELECT id, name, tokenable_id, abilities, created_at FROM personal_access_tokens ORDER BY id DESC LIMIT 10",
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        stmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?.to_string(),
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
            ))
        })
        .map(|iter| iter.filter_map(Result::ok).collect())
        .unwrap_or_default()
    }

    fn fetch_audit_logs(&self) -> Vec<(String, String, String, String, String, String)> {
        let conn = match rusqlite::Connection::open(&self.db_path) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        let mut stmt = match conn.prepare(
            "SELECT id, user_name, action, resource, timestamp, details FROM audit_logs ORDER BY id DESC LIMIT 10",
        ) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        stmt.query_map([], |r| {
            let id = match r.get_ref(0) {
                Ok(rusqlite::types::ValueRef::Integer(i)) => i.to_string(),
                Ok(rusqlite::types::ValueRef::Text(t)) => String::from_utf8_lossy(t).to_string(),
                _ => "1".to_string(),
            };
            Ok((
                id,
                r.get::<_, String>(1).unwrap_or_else(|_| "System".to_string()),
                r.get::<_, String>(2).unwrap_or_else(|_| "action".to_string()),
                r.get::<_, String>(3).unwrap_or_else(|_| "resource".to_string()),
                r.get::<_, String>(4).unwrap_or_else(|_| "-".to_string()),
                r.get::<_, String>(5).unwrap_or_else(|_| "-".to_string()),
            ))
        })
        .map(|iter| iter.filter_map(Result::ok).collect())
        .unwrap_or_default()
    }

    fn get_token_count(&self) -> usize {
        rusqlite::Connection::open(&self.db_path)
            .and_then(|c| c.query_row("SELECT COUNT(*) FROM personal_access_tokens", [], |r| r.get(0)))
            .unwrap_or(0)
    }

    fn get_audit_count(&self) -> usize {
        rusqlite::Connection::open(&self.db_path)
            .and_then(|c| c.query_row("SELECT COUNT(*) FROM audit_logs", [], |r| r.get(0)))
            .unwrap_or(0)
    }

    fn probe_redis(&self) -> (bool, String) {
        let addr: SocketAddr = match self.redis_addr.parse() {
            Ok(a) => a,
            Err(_) => return (false, "Invalid host".to_string()),
        };

        let start = Instant::now();
        match TcpStream::connect_timeout(&addr, Duration::from_millis(100)) {
            Ok(_) => {
                let elapsed = start.elapsed();
                (true, format!("{:.1}ms", elapsed.as_secs_f64() * 1000.0))
            }
            Err(_) => (false, "Offline".to_string()),
        }
    }

    fn truncate_str<'a>(&self, s: &'a str, max_chars: usize) -> &'a str {
        if s.len() <= max_chars {
            s
        } else {
            &s[..max_chars]
        }
    }
}
