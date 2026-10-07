use std::collections::HashMap;
use std::sync::Arc;
use axum::{
    extract::{Path, Query},
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
    Router,
};
use serde::Deserialize;
use crate::resource::{DynResource, QueryState, Resource};
use crate::view::{layout, table_view};

#[derive(Clone, Default)]
pub struct AdminPanel {
    resources: Vec<DynResource>,
    resource_map: HashMap<String, DynResource>,
}

#[derive(Debug, Deserialize)]
pub struct TableQuery {
    pub page: Option<usize>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub sort_desc: Option<String>,
}

impl AdminPanel {
    pub fn new() -> Self {
        Self {
            resources: Vec::new(),
            resource_map: HashMap::new(),
        }
    }

    pub fn register<R: Resource + 'static>(mut self, resource: R) -> Self {
        let arc_res: DynResource = Arc::new(resource);
        self.resource_map.insert(arc_res.slug().to_string(), arc_res.clone());
        self.resources.push(arc_res);
        self
    }

    pub fn into_router(self) -> Router {
        let shared_panel = Arc::new(self);

        Router::new()
            .route("/", get({
                let panel = shared_panel.clone();
                move || root_redirect(panel)
            }))
            .route("/:slug", get({
                let panel = shared_panel.clone();
                move |path, query| resource_page(panel, path, query)
            }))
            .route("/:slug/table", get({
                let panel = shared_panel.clone();
                move |path, query| resource_table_partial(panel, path, query)
            }))
    }
}

async fn root_redirect(panel: Arc<AdminPanel>) -> Response {
    if let Some(first) = panel.resources.first() {
        Redirect::to(&format!("/admin/{}", first.slug())).into_response()
    } else {
        Html("<h1>No resources registered in AdminPanel.</h1>").into_response()
    }
}

async fn resource_page(
    panel: Arc<AdminPanel>,
    Path(slug): Path<String>,
    Query(query_params): Query<TableQuery>,
) -> Response {
    let Some(res) = panel.resource_map.get(&slug) else {
        return Html("<h1>404 Resource Not Found</h1>").into_response();
    };

    let query_state = parse_query(query_params);
    let table_html = table_view::render_table_partial(res.as_ref(), &query_state);
    let wrapped_content = format!(r#"<div id="table-wrapper">{table_html}</div>"#);

    let full_html = layout::render_page(
        res.plural_name(),
        &slug,
        &panel.resources,
        &wrapped_content,
    );

    Html(full_html).into_response()
}

async fn resource_table_partial(
    panel: Arc<AdminPanel>,
    Path(slug): Path<String>,
    Query(query_params): Query<TableQuery>,
) -> Response {
    let Some(res) = panel.resource_map.get(&slug) else {
        return Html("<h1>404 Resource Not Found</h1>").into_response();
    };

    let query_state = parse_query(query_params);
    let table_html = table_view::render_table_partial(res.as_ref(), &query_state);

    Html(table_html).into_response()
}

fn parse_query(q: TableQuery) -> QueryState {
    QueryState {
        page: q.page.unwrap_or(1),
        per_page: 10,
        search: q.search.unwrap_or_default().trim().to_string(),
        sort_by: q.sort_by.filter(|s| !s.is_empty()),
        sort_desc: q.sort_desc.map(|v| v == "true").unwrap_or(false),
    }
}
