#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditLog {
    pub id: String,
    pub user_name: String,
    pub action: String, // CREATE, UPDATE, DELETE
    pub resource: String,
    pub record_id: String,
    pub details: String,
    pub timestamp: String,
}
