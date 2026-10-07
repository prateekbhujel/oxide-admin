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
}

pub type DynResource = Arc<dyn Resource>;
