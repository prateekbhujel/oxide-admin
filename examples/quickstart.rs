use axum::Router;
use oxide_admin::prelude::*;
use std::sync::{Arc, Mutex};

pub struct UserResource {
    repo: Arc<dyn UserRepository>,
}

impl UserResource {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self { repo }
    }
}

impl Resource for UserResource {
    fn name(&self) -> &str {
        "User"
    }

    fn plural_name(&self) -> &str {
        "Users"
    }

    fn slug(&self) -> &str {
        "users"
    }

    fn table(&self) -> Table {
        Table::new()
            .column(Column::text("id").label("ID").sortable())
            .column(Column::text("name").label("Name").searchable().sortable())
            .column(Column::text("email").label("Email").searchable())
            .column(Column::badge("role", vec![
                ("Founder", "amber"),
                ("Admin", "blue"),
                ("Member", "slate"),
            ]))
            .column(Column::badge("status", vec![
                ("Active", "emerald"),
                ("Suspended", "rose"),
            ]))
            .page_size(8)
    }

    fn form(&self) -> Form {
        Form::new()
            .field(FormField::text("name").required().placeholder("Full Name"))
            .field(FormField::email("email").required().placeholder("email@example.com"))
    }

    fn fetch_rows(&self, query: &QueryState) -> (Vec<RowData>, usize) {
        let (users, total) = self.repo.list(query);
        let rows = users
            .into_iter()
            .map(|u| {
                RowData::new(&u.id)
                    .insert("id", &u.id)
                    .insert("name", &u.name)
                    .insert("email", &u.email)
                    .insert("role", u.role.as_str())
                    .insert("status", u.status.as_str())
                    .insert("created_at", &u.created_at)
            })
            .collect();
        (rows, total)
    }

    fn delete_row(&self, id: &str) -> Result<(), String> {
        self.repo.delete(id)
    }
}

#[tokio::main]
async fn main() {
    let db_conn = Arc::new(Mutex::new(
        rusqlite::Connection::open("oxide.db").unwrap(),
    ));

    let user_repo = Arc::new(SqliteUserRepository::new(db_conn).unwrap());

    let admin = AdminPanel::new()
        .register(UserResource::new(user_repo));

    let app = Router::new()
        .nest("/admin", admin.into_router());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Quickstart running at http://127.0.0.1:3000/admin");
    axum::serve(listener, app).await.unwrap();
}
