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
    pub fn permissions(&self) -> &'static [&'static str] {
        match self {
            Self::Founder | Self::Superadmin => &[
                "users.view",
                "users.create",
                "users.edit",
                "users.delete",
                "orders.view",
                "orders.create",
                "orders.edit",
                "orders.delete",
                "audit.view",
            ],
            Self::Admin => &[
                "users.view",
                "users.create",
                "users.edit",
                "orders.view",
                "orders.create",
                "orders.edit",
                "orders.delete",
                "audit.view",
            ],
            Self::Editor => &[
                "users.view",
                "orders.view",
                "orders.create",
                "orders.edit",
            ],
            Self::Member => &[
                "users.view",
                "orders.view",
            ],
        }
    }

    pub fn can(&self, permission: &str) -> bool {
        match self {
            Self::Founder | Self::Superadmin => true,
            _ => self.permissions().contains(&permission),
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

impl User {
    pub fn can(&self, permission: &str) -> bool {
        if self.status != UserStatus::Active {
            return false;
        }
        self.role.can(permission)
    }

    pub fn has_role(&self, role: &Role) -> bool {
        &self.role == role
    }

    pub fn is_superadmin(&self) -> bool {
        matches!(self.role, Role::Founder | Role::Superadmin)
    }

    pub fn initials(&self) -> String {
        let parts: Vec<&str> = self.name.split_whitespace().collect();
        match parts.len() {
            0 => "U".to_string(),
            1 => parts[0].chars().take(2).collect::<String>().to_uppercase(),
            _ => format!(
                "{}{}",
                parts[0].chars().next().unwrap_or('U'),
                parts.last().and_then(|p| p.chars().next()).unwrap_or(' ')
            ).to_uppercase(),
        }
    }

    pub fn default_admin() -> Self {
        Self {
            id: "2".into(),
            name: "Pratik Bhujel".into(),
            email: "pratik.bhujel@oxideadmin.dev".into(),
            role: Role::Superadmin,
            status: UserStatus::Active,
            created_at: "2026-09-02".into(),
        }
    }
}
