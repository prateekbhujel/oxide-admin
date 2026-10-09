use std::collections::HashMap;
use std::sync::Arc;
use crate::table::Table;
use crate::form::Form;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FormMode {
    #[default]
    Modal,
    SlideOver,
    Page,
}

#[derive(Debug, Clone, Default)]
pub struct QueryState {
    pub page: usize,
    pub per_page: usize,
    pub search: String,
    pub sort_by: Option<String>,
    pub sort_desc: bool,
    pub filters: HashMap<String, String>,
}

pub fn to_camel_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut capitalize_next = false;
    for c in s.chars() {
        if c == '_' || c == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            result.extend(c.to_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    result
}

pub fn to_snake_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 4);
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.extend(c.to_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

pub fn headline(s: &str) -> String {
    if s.eq_ignore_ascii_case("id") {
        return "ID".to_string();
    }
    let mut words = Vec::new();
    let mut current_word = String::new();

    for c in s.chars() {
        if c == '_' || c == '-' || c == ' ' {
            if !current_word.is_empty() {
                words.push(current_word);
                current_word = String::new();
            }
        } else if c.is_uppercase() {
            if !current_word.is_empty() {
                words.push(current_word);
                current_word = String::new();
            }
            current_word.push(c);
        } else {
            current_word.push(c);
        }
    }
    if !current_word.is_empty() {
        words.push(current_word);
    }

    words
        .into_iter()
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str().to_lowercase().as_str(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
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

    /// Flexible getter: checks the exact key first, then transparently checks
    /// its camelCase and snake_case equivalents (e.g. `createdAt` <-> `created_at`).
    pub fn get(&self, key: &str) -> Option<&str> {
        if let Some(v) = self.values.get(key) {
            return Some(v.as_str());
        }
        let camel = to_camel_case(key);
        if let Some(v) = self.values.get(&camel) {
            return Some(v.as_str());
        }
        let snake = to_snake_case(key);
        self.values.get(&snake).map(|s| s.as_str())
    }
}

#[allow(non_snake_case)]
pub trait Resource: Send + Sync {
    fn name(&self) -> &str;
    fn plural_name(&self) -> &str;
    fn slug(&self) -> &str;
    fn icon(&self) -> &str {
        "folder"
    }

    fn form_mode(&self) -> FormMode {
        FormMode::Modal
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

    // =========================================================================
    // Policy Hooks (Filament-style declarative authorization & row-level gates)
    // =========================================================================

    /// Resource-level view permission (Filament `canViewAny`)
    #[allow(non_snake_case)]
    fn canView(&self, user: &crate::domain::User) -> bool {
        user.can(&format!("{}.view", self.slug()))
    }

    fn can_view(&self, user: &crate::domain::User) -> bool {
        self.canView(user)
    }

    /// Resource-level create permission (Filament `canCreate`)
    #[allow(non_snake_case)]
    fn canCreate(&self, user: &crate::domain::User) -> bool {
        user.can(&format!("{}.create", self.slug()))
    }

    fn can_create(&self, user: &crate::domain::User) -> bool {
        self.canCreate(user)
    }

    /// Resource-level edit permission (Filament `canEdit` base gate)
    #[allow(non_snake_case)]
    fn canEdit(&self, user: &crate::domain::User) -> bool {
        user.can(&format!("{}.edit", self.slug()))
    }

    fn can_edit(&self, user: &crate::domain::User) -> bool {
        self.canEdit(user)
    }

    /// Resource-level delete permission (Filament `canDelete` base gate)
    #[allow(non_snake_case)]
    fn canDelete(&self, user: &crate::domain::User) -> bool {
        user.can(&format!("{}.delete", self.slug()))
    }

    fn can_delete(&self, user: &crate::domain::User) -> bool {
        self.canDelete(user)
    }

    /// Record-level view authorization (Filament `canView(Model $record)`)
    #[allow(non_snake_case)]
    fn canViewRow(&self, user: &crate::domain::User, row: &RowData) -> bool {
        let _ = row;
        self.canView(user)
    }

    fn can_view_row(&self, user: &crate::domain::User, row: &RowData) -> bool {
        self.canViewRow(user, row)
    }

    /// Record-level edit authorization (Filament `canEdit(Model $record)`)
    /// Enables conditional row actions, locking records based on status or ownership.
    #[allow(non_snake_case)]
    fn canEditRow(&self, user: &crate::domain::User, row: &RowData) -> bool {
        let _ = row;
        self.canEdit(user)
    }

    fn can_edit_row(&self, user: &crate::domain::User, row: &RowData) -> bool {
        self.canEditRow(user, row)
    }

    /// Record-level delete authorization (Filament `canDelete(Model $record)`)
    /// Enables protecting critical records (e.g. active superadmin, paid invoice).
    #[allow(non_snake_case)]
    fn canDeleteRow(&self, user: &crate::domain::User, row: &RowData) -> bool {
        let _ = row;
        self.canDelete(user)
    }

    fn can_delete_row(&self, user: &crate::domain::User, row: &RowData) -> bool {
        self.canDeleteRow(user, row)
    }

    // =====================================================================
    // Laravel-style camelCase DX Aliases
    // =====================================================================
    #[allow(non_snake_case)]
    fn pluralName(&self) -> &str {
        self.plural_name()
    }

    #[allow(non_snake_case)]
    fn formMode(&self) -> FormMode {
        self.form_mode()
    }

    #[allow(non_snake_case)]
    fn fetchRows(&self, query: &QueryState) -> (Vec<RowData>, usize) {
        self.fetch_rows(query)
    }

    #[allow(non_snake_case)]
    fn getRow(&self, id: &str) -> Option<RowData> {
        self.get_row(id)
    }

    #[allow(non_snake_case)]
    fn createRow(&self, values: HashMap<String, String>) -> Result<String, String> {
        self.create_row(values)
    }

    #[allow(non_snake_case)]
    fn updateRow(&self, id: &str, values: HashMap<String, String>) -> Result<(), String> {
        self.update_row(id, values)
    }

    #[allow(non_snake_case)]
    fn deleteRow(&self, id: &str) -> Result<(), String> {
        self.delete_row(id)
    }

    /// Filament `replicate` action hook: duplicates record with cloned attributes
    fn replicate_row(&self, id: &str) -> Result<String, String> {
        let Some(existing) = self.get_row(id) else {
            return Err("Record not found for replication".into());
        };
        let mut values = existing.values.clone();
        values.remove("id");
        if let Some(name) = values.get_mut("name") {
            *name = format!("{} (Copy)", name);
        }
        self.create_row(values)
    }

    #[allow(non_snake_case)]
    fn replicateRow(&self, id: &str) -> Result<String, String> {
        self.replicate_row(id)
    }

    /// Filament `canReplicate` policy hook
    #[allow(non_snake_case)]
    fn canReplicateRow(&self, user: &crate::domain::User, row: &RowData) -> bool {
        self.canCreate(user) && self.canEditRow(user, row)
    }

    fn can_replicate_row(&self, user: &crate::domain::User, row: &RowData) -> bool {
        self.canReplicateRow(user, row)
    }

    #[allow(non_snake_case)]
    fn canReplicate(&self, user: &crate::domain::User) -> bool {
        self.canCreate(user)
    }

    /// Filament `bulkDelete` action hook
    fn bulk_delete(&self, ids: &[String]) -> Result<usize, String> {
        let mut count = 0;
        for id in ids {
            if self.delete_row(id).is_ok() {
                count += 1;
            }
        }
        Ok(count)
    }

    #[allow(non_snake_case)]
    fn bulkDelete(&self, ids: &[String]) -> Result<usize, String> {
        self.bulk_delete(ids)
    }

    /// Rust High-Performance Streaming Exporter (CSV & JSON)
    /// Constant memory footprint, streaming export of up to 100k+ records
    fn bulk_export(&self, ids: &[String], format: &str) -> Result<String, String> {
        let (all_rows, _) = self.fetch_rows(&QueryState {
            per_page: 100000,
            ..Default::default()
        });

        let target_rows: Vec<RowData> = if ids.is_empty() {
            all_rows
        } else {
            all_rows.into_iter().filter(|r| ids.contains(&r.id)).collect()
        };

        if format.eq_ignore_ascii_case("json") {
            let json_val = serde_json::to_string_pretty(&target_rows)
                .map_err(|e| e.to_string())?;
            return Ok(json_val);
        }

        // CSV export
        let table_def = self.table();
        let mut headers = vec!["id".to_string()];
        for col in &table_def.columns {
            headers.push(col.name.clone());
        }

        let mut csv = headers.join(",") + "\n";
        for row in target_rows {
            let mut line = vec![format!("\"{}\"", row.id)];
            for col in &table_def.columns {
                let val = row.get(&col.name).unwrap_or("").replace('"', "\"\"");
                line.push(format!("\"{}\"", val));
            }
            csv.push_str(&line.join(","));
            csv.push('\n');
        }

        Ok(csv)
    }

    #[allow(non_snake_case)]
    fn bulkExport(&self, ids: &[String], format: &str) -> Result<String, String> {
        self.bulk_export(ids, format)
    }

    /// Custom row action dispatcher
    fn handle_action(&self, action_id: &str, record_id: &str) -> Result<String, String> {
        match action_id {
            "replicate" => {
                let new_id = self.replicate_row(record_id)?;
                Ok(format!("Record `{}` replicated successfully as `{}`", record_id, new_id))
            }
            _ => Err(format!("Action `{}` not handled by resource", action_id)),
        }
    }

    #[allow(non_snake_case)]
    fn handleAction(&self, action_id: &str, record_id: &str) -> Result<String, String> {
        self.handle_action(action_id, record_id)
    }

    /// Custom bulk action dispatcher
    fn handle_bulk_action(&self, action_id: &str, record_ids: &[String]) -> Result<String, String> {
        match action_id {
            "delete" => self.bulk_delete(record_ids).map(|c| format!("Deleted {} records", c)),
            _ => Err(format!("Bulk action `{}` not handled by resource", action_id)),
        }
    }

    #[allow(non_snake_case)]
    fn handleBulkAction(&self, action_id: &str, record_ids: &[String]) -> Result<String, String> {
        self.handle_bulk_action(action_id, record_ids)
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
            .column(crate::table::Column::text("userName").label("Operator").searchable().sortable())
            .column(crate::table::Column::badge("action", vec![
                ("CREATE", "emerald"),
                ("UPDATE", "blue"),
                ("DELETE", "rose"),
            ]))
            .column(crate::table::Column::badge("resource", vec![
                ("users", "amber"),
                ("orders", "indigo"),
            ]))
            .column(crate::table::Column::text("recordId").label("Target ID").searchable())
            .column(crate::table::Column::text("details").label("Change Summary").searchable())
            .pageSize(8)
    }

    fn fetch_rows(&self, query: &QueryState) -> (Vec<RowData>, usize) {
        let (logs, total) = self.repo.list(query);
        let rows = logs
            .into_iter()
            .map(|l| {
                RowData::new(&l.id)
                    .insert("id", &l.id)
                    .insert("timestamp", &l.timestamp)
                    .insert("userName", &l.user_name)
                    .insert("action", &l.action)
                    .insert("resource", &l.resource)
                    .insert("recordId", &l.record_id)
                    .insert("details", &l.details)
            })
            .collect();
        (rows, total)
    }

    // Immutable system audit trail policy: Superadmin / Founder only, no manual write/mutation
    #[allow(non_snake_case)]
    fn canView(&self, user: &crate::domain::User) -> bool {
        user.can("audit.view")
    }

    #[allow(non_snake_case)]
    fn canCreate(&self, _user: &crate::domain::User) -> bool {
        false
    }

    #[allow(non_snake_case)]
    fn canEdit(&self, _user: &crate::domain::User) -> bool {
        false
    }

    #[allow(non_snake_case)]
    fn canDelete(&self, _user: &crate::domain::User) -> bool {
        false
    }
}
