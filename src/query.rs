//! Eloquent-style fluent query builder for OxideAdmin
//! Supports type-safe, parameterized SQL query construction with camelCase aliases.

#[derive(Debug, Clone, PartialEq)]
pub enum WhereClause {
    Eq(String, String),
    Neq(String, String),
    Gt(String, String),
    Lt(String, String),
    Like(String, String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderDir {
    Asc,
    Desc,
}

#[derive(Debug, Clone)]
pub struct Query {
    pub table_name: String,
    pub columns: Vec<String>,
    pub wheres: Vec<WhereClause>,
    pub order_by: Option<(String, OrderDir)>,
    pub limit_val: Option<usize>,
    pub offset_val: Option<usize>,
}

impl Query {
    /// Start a new query on a table (Laravel `DB::table('users')` / `User::query()`)
    pub fn table(table: impl Into<String>) -> Self {
        Self {
            table_name: table.into(),
            columns: vec!["*".to_string()],
            wheres: Vec::new(),
            order_by: None,
            limit_val: None,
            offset_val: None,
        }
    }

    /// Select specific columns (Laravel `->select(['id', 'name'])`)
    pub fn select(mut self, cols: Vec<&str>) -> Self {
        self.columns = cols.into_iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add an equality where clause (Laravel `->where('status', 'Active')`)
    pub fn where_eq(mut self, column: impl Into<String>, value: impl Into<String>) -> Self {
        self.wheres.push(WhereClause::Eq(column.into(), value.into()));
        self
    }

    #[allow(non_snake_case)]
    pub fn whereEq(self, column: impl Into<String>, value: impl Into<String>) -> Self {
        self.where_eq(column, value)
    }

    /// Add a not-equals where clause (Laravel `->where('status', '!=', 'Archived')`)
    pub fn where_neq(mut self, column: impl Into<String>, value: impl Into<String>) -> Self {
        self.wheres.push(WhereClause::Neq(column.into(), value.into()));
        self
    }

    #[allow(non_snake_case)]
    pub fn whereNeq(self, column: impl Into<String>, value: impl Into<String>) -> Self {
        self.where_neq(column, value)
    }

    /// Add a LIKE where clause (Laravel `->where('name', 'like', '%John%')`)
    pub fn where_like(mut self, column: impl Into<String>, pattern: impl Into<String>) -> Self {
        self.wheres.push(WhereClause::Like(column.into(), pattern.into()));
        self
    }

    #[allow(non_snake_case)]
    pub fn whereLike(self, column: impl Into<String>, pattern: impl Into<String>) -> Self {
        self.where_like(column, pattern)
    }

    /// Add a greater-than clause (Laravel `->where('amount', '>', '100')`)
    pub fn where_gt(mut self, column: impl Into<String>, value: impl Into<String>) -> Self {
        self.wheres.push(WhereClause::Gt(column.into(), value.into()));
        self
    }

    #[allow(non_snake_case)]
    pub fn whereGt(self, column: impl Into<String>, value: impl Into<String>) -> Self {
        self.where_gt(column, value)
    }

    /// Order by a column (Laravel `->orderBy('created_at', 'desc')`)
    pub fn order_by(mut self, column: impl Into<String>, direction: &str) -> Self {
        let dir = if direction.eq_ignore_ascii_case("desc") {
            OrderDir::Desc
        } else {
            OrderDir::Asc
        };
        self.order_by = Some((column.into(), dir));
        self
    }

    #[allow(non_snake_case)]
    pub fn orderBy(self, column: impl Into<String>, direction: &str) -> Self {
        self.order_by(column, direction)
    }

    /// Set limit (Laravel `->limit(10)` / `->take(10)`)
    pub fn limit(mut self, count: usize) -> Self {
        self.limit_val = Some(count);
        self
    }

    /// Set offset (Laravel `->offset(20)` / `->skip(20)`)
    pub fn offset(mut self, count: usize) -> Self {
        self.offset_val = Some(count);
        self
    }

    /// Convenient pagination helper (Laravel `->paginate(page, perPage)`)
    pub fn paginate(mut self, page: usize, per_page: usize) -> Self {
        let page_clamped = if page == 0 { 1 } else { page };
        self.limit_val = Some(per_page);
        self.offset_val = Some((page_clamped - 1) * per_page);
        self
    }

    /// Compile into parameterized SQL and binding values
    pub fn to_sql(&self) -> (String, Vec<String>) {
        let cols = if self.columns.is_empty() {
            "*".to_string()
        } else {
            self.columns.join(", ")
        };

        let mut sql = format!("SELECT {} FROM {}", cols, self.table_name);
        let mut bindings = Vec::new();

        if !self.wheres.is_empty() {
            let mut where_parts = Vec::new();
            for w in &self.wheres {
                match w {
                    WhereClause::Eq(c, v) => {
                        where_parts.push(format!("{} = ?", c));
                        bindings.push(v.clone());
                    }
                    WhereClause::Neq(c, v) => {
                        where_parts.push(format!("{} != ?", c));
                        bindings.push(v.clone());
                    }
                    WhereClause::Gt(c, v) => {
                        where_parts.push(format!("{} > ?", c));
                        bindings.push(v.clone());
                    }
                    WhereClause::Lt(c, v) => {
                        where_parts.push(format!("{} < ?", c));
                        bindings.push(v.clone());
                    }
                    WhereClause::Like(c, v) => {
                        where_parts.push(format!("{} LIKE ?", c));
                        bindings.push(v.clone());
                    }
                }
            }
            sql.push_str(" WHERE ");
            sql.push_str(&where_parts.join(" AND "));
        }

        if let Some((col, dir)) = &self.order_by {
            let dir_str = match dir {
                OrderDir::Asc => "ASC",
                OrderDir::Desc => "DESC",
            };
            sql.push_str(&format!(" ORDER BY {} {}", col, dir_str));
        }

        if let Some(limit) = self.limit_val {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = self.offset_val {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        (sql, bindings)
    }

    #[allow(non_snake_case)]
    pub fn toSql(&self) -> (String, Vec<String>) {
        self.to_sql()
    }
}
