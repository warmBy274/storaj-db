use crate::*;

pub enum GetRowError {
    NoPermission,
    NotFound
}
impl Serializable for GetRowError {
    fn serialize(&self) -> Vec<u8> {
        match self {
            Self::NoPermission => vec![0],
            Self::NotFound => vec![1]
        }
    }
}

pub enum SetRowError {
    NoPermission,
    NotFound,
    InappropriateRows
}
impl Serializable for SetRowError {
    fn serialize(&self) -> Vec<u8> {
        match self {
            Self::NoPermission => vec![0],
            Self::NotFound => vec![1],
            Self::InappropriateRows => vec![2]
        }
    }
}

pub enum AddRowError {
    NoPermission,
    Overflow,
    InappropriateRows
}
impl Serializable for AddRowError {
    fn serialize(&self) -> Vec<u8> {
        match self {
            Self::NoPermission => vec![0],
            Self::Overflow => vec![1],
            Self::InappropriateRows => vec![2]
        }
    }
}

pub enum RemoveRowError {
    NoPermission,
    NotFound
}
impl Serializable for RemoveRowError {
    fn serialize(&self) -> Vec<u8> {
        match self {
            Self::NoPermission => vec![0],
            Self::NotFound => vec![1]
        }
    }
}

pub enum AddUserError {
    NoPermission,
    RoleNotFound
}
impl Serializable for AddUserError {
    fn serialize(&self) -> Vec<u8> {
        match self {
            Self::NoPermission => vec![0],
            Self::RoleNotFound => vec![1]
        }
    }
}