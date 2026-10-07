use std::sync::{Arc, Mutex};
use rusqlite::{params, Connection};
use crate::domain::{Role, User, UserStatus};
use crate::repository::UserRepository;
use crate::resource::QueryState;

pub struct SqliteUserRepository {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteUserRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Result<Self, rusqlite::Error> {
        {
            let db = conn.lock().unwrap();
            db.execute(
                r#"CREATE TABLE IF NOT EXISTS users (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL,
                    email TEXT NOT NULL,
                    role TEXT NOT NULL,
                    status TEXT NOT NULL,
                    created_at TEXT NOT NULL
                );"#,
                [],
            )?;

            // Seed initial data if table is brand new
            let count: i64 = db.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))?;
            if count == 0 {
                let seed_users = vec![
                    ("Hari Bahadur Bhujel", "hari.bhujel@oxideadmin.dev", "Founder", "Active", "2026-09-01"),
                    ("Pratik Bhujel", "pratik.bhujel@oxideadmin.dev", "Superadmin", "Active", "2026-09-02"),
                    ("Dharma Raj Shrestha", "dharma.shrestha@oxideadmin.dev", "Admin", "Active", "2026-09-05"),
                    ("Gokul Subedi", "gokul.subedi@oxideadmin.dev", "Admin", "Active", "2026-09-08"),
                    ("Ranjan Gumanju", "ranjan.gumanju@oxideadmin.dev", "Member", "Active", "2026-09-12"),
                    ("Sampanna Rimal", "sampanna.rimal@oxideadmin.dev", "Member", "Active", "2026-09-15"),
                    ("Subash Ranabat", "subash.ranabat@oxideadmin.dev", "Member", "Active", "2026-09-19"),
                    ("Lasta Chaudhary", "lasta.chaudhary@oxideadmin.dev", "Editor", "Active", "2026-09-22"),
                    ("Ryan Koirala", "ryan.koirala@oxideadmin.dev", "Member", "Pending", "2026-10-01"),
                    ("Alex Vance", "alex.vance@example.com", "Member", "Suspended", "2026-10-04"),
                ];

                for (name, email, role, status, date) in seed_users {
                    db.execute(
                        "INSERT INTO users (name, email, role, status, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                        params![name, email, role, status, date],
                    )?;
                }
            }
        }

        Ok(Self { conn })
    }
}

impl UserRepository for SqliteUserRepository {
    fn list(&self, query: &QueryState) -> (Vec<User>, usize) {
        let db = self.conn.lock().unwrap();

        let search_pattern = format!("%{}%", query.search.to_lowercase());
        let count_sql = if query.search.is_empty() {
            "SELECT COUNT(*) FROM users".to_string()
        } else {
            "SELECT COUNT(*) FROM users WHERE LOWER(name) LIKE ?1 OR LOWER(email) LIKE ?1 OR LOWER(role) LIKE ?1".to_string()
        };

        let total: usize = if query.search.is_empty() {
            db.query_row(&count_sql, [], |r| r.get(0)).unwrap_or(0)
        } else {
            db.query_row(&count_sql, params![search_pattern], |r| r.get(0)).unwrap_or(0)
        };

        let per_page = if query.per_page == 0 { 8 } else { query.per_page };
        let page = if query.page == 0 { 1 } else { query.page };
        let offset = (page - 1) * per_page;

        let order_by = match query.sort_by.as_deref() {
            Some("name") => "name",
            Some("created_at") => "created_at",
            _ => "id",
        };
        let direction = if query.sort_desc { "DESC" } else { "ASC" };

        let select_sql = if query.search.is_empty() {
            format!("SELECT id, name, email, role, status, created_at FROM users ORDER BY {order_by} {direction} LIMIT ?1 OFFSET ?2")
        } else {
            format!("SELECT id, name, email, role, status, created_at FROM users WHERE LOWER(name) LIKE ?1 OR LOWER(email) LIKE ?1 OR LOWER(role) LIKE ?1 ORDER BY {order_by} {direction} LIMIT ?2 OFFSET ?3")
        };

        let mut stmt = db.prepare(&select_sql).unwrap();

        let user_iter = if query.search.is_empty() {
            stmt.query_map(params![per_page, offset], |row| {
                let id: i64 = row.get(0)?;
                let name: String = row.get(1)?;
                let email: String = row.get(2)?;
                let role_str: String = row.get(3)?;
                let status_str: String = row.get(4)?;
                let created_at: String = row.get(5)?;

                Ok(User {
                    id: id.to_string(),
                    name,
                    email,
                    role: Role::from_str_loose(&role_str),
                    status: UserStatus::from_str_loose(&status_str),
                    created_at,
                })
            }).unwrap().collect::<Result<Vec<_>, _>>().unwrap_or_default()
        } else {
            stmt.query_map(params![search_pattern, per_page, offset], |row| {
                let id: i64 = row.get(0)?;
                let name: String = row.get(1)?;
                let email: String = row.get(2)?;
                let role_str: String = row.get(3)?;
                let status_str: String = row.get(4)?;
                let created_at: String = row.get(5)?;

                Ok(User {
                    id: id.to_string(),
                    name,
                    email,
                    role: Role::from_str_loose(&role_str),
                    status: UserStatus::from_str_loose(&status_str),
                    created_at,
                })
            }).unwrap().collect::<Result<Vec<_>, _>>().unwrap_or_default()
        };

        (user_iter, total)
    }

    fn find_by_id(&self, id: &str) -> Option<User> {
        let db = self.conn.lock().unwrap();
        let parsed_id: i64 = id.parse().ok()?;

        db.query_row(
            "SELECT id, name, email, role, status, created_at FROM users WHERE id = ?1",
            params![parsed_id],
            |row| {
                let id: i64 = row.get(0)?;
                let name: String = row.get(1)?;
                let email: String = row.get(2)?;
                let role_str: String = row.get(3)?;
                let status_str: String = row.get(4)?;
                let created_at: String = row.get(5)?;

                Ok(User {
                    id: id.to_string(),
                    name,
                    email,
                    role: Role::from_str_loose(&role_str),
                    status: UserStatus::from_str_loose(&status_str),
                    created_at,
                })
            },
        ).ok()
    }

    fn save(&self, user: User) -> Result<String, String> {
        let db = self.conn.lock().unwrap();
        db.execute(
            "INSERT INTO users (name, email, role, status, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![user.name, user.email, user.role.as_str(), user.status.as_str(), user.created_at],
        ).map_err(|e| e.to_string())?;

        let last_id = db.last_insert_rowid();
        Ok(last_id.to_string())
    }

    fn update(&self, id: &str, name: String, email: String, role: Role, status: UserStatus) -> Result<(), String> {
        let db = self.conn.lock().unwrap();
        let parsed_id: i64 = id.parse().map_err(|_| "Invalid ID".to_string())?;

        db.execute(
            "UPDATE users SET name = ?1, email = ?2, role = ?3, status = ?4 WHERE id = ?5",
            params![name, email, role.as_str(), status.as_str(), parsed_id],
        ).map_err(|e| e.to_string())?;

        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), String> {
        let db = self.conn.lock().unwrap();
        let parsed_id: i64 = id.parse().map_err(|_| "Invalid ID".to_string())?;

        db.execute("DELETE FROM users WHERE id = ?1", params![parsed_id])
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}
