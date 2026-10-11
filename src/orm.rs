//! Terminus ORM — Active Record ORM in Rust
//! Models, Active Record pattern, query scopes, relationships (has_many, belongs_to), and schema builders.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Active Record Model trait
pub trait Model: Sized + Clone + Serialize + for<'de> Deserialize<'de> {
    fn table_name() -> &'static str;
    fn primary_key() -> &'static str { "id" }
    fn has_timestamps() -> bool { true }

    fn get_id(&self) -> Option<&str>;
    fn set_id(&mut self, id: String);
    fn to_map(&self) -> HashMap<String, String>;
    fn from_map(map: HashMap<String, String>) -> Self;

    /// Begin a new query builder for this Model
    fn query() -> ModelQuery<Self> {
        ModelQuery::new()
    }

    /// Find a record by its primary key
    fn find(id: &str, conn: &rusqlite::Connection) -> Result<Option<Self>, String> {
        Self::query().find(id, conn)
    }

    /// Retrieve all records
    fn all(conn: &rusqlite::Connection) -> Result<Vec<Self>, String> {
        Self::query().get(conn)
    }

    /// Create and persist a new model instance
    fn create(attributes: HashMap<&str, &str>, conn: &rusqlite::Connection) -> Result<Self, String> {
        let mut string_map = HashMap::new();
        for (k, v) in attributes {
            string_map.insert(k.to_string(), v.to_string());
        }
        let now = chrono::Utc::now().to_rfc3339();
        if Self::has_timestamps() {
            string_map.entry("created_at".to_string()).or_insert_with(|| now.clone());
            string_map.entry("updated_at".to_string()).or_insert_with(|| now);
        }
        if !string_map.contains_key(Self::primary_key()) {
            let auto_id = format!("{:x}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0));
            string_map.insert(Self::primary_key().to_string(), auto_id);
        }

        let model = Self::from_map(string_map);
        model.save(conn)?;
        Ok(model)
    }

    /// Save (insert or update) the model instance
    fn save(&self, conn: &rusqlite::Connection) -> Result<(), String> {
        let map = self.to_map();
        let table = Self::table_name();
        let pk = Self::primary_key();
        let pk_val = self.get_id().unwrap_or_default();

        // Check if record exists
        let exists: bool = conn
            .query_row(
                &format!("SELECT COUNT(1) FROM {} WHERE {} = ?1", table, pk),
                rusqlite::params![pk_val],
                |r| r.get::<_, usize>(0),
            )
            .map(|c| c > 0)
            .unwrap_or(false);

        if exists {
            // UPDATE
            let mut set_clauses = Vec::new();
            let mut values = Vec::new();

            for (k, v) in &map {
                if k != pk {
                    set_clauses.push(format!("{} = ?", k));
                    values.push(v.clone());
                }
            }
            if Self::has_timestamps() && !map.contains_key("updated_at") {
                set_clauses.push("updated_at = ?".to_string());
                values.push(chrono::Utc::now().to_rfc3339());
            }

            let sql = format!("UPDATE {} SET {} WHERE {} = ?", table, set_clauses.join(", "), pk);
            values.push(pk_val.to_string());

            let params: Vec<&dyn rusqlite::ToSql> = values.iter().map(|v| v as &dyn rusqlite::ToSql).collect();
            conn.execute(&sql, params.as_slice()).map_err(|e| e.to_string())?;
        } else {
            // INSERT
            let mut cols = Vec::new();
            let mut placeholders = Vec::new();
            let mut values = Vec::new();

            for (k, v) in &map {
                cols.push(k.clone());
                placeholders.push("?".to_string());
                values.push(v.clone());
            }

            let sql = format!(
                "INSERT INTO {} ({}) VALUES ({})",
                table,
                cols.join(", "),
                placeholders.join(", ")
            );

            let params: Vec<&dyn rusqlite::ToSql> = values.iter().map(|v| v as &dyn rusqlite::ToSql).collect();
            conn.execute(&sql, params.as_slice()).map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    /// Delete the model instance
    fn delete(&self, conn: &rusqlite::Connection) -> Result<(), String> {
        let pk = Self::primary_key();
        let Some(id) = self.get_id() else {
            return Err("Cannot delete model without primary key".into());
        };
        let sql = format!("DELETE FROM {} WHERE {} = ?1", Self::table_name(), pk);
        conn.execute(&sql, rusqlite::params![id]).map_err(|e| e.to_string())?;
        Ok(())
    }
}

/// Dynamic Active Record model with dirty tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicModel {
    pub table: String,
    pub primary_key: String,
    pub attributes: HashMap<String, String>,
    pub original: HashMap<String, String>,
}

impl DynamicModel {
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            primary_key: "id".to_string(),
            attributes: HashMap::new(),
            original: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: impl Into<String>, val: impl Into<String>) -> &mut Self {
        self.attributes.insert(key.into(), val.into());
        self
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.attributes.get(key).map(|s| s.as_str())
    }

    pub fn is_dirty(&self) -> bool {
        self.attributes != self.original
    }

    pub fn get_dirty(&self) -> HashMap<String, String> {
        let mut dirty = HashMap::new();
        for (k, v) in &self.attributes {
            if self.original.get(k) != Some(v) {
                dirty.insert(k.clone(), v.clone());
            }
        }
        dirty
    }

    pub fn save(&mut self, conn: &rusqlite::Connection) -> Result<(), String> {
        let pk_val = self.get(&self.primary_key).map(String::from);
        let exists = if let Some(ref id) = pk_val {
            conn.query_row(
                &format!("SELECT COUNT(1) FROM {} WHERE {} = ?1", self.table, self.primary_key),
                rusqlite::params![id],
                |r| r.get::<_, usize>(0),
            )
            .map(|c| c > 0)
            .unwrap_or(false)
        } else {
            false
        };

        let now = chrono::Utc::now().to_rfc3339();

        if exists {
            let id = pk_val.unwrap();
            let mut set_clauses = Vec::new();
            let mut values = Vec::new();

            for (k, v) in &self.attributes {
                if k != &self.primary_key {
                    set_clauses.push(format!("{} = ?", k));
                    values.push(v.clone());
                }
            }
            set_clauses.push("updated_at = ?".to_string());
            values.push(now.clone());

            let sql = format!("UPDATE {} SET {} WHERE {} = ?", self.table, set_clauses.join(", "), self.primary_key);
            values.push(id);

            let params: Vec<&dyn rusqlite::ToSql> = values.iter().map(|v| v as &dyn rusqlite::ToSql).collect();
            conn.execute(&sql, params.as_slice()).map_err(|e| e.to_string())?;
        } else {
            let auto_id = pk_val.unwrap_or_else(|| format!("{:x}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)));
            self.attributes.insert(self.primary_key.clone(), auto_id.clone());
            self.attributes.entry("created_at".to_string()).or_insert_with(|| now.clone());
            self.attributes.entry("updated_at".to_string()).or_insert_with(|| now.clone());

            let mut cols = Vec::new();
            let mut placeholders = Vec::new();
            let mut values = Vec::new();

            for (k, v) in &self.attributes {
                cols.push(k.clone());
                placeholders.push("?".to_string());
                values.push(v.clone());
            }

            let sql = format!(
                "INSERT INTO {} ({}) VALUES ({})",
                self.table,
                cols.join(", "),
                placeholders.join(", ")
            );

            let params: Vec<&dyn rusqlite::ToSql> = values.iter().map(|v| v as &dyn rusqlite::ToSql).collect();
            conn.execute(&sql, params.as_slice()).map_err(|e| e.to_string())?;
        }

        self.original = self.attributes.clone();
        Ok(())
    }

    pub fn delete(&self, conn: &rusqlite::Connection) -> Result<(), String> {
        let Some(id) = self.get(&self.primary_key) else {
            return Err("Model has no primary key to delete".into());
        };
        let sql = format!("DELETE FROM {} WHERE {} = ?1", self.table, self.primary_key);
        conn.execute(&sql, rusqlite::params![id]).map_err(|e| e.to_string())?;
        Ok(())
    }
}

/// Fluent Model Query Builder
#[derive(Debug, Clone)]
pub struct ModelQuery<M: Model> {
    _marker: std::marker::PhantomData<M>,
    wheres: Vec<(String, String, String)>, // (col, op, val)
    order_by: Option<(String, String)>,
    limit_val: Option<usize>,
    offset_val: Option<usize>,
}

impl<M: Model> ModelQuery<M> {
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
            wheres: Vec::new(),
            order_by: None,
            limit_val: None,
            offset_val: None,
        }
    }

    pub fn where_eq(mut self, col: impl Into<String>, val: impl Into<String>) -> Self {
        self.wheres.push((col.into(), "=".to_string(), val.into()));
        self
    }

    #[allow(non_snake_case)]
    pub fn whereEq(self, col: impl Into<String>, val: impl Into<String>) -> Self {
        self.where_eq(col, val)
    }

    pub fn where_neq(mut self, col: impl Into<String>, val: impl Into<String>) -> Self {
        self.wheres.push((col.into(), "!=".to_string(), val.into()));
        self
    }

    #[allow(non_snake_case)]
    pub fn whereNeq(self, col: impl Into<String>, val: impl Into<String>) -> Self {
        self.where_neq(col, val)
    }

    pub fn where_gt(mut self, col: impl Into<String>, val: impl Into<String>) -> Self {
        self.wheres.push((col.into(), ">".to_string(), val.into()));
        self
    }

    #[allow(non_snake_case)]
    pub fn whereGt(self, col: impl Into<String>, val: impl Into<String>) -> Self {
        self.where_gt(col, val)
    }

    pub fn where_lt(mut self, col: impl Into<String>, val: impl Into<String>) -> Self {
        self.wheres.push((col.into(), "<".to_string(), val.into()));
        self
    }

    #[allow(non_snake_case)]
    pub fn whereLt(self, col: impl Into<String>, val: impl Into<String>) -> Self {
        self.where_lt(col, val)
    }

    pub fn where_like(mut self, col: impl Into<String>, pattern: impl Into<String>) -> Self {
        self.wheres.push((col.into(), "LIKE".to_string(), pattern.into()));
        self
    }

    #[allow(non_snake_case)]
    pub fn whereLike(self, col: impl Into<String>, pattern: impl Into<String>) -> Self {
        self.where_like(col, pattern)
    }

    pub fn order_by(mut self, col: impl Into<String>, dir: &str) -> Self {
        self.order_by = Some((col.into(), dir.to_uppercase()));
        self
    }

    #[allow(non_snake_case)]
    pub fn orderBy(self, col: impl Into<String>, dir: &str) -> Self {
        self.order_by(col, dir)
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit_val = Some(limit);
        self
    }

    pub fn offset(mut self, offset: usize) -> Self {
        self.offset_val = Some(offset);
        self
    }

    fn build_sql(&self) -> (String, Vec<String>) {
        let mut sql = format!("SELECT * FROM {}", M::table_name());
        let mut params = Vec::new();

        if !self.wheres.is_empty() {
            let clauses: Vec<String> = self
                .wheres
                .iter()
                .map(|(c, op, val)| {
                    params.push(val.clone());
                    format!("{} {} ?", c, op)
                })
                .collect();
            sql.push_str(&format!(" WHERE {}", clauses.join(" AND ")));
        }

        if let Some((ref col, ref dir)) = self.order_by {
            sql.push_str(&format!(" ORDER BY {} {}", col, dir));
        }

        if let Some(lim) = self.limit_val {
            sql.push_str(&format!(" LIMIT {}", lim));
        }

        if let Some(off) = self.offset_val {
            sql.push_str(&format!(" OFFSET {}", off));
        }

        (sql, params)
    }

    /// Execute the query and return matching Model instances
    pub fn get(&self, conn: &rusqlite::Connection) -> Result<Vec<M>, String> {
        let (sql, params) = self.build_sql();
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;

        let col_names: Vec<String> = stmt.column_names().into_iter().map(String::from).collect();
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();

        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            let mut map = HashMap::new();
            for (i, name) in col_names.iter().enumerate() {
                let val_str: String = row.get(i).unwrap_or_default();
                map.insert(name.clone(), val_str);
            }
            Ok(M::from_map(map))
        }).map_err(|e| e.to_string())?;

