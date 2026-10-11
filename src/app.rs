//! Terminus Universal Application Engine
//!
//! Provides zero-friction, beginner-friendly application bootstrapping,
//! fluent route registration, and multi-platform service orchestration
//! without manual lifetime or concurrency boilerplate.

use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    routing::{delete, get, patch, post, put, MethodRouter},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

/// High-level, beginner-friendly Terminus Application Builder
#[derive(Clone)]
pub struct App {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub router: Router,
    pub cors_enabled: bool,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    /// Initialize a new Terminus application
    pub fn new() -> Self {
        Self {
            name: "Terminus Application".to_string(),
            host: "127.0.0.1".to_string(),
            port: 3000,
            router: Router::new(),
            cors_enabled: true,
        }
    }

    /// Alias for boot
    pub fn boot() -> Self {
        Self::new()
    }

    /// Set the human-readable application name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Set the listening port (default 3000)
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Set the host interface (default 127.0.0.1)
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }

    /// Register a raw Axum method router at a path
    pub fn route(mut self, path: &str, method_router: MethodRouter) -> Self {
        self.router = self.router.route(path, method_router);
        self
    }

    /// Register a GET endpoint
    pub fn get<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: axum::handler::Handler<T, ()>,
        T: 'static,
    {
        self.router = self.router.route(path, get(handler));
        self
    }

    /// Register a POST endpoint
    pub fn post<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: axum::handler::Handler<T, ()>,
        T: 'static,
    {
        self.router = self.router.route(path, post(handler));
        self
    }

    /// Register a PUT endpoint
    pub fn put<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: axum::handler::Handler<T, ()>,
        T: 'static,
    {
        self.router = self.router.route(path, put(handler));
        self
    }

    /// Register a PATCH endpoint
    pub fn patch<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: axum::handler::Handler<T, ()>,
        T: 'static,
    {
        self.router = self.router.route(path, patch(handler));
        self
    }

    /// Register a DELETE endpoint
    pub fn delete<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: axum::handler::Handler<T, ()>,
        T: 'static,
    {
        self.router = self.router.route(path, delete(handler));
        self
    }

    /// Mount the Terminus Admin Panel router seamlessly
    pub fn panel(mut self, panel: crate::panel::AdminPanel) -> Self {
        self.router = self.router.merge(panel.into_router());
        self
    }

    /// Merge any nested API sub-router
    pub fn api(mut self, prefix: &str, sub_router: Router) -> Self {
        self.router = self.router.nest(prefix, sub_router);
        self
    }

    /// Enable or disable CORS headers for mobile clients and frontend apps
    pub fn cors(mut self, enabled: bool) -> Self {
        self.cors_enabled = enabled;
        self
    }

    /// Finalize and build the Axum Router
    pub fn build(self) -> Router {
        let mut r = self.router;
        if self.cors_enabled {
            let cors = CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any);
            r = r.layer(cors);
        }
        r
    }

    /// Start the server and listen on the configured interface
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let addr: SocketAddr = format!("{}:{}", self.host, self.port).parse()?;
        println!("  Terminus Application: {}", self.name);
        println!("  Listening on: http://{}", addr);

        let router = self.build();
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, router).await?;
        Ok(())
    }
}

/// JSON HTTP response helper for beginner developer experience
pub fn json_response<T: serde::Serialize>(data: T) -> Response {
    match serde_json::to_string(&data) {
        Ok(body) => (
            StatusCode::OK,
            [("Content-Type", "application/json; charset=utf-8")],
            body,
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Serialization error: {}", err),
        )
            .into_response(),
    }
}

/// HTML HTTP response helper
pub fn html_response(html: impl Into<String>) -> Html<String> {
    Html(html.into())
}

/// Redirect helper
pub fn redirect_to(url: &str) -> Redirect {
    Redirect::to(url)
}
