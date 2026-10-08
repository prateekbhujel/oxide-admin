use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldType {
    Text,
    Email,
    Password,
    Number,
    Select { options: Vec<(String, String)> },
    SearchableSelect { options: Vec<(String, String)> },
    Textarea,
    Toggle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormField {
    pub name: String,
    pub label: String,
    pub field_type: FieldType,
    pub required: bool,
    pub placeholder: Option<String>,
}

impl FormField {
    pub fn text(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            label: crate::resource::headline(&name),
            name,
            field_type: FieldType::Text,
            required: false,
            placeholder: None,
        }
    }

    pub fn email(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            label: crate::resource::headline(&name),
            name,
            field_type: FieldType::Email,
            required: false,
            placeholder: None,
        }
    }

    pub fn select(name: impl Into<String>, options: Vec<(&str, &str)>) -> Self {
        let name = name.into();
        Self {
            label: crate::resource::headline(&name),
            name,
            field_type: FieldType::Select {
                options: options.into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
            },
            required: false,
            placeholder: None,
        }
    }

    pub fn searchable_select(name: impl Into<String>, options: Vec<(&str, &str)>) -> Self {
        let name = name.into();
        Self {
            label: crate::resource::headline(&name),
            name,
            field_type: FieldType::SearchableSelect {
                options: options.into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
            },
            required: false,
            placeholder: None,
        }
    }

    // Laravel-style camelCase alias
    #[allow(non_snake_case)]
    pub fn searchableSelect(name: impl Into<String>, options: Vec<(&str, &str)>) -> Self {
        Self::searchable_select(name, options)
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = Some(text.into());
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct Form {
    pub fields: Vec<FormField>,
}

impl Form {
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }

    pub fn field(mut self, field: FormField) -> Self {
        self.fields.push(field);
        self
    }
}