        let mut models = Vec::new();
        for r in rows {
            if let Ok(m) = r {
                models.push(m);
            }
        }
        Ok(models)
    }

    /// Execute the query and return the first matching Model instance
    pub fn first(&self, conn: &rusqlite::Connection) -> Result<Option<M>, String> {
        let q = self.clone().limit(1);
        let mut results = q.get(conn)?;
        Ok(results.pop())
    }

    /// Find by primary key
    pub fn find(&self, id: &str, conn: &rusqlite::Connection) -> Result<Option<M>, String> {
        Self::new().where_eq(M::primary_key(), id).first(conn)
    }

    /// Count matching records
    pub fn count(&self, conn: &rusqlite::Connection) -> Result<usize, String> {
        let mut sql = format!("SELECT COUNT(1) FROM {}", M::table_name());
        let mut params = Vec::new();

        if !self.wheres.is_empty() {
            let clauses: Vec<String> = self
                .wheres
                .iter()
                .map(|(c, op, val)| {
                    params.push(val.clone());
                    format!("{} {} ?", c, op)
                })
                .collect();
            sql.push_str(&format!(" WHERE {}", clauses.join(" AND ")));
        }

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();
        let count: usize = conn
            .query_row(&sql, param_refs.as_slice(), |r| r.get(0))
            .map_err(|e| e.to_string())?;

        Ok(count)
    }

    /// Delete all matching records
    pub fn delete_all(&self, conn: &rusqlite::Connection) -> Result<usize, String> {
        let mut sql = format!("DELETE FROM {}", M::table_name());
        let mut params = Vec::new();

        if !self.wheres.is_empty() {
            let clauses: Vec<String> = self
                .wheres
                .iter()
                .map(|(c, op, val)| {
                    params.push(val.clone());
                    format!("{} {} ?", c, op)
                })
                .collect();
            sql.push_str(&format!(" WHERE {}", clauses.join(" AND ")));
        }

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();
        let count = conn.execute(&sql, param_refs.as_slice()).map_err(|e| e.to_string())?;
        Ok(count)
    }
}

