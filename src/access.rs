pub struct User {
    pub name: String,
    pub password: String,
    pub role: u64
}
impl User {
    pub fn new(name: String, password: String, role: u64) -> Self {
        Self {
            name,
            password,
            role
        }
    }
}

pub struct Role {
    pub name: String,
    pub id: u64,
    pub permissions: Permissions
}

pub struct Permissions {
    pub read: Vec<u64>,
    pub write: Vec<u64>,
    pub edit_tables: bool,
    pub edit_roles: bool,
    pub edit_users: bool,
    pub edit_settings: bool
}