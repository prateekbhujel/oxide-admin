pub mod auth;
pub mod domain;
pub mod form;
pub mod panel;
pub mod repository;
pub mod resource;
pub mod table;
pub mod view;

pub mod prelude {
    pub use crate::domain::{AuditLog, Order, OrderStatus, Role, User, UserStatus};
    pub use crate::form::{Form, FormField};
    pub use crate::panel::AdminPanel;
    pub use crate::repository::{
        AuditRepository, InMemoryAuditRepository, InMemoryOrderRepository, InMemoryUserRepository,
        OrderRepository, SqliteAuditRepository, SqliteOrderRepository, SqliteUserRepository,
        UserRepository,
    };
    pub use crate::resource::{AuditLogResource, FormMode, QueryState, Resource, RowData};
    pub use crate::table::{Column, Table, TableAction, TableFilter, TableStyle};
}
