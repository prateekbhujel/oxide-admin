pub mod auth;
pub mod cache;
pub mod debug;
pub mod domain;
pub mod form;
pub mod mail;
pub mod panel;
pub mod query;
pub mod queue;
pub mod repository;
pub mod resource;
pub mod table;
pub mod testing;
pub mod theme;
pub mod view;
pub mod ws;

pub mod prelude {
    pub use crate::cache::{CacheDriver, MemoryCache, RedisCache};
    pub use crate::domain::{AuditLog, Order, OrderStatus, Role, User, UserStatus};
    pub use crate::form::{FieldType, Form, FormField, FormSection, Section};
    pub use crate::mail::{MailMessage, Mailer};
    pub use crate::panel::AdminPanel;
    pub use crate::query::{OrderDir, Query, WhereClause};
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
    pub use crate::table::{Action, BulkAction, Column, ColumnType, Table, TableAction, TableFilter, TableStyle};
    pub use crate::testing::{expect, test as pest_test, Expectation};
    pub use crate::theme::{FontFamily, PrimaryColor, ThemeConfig};
    pub use crate::ws::{BroadcastEvent, Broadcaster};
}


