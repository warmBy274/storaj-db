use little_collections::fn_map::FnMap;
use crate::*;

#[derive(Clone)]
pub struct User {
    pub name: String,
    pub id: u64,
    pub password: String,
    pub role: u64
}
impl User {
    pub fn new(name: String, id: u64, password: String, role: u64) -> Self {
        Self {
            name,
            id,
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
impl Role {
    pub fn new(name: String, id: u64, permissions: Permissions) -> Self {
        Self {
            name,
            id,
            permissions
        }
    }
    pub fn can_perform(&self, operation: &Operation) -> bool {
        match operation {
            Operation::GetRow(i, _) => {
                if self.permissions.read.get(*i).is_some() {true}
                else {false}
            }
            Operation::SetRow(i, _, _) |
            Operation::AddRow(i, _) |
            Operation::RemoveRow(i, _) => {
                if self.permissions.write.get(*i).is_some() {true}
                else {false}
            }
            Operation::AddUser(_, _, _) => {
                if self.permissions.edit_users {true}
                else {false}
            }
        }
    }
}

pub struct Permissions {
    pub read: FnMap<u64>,
    pub write: FnMap<u64>,
    pub edit_tables: bool,
    pub edit_roles: bool,
    pub edit_users: bool,
    pub edit_settings: bool
}
impl Permissions {
    pub fn new(
        read: Vec<u64>,
        write: Vec<u64>,
        edit_tables: bool,
        edit_roles: bool,
        edit_users: bool,
        edit_settings: bool
    ) -> Self {
        let mut read_map = FnMap::new(|table_id: &u64| *table_id as usize);
        for table in read {
            read_map.insert(table);
        }
        let mut write_map = FnMap::new(|table_id: &u64| *table_id as usize);
        for table in write {
            write_map.insert(table);
        }
        Self {
            read: read_map,
            write: write_map,
            edit_tables,
            edit_roles,
            edit_users,
            edit_settings
        }
    }
}