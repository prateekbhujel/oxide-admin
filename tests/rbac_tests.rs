use oxide_admin::prelude::*;

#[test]
fn test_role_hierarchy_and_permissions() {
    let superadmin_role = Role::Superadmin;
    assert!(superadmin_role.can("users.create"));
    assert!(superadmin_role.can("users.delete"));
    assert!(superadmin_role.can("audit.view"));
    assert!(superadmin_role.can("orders.delete"));

    let admin_role = Role::Admin;
    assert!(admin_role.can("users.create"));
    assert!(admin_role.can("users.edit"));
    assert!(!admin_role.can("users.delete"), "Admins cannot delete users");
    assert!(admin_role.can("orders.delete"));
    assert!(admin_role.can("audit.view"));

    let editor_role = Role::Editor;
    assert!(editor_role.can("users.view"));
    assert!(!editor_role.can("users.create"), "Editors cannot create users");
    assert!(!editor_role.can("users.edit"), "Editors cannot edit users");
    assert!(editor_role.can("orders.create"));
    assert!(editor_role.can("orders.edit"));
    assert!(!editor_role.can("orders.delete"), "Editors cannot delete orders");
    assert!(!editor_role.can("audit.view"), "Editors cannot view audit logs");

    let member_role = Role::Member;
    assert!(member_role.can("users.view"));
    assert!(member_role.can("orders.view"));
    assert!(!member_role.can("users.create"));
    assert!(!member_role.can("orders.create"));
    assert!(!member_role.can("orders.edit"));
    assert!(!member_role.can("orders.delete"));
    assert!(!member_role.can("audit.view"));
}

#[test]
fn test_user_status_guards() {
    let active_admin = User {
        id: "1".into(),
        name: "Active Admin".into(),
        email: "admin@example.com".into(),
        role: Role::Admin,
        status: UserStatus::Active,
        created_at: "2026-10-01".into(),
    };
    assert!(active_admin.can("users.create"));

    let suspended_admin = User {
        id: "2".into(),
        name: "Suspended Admin".into(),
        email: "suspended@example.com".into(),
        role: Role::Admin,
        status: UserStatus::Suspended,
        created_at: "2026-10-01".into(),
    };
    assert!(!suspended_admin.can("users.create"), "Suspended users cannot perform actions");

    let pending_admin = User {
        id: "3".into(),
        name: "Pending Admin".into(),
        email: "pending@example.com".into(),
        role: Role::Admin,
        status: UserStatus::Pending,
        created_at: "2026-10-01".into(),
    };
    assert!(!pending_admin.can("users.create"), "Pending users cannot perform actions");
}

#[test]
fn test_user_initials_and_helpers() {
    let u1 = User {
        id: "1".into(),
        name: "Hari Bahadur Bhujel".into(),
        email: "hari@example.com".into(),
        role: Role::Founder,
        status: UserStatus::Active,
        created_at: "2026-10-01".into(),
    };
    assert_eq!(u1.initials(), "HB");
    assert!(u1.is_superadmin());

    let u2 = User {
        id: "2".into(),
        name: "Pratik Bhujel".into(),
        email: "pratik@example.com".into(),
        role: Role::Superadmin,
        status: UserStatus::Active,
        created_at: "2026-10-01".into(),
    };
    assert_eq!(u2.initials(), "PB");
    assert!(u2.is_superadmin());

    let u3 = User {
        id: "3".into(),
        name: "Lasta Chaudhary".into(),
        email: "lasta@example.com".into(),
        role: Role::Editor,
        status: UserStatus::Active,
        created_at: "2026-10-01".into(),
    };
    assert_eq!(u3.initials(), "LC");
    assert!(!u3.is_superadmin());
}
