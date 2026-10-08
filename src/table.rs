use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColumnType {
    Text,
    Badge { color_map: Vec<(String, String)> },
    DateTime,
    Numeric,
}

#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub label: String,
    pub column_type: ColumnType,
    pub sortable: bool,
    pub searchable: bool,
}

impl Column {
    pub fn text(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            label: crate::resource::headline(&name),
            name,
            column_type: ColumnType::Text,
            sortable: false,
            searchable: false,
        }
    }

    pub fn badge(name: impl Into<String>, colors: Vec<(&str, &str)>) -> Self {
        let name = name.into();
        Self {
            label: crate::resource::headline(&name),
            name,
            column_type: ColumnType::Badge {
                color_map: colors.into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
            },
            sortable: false,
            searchable: false,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn sortable(mut self) -> Self {
        self.sortable = true;
        self
    }

    pub fn searchable(mut self) -> Self {
        self.searchable = true;
        self
    }
}

#[derive(Debug, Clone)]
pub struct TableAction {
    pub id: String,
    pub label: String,
    pub style: ActionStyle,
    pub confirmation: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ActionStyle {
    Default,
    Primary,
    Danger,
}

impl TableAction {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            style: ActionStyle::Default,
            confirmation: None,
        }
    }

    pub fn danger(mut self) -> Self {
        self.style = ActionStyle::Danger;
        self
    }

    pub fn confirm(mut self, message: impl Into<String>) -> Self {
        self.confirmation = Some(message.into());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TableStyle {
    #[default]
    Default,
    Striped,
    Compact,
}

#[derive(Debug, Clone)]
pub struct TableFilter {
    pub name: String,
    pub label: String,
    pub options: Vec<(String, String)>,
}

impl TableFilter {
    pub fn select(name: impl Into<String>, label: impl Into<String>, options: Vec<(&str, &str)>) -> Self {
        Self {
            name: name.into(),
            label: label.into(),
            options: options.into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Table {
    pub columns: Vec<Column>,
    pub actions: Vec<TableAction>,
    pub filters: Vec<TableFilter>,
    pub style: TableStyle,
    pub default_sort_by: Option<String>,
    pub default_page_size: usize,
}

impl Table {
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            actions: Vec::new(),
            filters: Vec::new(),
            style: TableStyle::Default,
            default_sort_by: None,
            default_page_size: 10,
        }
    }

    pub fn column(mut self, column: Column) -> Self {
        self.columns.push(column);
        self
    }

    pub fn action(mut self, action: TableAction) -> Self {
        self.actions.push(action);
        self
    }

    pub fn filter(mut self, filter: TableFilter) -> Self {
        self.filters.push(filter);
        self
    }

    pub fn style(mut self, style: TableStyle) -> Self {
        self.style = style;
        self
    }

    pub fn striped(mut self) -> Self {
        self.style = TableStyle::Striped;
        self
    }

    pub fn compact(mut self) -> Self {
        self.style = TableStyle::Compact;
        self
    }

    pub fn page_size(mut self, size: usize) -> Self {
        self.default_page_size = size;
        self
    }

    // Laravel-style camelCase aliases
    #[allow(non_snake_case)]
    pub fn pageSize(self, size: usize) -> Self {
        self.page_size(size)
    }

    #[allow(non_snake_case)]
    pub fn defaultSortBy(mut self, sort_by: impl Into<String>) -> Self {
        self.default_sort_by = Some(sort_by.into());
        self
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}
