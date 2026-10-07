use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Role {
    Founder,
    Superadmin,
    Admin,
    Editor,
    Member,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Founder => "Founder",
            Self::Superadmin => "Superadmin",
            Self::Admin => "Admin",
            Self::Editor => "Editor",
            Self::Member => "Member",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "founder" => Self::Founder,
            "superadmin" => Self::Superadmin,
            "admin" => Self::Admin,
            "editor" => Self::Editor,
            _ => Self::Member,
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserStatus {
    Active,
    Pending,
    Suspended,
}

impl UserStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "Active",
            Self::Pending => "Pending",
            Self::Suspended => "Suspended",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "pending" => Self::Pending,
            "suspended" => Self::Suspended,
            _ => Self::Active,
        }
    }
}

impl fmt::Display for UserStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: Role,
    pub status: UserStatus,
    pub created_at: String,
}
