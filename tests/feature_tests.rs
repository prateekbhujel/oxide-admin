use oxide_admin::prelude::*;
use std::collections::HashMap;

struct TestModalResource;
impl Resource for TestModalResource {
    fn name(&self) -> &str { "Item" }
    fn plural_name(&self) -> &str { "Items" }
    fn slug(&self) -> &str { "items" }
    fn table(&self) -> Table { Table::new() }
    fn form(&self) -> Form { Form::new() }
    fn fetch_rows(&self, _query: &QueryState) -> (Vec<RowData>, usize) { (vec![], 0) }
}

struct TestPageResource;
impl Resource for TestPageResource {
    fn name(&self) -> &str { "Article" }
    fn plural_name(&self) -> &str { "Articles" }
    fn slug(&self) -> &str { "articles" }
    fn form_mode(&self) -> FormMode { FormMode::Page }
    fn table(&self) -> Table {
        Table::new()
            .striped()
            .filter(TableFilter::select("status", "Status", vec![("draft", "Draft"), ("published", "Published")]))
    }
    fn form(&self) -> Form {
        Form::new()
            .field(FormField::searchable_select("category", vec![("tech", "Technology"), ("life", "Lifestyle")]))
    }
    fn fetch_rows(&self, _query: &QueryState) -> (Vec<RowData>, usize) { (vec![], 0) }
}

#[test]
fn test_form_mode_defaults_and_customization() {
    let modal_res = TestModalResource;
    assert_eq!(modal_res.form_mode(), FormMode::Modal);

    let page_res = TestPageResource;
    assert_eq!(page_res.form_mode(), FormMode::Page);
}

#[test]
fn test_table_styles_and_filters() {
    let table = Table::new()
        .striped()
        .compact()
        .filter(TableFilter::select("role", "Role", vec![("admin", "Admin"), ("user", "User")]));

    assert_eq!(table.style, TableStyle::Compact);
    assert_eq!(table.filters.len(), 1);
    assert_eq!(table.filters[0].name, "role");
    assert_eq!(table.filters[0].label, "Role");
    assert_eq!(table.filters[0].options.len(), 2);
}

#[test]
fn test_searchable_select_form_field() {
    let field = FormField::searchable_select(
        "assignee",
        vec![("1", "Pratik Bhujel"), ("2", "Dharma Raj"), ("3", "Lasta Chaudhary")],
    );

    assert_eq!(field.name, "assignee");
    assert_eq!(field.label, "Assignee");
    match field.field_type {
        oxide_admin::form::FieldType::SearchableSelect { options } => {
            assert_eq!(options.len(), 3);
            assert_eq!(options[0], ("1".to_string(), "Pratik Bhujel".to_string()));
        }
        _ => panic!("Expected SearchableSelect field type"),
    }
}

#[test]
fn test_query_state_with_filters() {
    let mut filters = HashMap::new();
    filters.insert("role".to_string(), "Superadmin".to_string());
    filters.insert("status".to_string(), "Active".to_string());

    let query = QueryState {
        page: 1,
        per_page: 8,
        search: "pratik".to_string(),
        sort_by: Some("name".to_string()),
        sort_desc: false,
        filters,
    };

    assert_eq!(query.filters.get("role").unwrap(), "Superadmin");
    assert_eq!(query.filters.get("status").unwrap(), "Active");
}
