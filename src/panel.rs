use std::collections::HashMap;
use std::sync::Arc;
use axum::{
    extract::{Form, Path, Query},
    http::header::{COOKIE, SET_COOKIE},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use crate::auth;
use crate::domain::audit::AuditLog;
use crate::domain::user::{Role, User};
use crate::repository::{AuditRepository, UserRepository};
use crate::resource::{DynResource, QueryState, Resource};
use crate::view::{dialogs, layout, table_view};

#[derive(Clone, Default)]
pub struct AdminPanel {
    resources: Vec<DynResource>,
    resource_map: HashMap<String, DynResource>,
    user_repo: Option<Arc<dyn UserRepository>>,
    audit_repo: Option<Arc<dyn AuditRepository>>,
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
            user_repo: None,
            audit_repo: None,
        }
    }

    pub fn register<R: Resource + 'static>(mut self, resource: R) -> Self {
        let arc_res: DynResource = Arc::new(resource);
        self.resource_map.insert(arc_res.slug().to_string(), arc_res.clone());
        self.resources.push(arc_res);
        self
    }

    pub fn users(mut self, repo: Arc<dyn UserRepository>) -> Self {
        self.user_repo = Some(repo);
        self
    }

    pub fn audit(mut self, repo: Arc<dyn AuditRepository>) -> Self {
        self.audit_repo = Some(repo);
        self
    }

    pub fn into_router(self) -> Router {
        let shared_panel = Arc::new(self);

        Router::new()
            .route("/login", get(login_page).post({
                let panel = shared_panel.clone();
                move |form| login_submit(panel, form)
            }))
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

fn get_authenticated_user(panel: &AdminPanel, headers: &HeaderMap) -> Option<User> {
    let cookie_val = headers.get(COOKIE).and_then(|v| v.to_str().ok())?;

    for part in cookie_val.split(';') {
        let trimmed = part.trim();
        if let Some(user_part) = trimmed.strip_prefix("oxide_session=user:") {
            let email = user_part.trim();
            if let Some(ref repo) = panel.user_repo {
                if let Some(user) = repo.find_by_email(email) {
                    return Some(user);
                }
            }
            // Fallback: construct user based on demo email or default
            let role = if email.contains("bhujel") {
                Role::Superadmin
            } else if email.contains("lasta") {
                Role::Editor
            } else if email.contains("dharma") || email.contains("gokul") {
                Role::Admin
            } else {
                Role::Member
            };

            let name = if email.contains("bhujel") {
                "Pratik Bhujel".to_string()
            } else if email.contains("lasta") {
                "Lasta Chaudhary".to_string()
            } else if email.contains("dharma") {
                "Dharma Raj Shrestha".to_string()
            } else {
                "Ranjan Gumanju".to_string()
            };

            return Some(User {
                id: "1".into(),
                name,
                email: email.to_string(),
                role,
                status: crate::domain::UserStatus::Active,
                created_at: "2026-10-07".into(),
            });
        } else if trimmed == "oxide_session=authenticated" {
            return Some(User::default_admin());
        }
    }
    None
}

async fn login_page() -> Response {
    Html(auth::render_login_page(None)).into_response()
}

async fn login_submit(panel: Arc<AdminPanel>, Form(form): Form<LoginForm>) -> Response {
    let email = form.email.unwrap_or_default().trim().to_string();
    let password = form.password.unwrap_or_default().trim().to_string();

    let authenticated_user = if let Some(ref repo) = panel.user_repo {
        repo.find_by_email(&email)
    } else {
        None
    };

    let user_to_login = match authenticated_user {
        Some(u) if password == "admin123" => Some(u),
        _ => {
            if (email.contains("bhujel") || email.contains("admin")) && password == "admin123" {
                Some(User::default_admin())
            } else if password == "admin123" && !email.is_empty() {
                // Allow logging in with any of the demo emails
                let role = if email.contains("lasta") {
                    Role::Editor
                } else if email.contains("dharma") || email.contains("gokul") {
                    Role::Admin
                } else {
                    Role::Member
                };
                Some(User {
                    id: "demo".into(),
                    name: if email.contains("lasta") {
                        "Lasta Chaudhary".into()
                    } else if email.contains("dharma") {
                        "Dharma Raj Shrestha".into()
                    } else {
                        "Ranjan Gumanju".into()
                    },
                    email: email.clone(),
                    role,
                    status: crate::domain::UserStatus::Active,
                    created_at: "2026-10-07".into(),
                })
            } else {
                None
            }
        }
    };

    if let Some(user) = user_to_login {
        let mut response = Redirect::to("/admin").into_response();
        let cookie_str = format!(
            "oxide_session=user:{}; Path=/admin; HttpOnly; SameSite=Lax",
            user.email
        );
        response.headers_mut().insert(
            SET_COOKIE,
            cookie_str.parse().unwrap(),
        );
        response
    } else {
        Html(auth::render_login_page(Some("Invalid email or password. Demo password is: admin123"))).into_response()
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
    let user = match get_authenticated_user(&panel, &headers) {
        Some(u) => u,
        None => return Redirect::to("/admin/login").into_response(),
    };

    if let Some(first) = panel.resources.iter().find(|r| r.can_view(&user)) {
        Redirect::to(&format!("/admin/{}", first.slug())).into_response()
    } else {
        Html("<h1>403 Forbidden: No resources accessible for your role.</h1>").into_response()
    }
}

async fn resource_page(
    panel: Arc<AdminPanel>,
    headers: HeaderMap,
    Path(slug): Path<String>,
    Query(query_params): Query<TableQuery>,
) -> Response {
    let user = match get_authenticated_user(&panel, &headers) {
        Some(u) => u,
        None => return Redirect::to("/admin/login").into_response(),
    };

    let Some(res) = panel.resource_map.get(&slug) else {
        return (StatusCode::NOT_FOUND, Html("<h1>404 Resource Not Found</h1>")).into_response();
    };

    if !res.can_view(&user) {
        let forbidden_body = layout::render_forbidden_page(&user, res.name());
        let full_html = layout::render_page(
            "Access Restricted",
            &slug,
            &panel.resources,
            &forbidden_body,
            "",
            None,
            &user,
        );
        return (StatusCode::FORBIDDEN, Html(full_html)).into_response();
    }

    let flash = query_params.flash.clone();
    let query_state = parse_query(query_params);
    let table_html = table_view::render_table_partial(res.as_ref(), &query_state, &user);
    let dialogs_html = dialogs::render_dialogs(res.as_ref());
    let wrapped_content = format!(r#"<div id="table-wrapper">{table_html}</div>"#);

    let full_html = layout::render_page(
        res.plural_name(),
        &slug,
        &panel.resources,
        &wrapped_content,
        &dialogs_html,
        flash.as_deref(),
        &user,
    );

    Html(full_html).into_response()
}

async fn resource_table_partial(
    panel: Arc<AdminPanel>,
    headers: HeaderMap,
    Path(slug): Path<String>,
    Query(query_params): Query<TableQuery>,
) -> Response {
    let user = match get_authenticated_user(&panel, &headers) {
        Some(u) => u,
        None => return Redirect::to("/admin/login").into_response(),
    };

    let Some(res) = panel.resource_map.get(&slug) else {
        return (StatusCode::NOT_FOUND, Html("<h1>404 Resource Not Found</h1>")).into_response();
    };

    if !res.can_view(&user) {
        return (StatusCode::FORBIDDEN, Html(layout::render_forbidden_page(&user, res.name()))).into_response();
    }

    let query_state = parse_query(query_params);
    let table_html = table_view::render_table_partial(res.as_ref(), &query_state, &user);

    Html(table_html).into_response()
}

async fn resource_create(
    panel: Arc<AdminPanel>,
    headers: HeaderMap,
    Path(slug): Path<String>,
    Form(values): Form<HashMap<String, String>>,
) -> Response {
    let user = match get_authenticated_user(&panel, &headers) {
        Some(u) => u,
        None => return Redirect::to("/admin/login").into_response(),
    };

    let Some(res) = panel.resource_map.get(&slug) else {
        return (StatusCode::NOT_FOUND, Html("<h1>404 Resource Not Found</h1>")).into_response();
    };

    if !res.can_create(&user) {
        return (StatusCode::FORBIDDEN, Html(layout::render_forbidden_page(&user, res.name()))).into_response();
    }

    match res.create_row(values) {
        Ok(new_id) => {
            if let Some(ref audit) = panel.audit_repo {
                let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                let _ = audit.save(AuditLog {
                    id: String::new(),
                    user_name: user.name.clone(),
                    action: "CREATE".to_string(),
                    resource: slug.clone(),
                    record_id: new_id.clone(),
                    details: format!("Created new {} record #{}", res.name(), new_id),
                    timestamp: now,
                });
            }
            Redirect::to(&format!("/admin/{slug}?flash=Record+created+successfully")).into_response()
        }
        Err(err) => {
            Redirect::to(&format!("/admin/{slug}?flash=Error:+{}", err.replace(' ', "+"))).into_response()
        }
    }
}

async fn resource_update(
    panel: Arc<AdminPanel>,
    headers: HeaderMap,
    Path((slug, id)): Path<(String, String)>,
    Form(values): Form<HashMap<String, String>>,
) -> Response {
    let user = match get_authenticated_user(&panel, &headers) {
        Some(u) => u,
        None => return Redirect::to("/admin/login").into_response(),
    };

    let Some(res) = panel.resource_map.get(&slug) else {
        return (StatusCode::NOT_FOUND, Html("<h1>404 Resource Not Found</h1>")).into_response();
    };

    if !res.can_edit(&user) {
        return (StatusCode::FORBIDDEN, Html(layout::render_forbidden_page(&user, res.name()))).into_response();
    }

    match res.update_row(&id, values) {
        Ok(()) => {
            if let Some(ref audit) = panel.audit_repo {
                let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                let _ = audit.save(AuditLog {
                    id: String::new(),
                    user_name: user.name.clone(),
                    action: "UPDATE".to_string(),
                    resource: slug.clone(),
                    record_id: id.clone(),
                    details: format!("Updated {} record #{}", res.name(), id),
                    timestamp: now,
                });
            }
            Redirect::to(&format!("/admin/{slug}?flash=Record+updated+successfully")).into_response()
        }
        Err(err) => {
            Redirect::to(&format!("/admin/{slug}?flash=Error:+{}", err.replace(' ', "+"))).into_response()
        }
    }
}

async fn resource_delete(
    panel: Arc<AdminPanel>,
    headers: HeaderMap,
    Path((slug, id)): Path<(String, String)>,
) -> Response {
    let user = match get_authenticated_user(&panel, &headers) {
        Some(u) => u,
        None => return Redirect::to("/admin/login").into_response(),
    };

    let Some(res) = panel.resource_map.get(&slug) else {
        return (StatusCode::NOT_FOUND, Html("<h1>404 Resource Not Found</h1>")).into_response();
    };

    if !res.can_delete(&user) {
        return (StatusCode::FORBIDDEN, Html(layout::render_forbidden_page(&user, res.name()))).into_response();
    }

    match res.delete_row(&id) {
        Ok(()) => {
            if let Some(ref audit) = panel.audit_repo {
                let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                let _ = audit.save(AuditLog {
                    id: String::new(),
                    user_name: user.name.clone(),
                    action: "DELETE".to_string(),
                    resource: slug.clone(),
                    record_id: id.clone(),
                    details: format!("Deleted {} record #{}", res.name(), id),
                    timestamp: now,
                });
            }
            Redirect::to(&format!("/admin/{slug}?flash=Record+deleted+successfully")).into_response()
        }
        Err(err) => {
            Redirect::to(&format!("/admin/{slug}?flash=Error:+{}", err.replace(' ', "+"))).into_response()
        }
    }
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