impl<M: Model> Default for ModelQuery<M> {
    fn default() -> Self {
        Self::new()
    }
}

/// HasMany Relationship
pub struct HasMany<Child: Model> {
    pub parent_id: String,
    pub foreign_key: String,
    _marker: std::marker::PhantomData<Child>,
}

impl<Child: Model> HasMany<Child> {
    pub fn new(parent_id: impl Into<String>, foreign_key: impl Into<String>) -> Self {
        Self {
            parent_id: parent_id.into(),
            foreign_key: foreign_key.into(),
            _marker: std::marker::PhantomData,
        }
    }

    /// Retrieve all related child models
    pub fn get(&self, conn: &rusqlite::Connection) -> Result<Vec<Child>, String> {
        Child::query().where_eq(&self.foreign_key, &self.parent_id).get(conn)
    }

    /// Create and persist a child model with the parent foreign key pre-assigned
    pub fn create(&self, attributes: HashMap<&str, &str>, conn: &rusqlite::Connection) -> Result<Child, String> {
        let mut string_map = HashMap::new();
        for (k, v) in attributes {
            string_map.insert(k.to_string(), v.to_string());
        }
        string_map.insert(self.foreign_key.clone(), self.parent_id.clone());
        let str_ref_map: HashMap<&str, &str> = string_map.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        Child::create(str_ref_map, conn)
    }
}

