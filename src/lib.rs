pub mod auth;
pub mod domain;
pub mod form;
pub mod mail;
pub mod panel;
pub mod queue;
pub mod repository;
pub mod resource;
pub mod table;
pub mod theme;
pub mod view;

pub mod prelude {
    pub use crate::domain::{AuditLog, Order, OrderStatus, Role, User, UserStatus};
    pub use crate::form::{Form, FormField};
    pub use crate::mail::{MailMessage, Mailer};
    pub use crate::panel::AdminPanel;
    pub use crate::queue::{Job, JobQueue, QueueStats};
    pub use crate::repository::{
        AuditRepository, InMemoryAuditRepository, InMemoryOrderRepository, InMemoryUserRepository,
        OrderRepository, SqliteAuditRepository, SqliteOrderRepository, SqliteUserRepository,
        UserRepository,
    };
    pub use crate::resource::{
        headline, to_camel_case, to_snake_case, AuditLogResource, FormMode, QueryState, Resource,
        RowData,
    };
    pub use crate::table::{Column, Table, TableAction, TableFilter, TableStyle};
    pub use crate::theme::{FontFamily, PrimaryColor, ThemeConfig};
}

