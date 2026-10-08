use std::collections::HashMap;
use std::sync::Arc;
use crate::table::Table;
use crate::form::Form;

#[derive(Debug, Clone, Default)]
pub struct QueryState {
    pub page: usize,
    pub per_page: usize,
    pub search: String,
    pub sort_by: Option<String>,
    pub sort_desc: bool,
}

#[derive(Debug, Clone)]
pub struct RowData {
    pub id: String,
    pub values: HashMap<String, String>,
}

impl RowData {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            values: HashMap::new(),
        }
    }

    pub fn insert(mut self, key: impl Into<String>, val: impl Into<String>) -> Self {
        self.values.insert(key.into(), val.into());
        self
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(|s| s.as_str())
    }
}

pub trait Resource: Send + Sync {
    fn name(&self) -> &str;
    fn plural_name(&self) -> &str;
    fn slug(&self) -> &str;
    fn icon(&self) -> &str {
        "folder"
    }

    fn table(&self) -> Table;

    fn form(&self) -> Form {
        Form::new()
    }

    fn fetch_rows(&self, query: &QueryState) -> (Vec<RowData>, usize);

    fn get_row(&self, _id: &str) -> Option<RowData> {
        None
    }

    fn create_row(&self, _values: HashMap<String, String>) -> Result<String, String> {
        Err("Create not supported for this resource".into())
    }

    fn update_row(&self, _id: &str, _values: HashMap<String, String>) -> Result<(), String> {
        Err("Update not supported for this resource".into())
    }

    fn delete_row(&self, _id: &str) -> Result<(), String> {
        Err("Delete not supported for this resource".into())
    }

    // Policy Hooks (Declarative RBAC & Gate authorization)
    fn can_view(&self, user: &crate::domain::User) -> bool {
        user.can(&format!("{}.view", self.slug()))
    }

    fn can_create(&self, user: &crate::domain::User) -> bool {
        user.can(&format!("{}.create", self.slug()))
    }

    fn can_edit(&self, user: &crate::domain::User) -> bool {
        user.can(&format!("{}.edit", self.slug()))
    }

    fn can_delete(&self, user: &crate::domain::User) -> bool {
        user.can(&format!("{}.delete", self.slug()))
    }
}

pub type DynResource = Arc<dyn Resource>;

/// Built-in immutable Audit Log Resource
pub struct AuditLogResource {
    repo: Arc<dyn crate::repository::AuditRepository>,
}

impl AuditLogResource {
    pub fn new(repo: Arc<dyn crate::repository::AuditRepository>) -> Self {
        Self { repo }
    }
}

impl Resource for AuditLogResource {
    fn name(&self) -> &str {
        "Audit Log"
    }

    fn plural_name(&self) -> &str {
        "Audit Trail"
    }

    fn slug(&self) -> &str {
        "audit"
    }

    fn icon(&self) -> &str {
        "shield"
    }

    fn table(&self) -> Table {
        Table::new()
            .column(crate::table::Column::text("timestamp").label("Timestamp").sortable())
            .column(crate::table::Column::text("user_name").label("Operator").searchable().sortable())
            .column(crate::table::Column::badge("action", vec![
                ("CREATE", "emerald"),
                ("UPDATE", "blue"),
                ("DELETE", "rose"),
            ]))
            .column(crate::table::Column::badge("resource", vec![
                ("users", "amber"),
                ("orders", "indigo"),
            ]))
            .column(crate::table::Column::text("record_id").label("Target ID").searchable())
            .column(crate::table::Column::text("details").label("Change Summary").searchable())
            .page_size(8)
    }

    fn fetch_rows(&self, query: &QueryState) -> (Vec<RowData>, usize) {
        let (logs, total) = self.repo.list(query);
        let rows = logs
            .into_iter()
            .map(|l| {
                RowData::new(&l.id)
                    .insert("id", &l.id)
                    .insert("timestamp", &l.timestamp)
                    .insert("user_name", &l.user_name)
                    .insert("action", &l.action)
                    .insert("resource", &l.resource)
                    .insert("record_id", &l.record_id)
                    .insert("details", &l.details)
            })
            .collect();
        (rows, total)
    }

    // Immutable system audit trail policy: Superadmin / Founder only, no manual write/mutation
    fn can_view(&self, user: &crate::domain::User) -> bool {
        user.can("audit.view")
    }

    fn can_create(&self, _user: &crate::domain::User) -> bool {
        false
    }

    fn can_edit(&self, _user: &crate::domain::User) -> bool {
        false
    }

    fn can_delete(&self, _user: &crate::domain::User) -> bool {
        false
    }
}