/// BelongsTo Relationship
pub struct BelongsTo<Parent: Model> {
    pub foreign_key_val: String,
    pub owner_key: String,
    _marker: std::marker::PhantomData<Parent>,
}

impl<Parent: Model> BelongsTo<Parent> {
    pub fn new(foreign_key_val: impl Into<String>, owner_key: impl Into<String>) -> Self {
        Self {
            foreign_key_val: foreign_key_val.into(),
            owner_key: owner_key.into(),
            _marker: std::marker::PhantomData,
        }
    }

    /// Retrieve the related parent model
    pub fn get(&self, conn: &rusqlite::Connection) -> Result<Option<Parent>, String> {
        Parent::query().where_eq(&self.owner_key, &self.foreign_key_val).first(conn)
    }
}

/// Blueprint for declarative SQLite schema migrations
pub struct Blueprint {
    pub table_name: String,
    pub columns: Vec<String>,
}

impl Blueprint {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            table_name: name.into(),
            columns: Vec::new(),
        }
    }

    pub fn id(&mut self) -> &mut Self {
        self.columns.push("id TEXT PRIMARY KEY".to_string());
        self
    }

    pub fn string(&mut self, col: &str) -> &mut Self {
        self.columns.push(format!("{} TEXT NOT NULL", col));
        self
    }

    pub fn nullable_string(&mut self, col: &str) -> &mut Self {
        self.columns.push(format!("{} TEXT", col));
        self
    }

    pub fn integer(&mut self, col: &str) -> &mut Self {
        self.columns.push(format!("{} INTEGER NOT NULL DEFAULT 0", col));
        self
    }

    pub fn boolean(&mut self, col: &str) -> &mut Self {
        self.columns.push(format!("{} INTEGER NOT NULL DEFAULT 0", col));
        self
    }

    pub fn timestamps(&mut self) -> &mut Self {
        self.columns.push("created_at TEXT NOT NULL".to_string());
        self.columns.push("updated_at TEXT NOT NULL".to_string());
        self
    }
}

/// Declarative Schema builder
pub struct Schema;

impl Schema {
    pub fn create<F: FnOnce(&mut Blueprint)>(
        conn: &rusqlite::Connection,
        table: &str,
        callback: F,
    ) -> Result<(), rusqlite::Error> {
        let mut bp = Blueprint::new(table);
        callback(&mut bp);

        let sql = format!(
            "CREATE TABLE IF NOT EXISTS {} ({});",
            table,
            bp.columns.join(", ")
        );
        conn.execute(&sql, [])?;
        Ok(())
    }

    pub fn drop_if_exists(conn: &rusqlite::Connection, table: &str) -> Result<(), rusqlite::Error> {
        conn.execute(&format!("DROP TABLE IF EXISTS {};", table), [])?;
        Ok(())
    }
}
