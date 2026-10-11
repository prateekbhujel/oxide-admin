//! Terminus Module — Enterprise Modular Architecture (DDD)
//! Encapsulated domain modules containing DTOs, Models, Repositories, Providers, Controllers, Requests, and Routes.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::resource::Resource;

/// Core interface for an encapsulated domain module
pub trait AppModule: Send + Sync {
    /// Human-readable module name (e.g. "User Management", "Billing")
    fn name(&self) -> &str;

    /// URL/route slug identifier (e.g. "users", "billing")
    fn slug(&self) -> &str;

    /// Optional module description
    fn description(&self) -> Option<&str> {
        None
    }

    /// Axum Router for this module's custom API/web endpoints
    fn routes(&self) -> axum::Router {
        axum::Router::new()
    }

    /// Optional Terminus administrative Resource for this module
    fn resource(&self) -> Option<Arc<dyn Resource>> {
        None
    }
}

/// Standardized enterprise JSON API response envelope
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<String>>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            message: None,
            data: Some(data),
            errors: None,
        }
    }

    pub fn with_message(data: T, message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: Some(message.into()),
            data: Some(data),
            errors: None,
        }
    }
}

impl ApiResponse<()> {
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: Some(message.into()),
            data: None,
            errors: None,
        }
    }

    pub fn validation_errors(errors: Vec<String>) -> Self {
        Self {
            success: false,
            message: Some("Validation failed".to_string()),
            data: None,
            errors: Some(errors),
        }
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        let status = if self.success {
            StatusCode::OK
        } else if self.errors.is_some() {
            StatusCode::UNPROCESSABLE_ENTITY
        } else {
            StatusCode::BAD_REQUEST
        };
        (status, Json(self)).into_response()
    }
}

/// Validation contract for enterprise Form Request objects
pub trait ValidatedRequest: Sized {
    fn validate(&self) -> Result<(), Vec<String>>;
}

/// Dynamic module wrapper allowing programmatic module construction
#[derive(Clone)]
pub struct DynamicModule {
    name: String,
    slug: String,
    description: Option<String>,
    routes: axum::Router,
    resource: Option<Arc<dyn Resource>>,
}

impl DynamicModule {
    pub fn new(name: impl Into<String>, slug: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            slug: slug.into(),
            description: None,
            routes: axum::Router::new(),
            resource: None,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn with_routes(mut self, router: axum::Router) -> Self {
        self.routes = router;
        self
    }

    pub fn with_resource(mut self, res: Arc<dyn Resource>) -> Self {
        self.resource = Some(res);
        self
    }
}

impl AppModule for DynamicModule {
    fn name(&self) -> &str {
        &self.name
    }

    fn slug(&self) -> &str {
        &self.slug
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn routes(&self) -> axum::Router {
        self.routes.clone()
    }

    fn resource(&self) -> Option<Arc<dyn Resource>> {
        self.resource.clone()
    }
}
