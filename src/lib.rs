pub mod form;
pub mod panel;
pub mod resource;
pub mod table;
pub mod view;

pub mod prelude {
    pub use crate::form::{Form, FormField};
    pub use crate::panel::AdminPanel;
    pub use crate::resource::{QueryState, Resource, RowData};
    pub use crate::table::{Column, Table, TableAction};
}
