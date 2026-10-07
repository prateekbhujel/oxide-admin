pub mod auth;
pub mod domain;
pub mod form;
pub mod panel;
pub mod repository;
pub mod resource;
pub mod table;
pub mod view;

pub mod prelude {
    pub use crate::domain::{Order, OrderStatus, Role, User, UserStatus};
    pub use crate::form::{Form, FormField};
    pub use crate::panel::AdminPanel;
    pub use crate::repository::{
        InMemoryOrderRepository, InMemoryUserRepository, OrderRepository, UserRepository,
    };
    pub use crate::resource::{QueryState, Resource, RowData};
    pub use crate::table::{Column, Table, TableAction};
}
