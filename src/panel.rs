use std::collections::HashMap;
use std::sync::Arc;
use axum::{
    extract::{Form, Path, Query},
    http::header::{COOKIE, SET_COOKIE},
    http::HeaderMap,
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use crate::auth;
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
    pub flash: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub email: Option<String>,
    pub password: Option<String>,
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
            .route("/login", get(login_page).post(login_submit))
            .route("/logout", get(logout_handler))
            .route("/", get({
                let panel = shared_panel.clone();
                move |headers| root_redirect(panel, headers)
            }))
            .route("/:slug", get({
                let panel = shared_panel.clone();
                move |headers, path, query| resource_page(panel, headers, path, query)
            }))
            .route("/:slug/table", get({
                let panel = shared_panel.clone();
                move |headers, path, query| resource_table_partial(panel, headers, path, query)
            }))
            .route("/:slug/create", post({
                let panel = shared_panel.clone();
                move |headers, path, form| resource_create(panel, headers, path, form)
            }))
            .route("/:slug/edit/:id", post({
                let panel = shared_panel.clone();
                move |headers, path, form| resource_update(panel, headers, path, form)
            }))
            .route("/:slug/delete/:id", post({
                let panel = shared_panel.clone();
                move |headers, path| resource_delete(panel, headers, path)
            }))
    }
}

fn is_authenticated(headers: &HeaderMap) -> bool {
    if let Some(cookie_val) = headers.get(COOKIE).and_then(|v| v.to_str().ok()) {
        cookie_val.contains("oxide_session=authenticated")
    } else {
        false
    }
}

async fn login_page() -> Response {
    Html(auth::render_login_page(None)).into_response()
}

async fn login_submit(Form(form): Form<LoginForm>) -> Response {
    let email = form.email.unwrap_or_default();
    let password = form.password.unwrap_or_default();

    // Default admin check (supports email matching or admin credentials)
    if (email.contains("bhujel") || email.contains("admin")) && password == "admin123" {
        let mut response = Redirect::to("/admin").into_response();
        response.headers_mut().insert(
            SET_COOKIE,
            "oxide_session=authenticated; Path=/admin; HttpOnly; SameSite=Lax".parse().unwrap(),
        );
        response
    } else {
        Html(auth::render_login_page(Some("Invalid email or password. Use password: admin123"))).into_response()
    }
}

async fn logout_handler() -> Response {
    let mut response = Redirect::to("/admin/login").into_response();
    response.headers_mut().insert(
        SET_COOKIE,
        "oxide_session=; Path=/admin; Max-Age=0; HttpOnly".parse().unwrap(),
    );
    response
}

async fn root_redirect(panel: Arc<AdminPanel>, headers: HeaderMap) -> Response {
    if !is_authenticated(&headers) {
        return Redirect::to("/admin/login").into_response();
    }

    if let Some(first) = panel.resources.first() {
        Redirect::to(&format!("/admin/{}", first.slug())).into_response()
    } else {
        Html("<h1>No resources registered in AdminPanel.</h1>").into_response()
    }
}

async fn resource_page(
    panel: Arc<AdminPanel>,
    headers: HeaderMap,
    Path(slug): Path<String>,
    Query(query_params): Query<TableQuery>,
) -> Response {
    if !is_authenticated(&headers) {
        return Redirect::to("/admin/login").into_response();
    }

    let Some(res) = panel.resource_map.get(&slug) else {
        return Html("<h1>404 Resource Not Found</h1>").into_response();
    };

    let flash = query_params.flash.clone();
    let query_state = parse_query(query_params);
    let table_html = table_view::render_table_partial(res.as_ref(), &query_state);
    let wrapped_content = format!(r#"<div id="table-wrapper">{table_html}</div>"#);

    let full_html = layout::render_page(
        res.plural_name(),
        &slug,
        &panel.resources,
        &wrapped_content,
        flash.as_deref(),
    );

    Html(full_html).into_response()
}

async fn resource_table_partial(
    panel: Arc<AdminPanel>,
    headers: HeaderMap,
    Path(slug): Path<String>,
    Query(query_params): Query<TableQuery>,
) -> Response {
    if !is_authenticated(&headers) {
        return Redirect::to("/admin/login").into_response();
    }

    let Some(res) = panel.resource_map.get(&slug) else {
        return Html("<h1>404 Resource Not Found</h1>").into_response();
    };

    let query_state = parse_query(query_params);
    let table_html = table_view::render_table_partial(res.as_ref(), &query_state);

    Html(table_html).into_response()
}

async fn resource_create(
    panel: Arc<AdminPanel>,
    headers: HeaderMap,
    Path(slug): Path<String>,
    Form(values): Form<HashMap<String, String>>,
) -> Response {
    if !is_authenticated(&headers) {
        return Redirect::to("/admin/login").into_response();
    }

    if let Some(res) = panel.resource_map.get(&slug) {
        let _ = res.create_row(values);
    }
    Redirect::to(&format!("/admin/{slug}?flash=Record+created+successfully")).into_response()
}

async fn resource_update(
    panel: Arc<AdminPanel>,
    headers: HeaderMap,
    Path((slug, id)): Path<(String, String)>,
    Form(values): Form<HashMap<String, String>>,
) -> Response {
    if !is_authenticated(&headers) {
        return Redirect::to("/admin/login").into_response();
    }

    if let Some(res) = panel.resource_map.get(&slug) {
        let _ = res.update_row(&id, values);
    }
    Redirect::to(&format!("/admin/{slug}?flash=Record+updated+successfully")).into_response()
}

async fn resource_delete(
    panel: Arc<AdminPanel>,
    headers: HeaderMap,
    Path((slug, id)): Path<(String, String)>,
) -> Response {
    if !is_authenticated(&headers) {
        return Redirect::to("/admin/login").into_response();
    }

    if let Some(res) = panel.resource_map.get(&slug) {
        let _ = res.delete_row(&id);
    }
    Redirect::to(&format!("/admin/{slug}?flash=Record+deleted+successfully")).into_response()
}

fn parse_query(q: TableQuery) -> QueryState {
    QueryState {
        page: q.page.unwrap_or(1),
        per_page: 8,
        search: q.search.unwrap_or_default().trim().to_string(),
        sort_by: q.sort_by.filter(|s| !s.is_empty()),
        sort_desc: q.sort_desc.map(|v| v == "true").unwrap_or(false),
    }
}
