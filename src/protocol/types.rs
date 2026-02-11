use little_collections::heap_array::HeapArray as Array;
use std::{fmt::Display, mem::discriminant};
use crate::*;

#[derive(Debug)]
pub enum Operation {
    GetRow(usize, usize),
    SetRow(usize, usize, Array<Data>),
    AddRow(usize, Array<Data>),
    RemoveRow(usize, usize),
    AddUser(String, String, u64)
}
impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GetRow(i, j) => write!(f, "GetRow({}, {})", i, j),
            Self::SetRow(i, j, row) => write!(f, "SetRow({}, {}, {:?})", i, j, row),
            Self::AddRow(i, row) => write!(f, "AddRow({}, {:?})", i, row),
            Self::RemoveRow(i, j) => write!(f, "RemoveRow({}, {})", i, j),
            Self::AddUser(name, password, role) => write!(f, "AddUser({}, {}, {})", name, password, role)
        }
    }
}
impl Deserializable for Operation {
    fn deserialize(data: &[u8]) -> Self {
        match data[0] {
            0 => Self::GetRow(get_u64(&data[1..9]) as usize, get_u64(&data[9..17]) as usize),
            1 => Self::SetRow(get_u64(&data[1..9]) as usize, get_u64(&data[9..17]) as usize, deserialize_data_array(&data[17..])),
            2 => Self::AddRow(get_u64(&data[1..9]) as usize, deserialize_data_array(&data[9..])),
            3 => Self::RemoveRow(get_u64(&data[1..9]) as usize, get_u64(&data[9..17]) as usize),
            4 => {
                let name_len = get_u64(&data[9..17]) as usize;
                let password_len = get_u64(&data[17..25]) as usize;
                let name = if let Ok(v) = String::from_utf8(data[25..25 + name_len].to_vec()) {v}
                    else {
                        eprintln!("(Operation Deserialization): Non-UTF8 user name");
                        "Non-UTF8 user name".to_string()
                    };
                let password = if let Ok(v) = String::from_utf8(data[25 + name_len..25 + name_len + password_len].to_vec()) {v}
                    else {
                        eprintln!("(Operation Deserialization): Non-UTF8 password");
                        "Non-UTF8 password".to_string()
                    };
                Self::AddUser(
                    name,
                    password,
                    get_u64(&data[1..9]))
            }
            _ => panic!("Unsupported operation")
        }
    }
}
impl Operation {
    pub fn get_no_permission(&self) -> OperationResult {
        match self {
            Self::GetRow(_, _) => OperationResult::GetRow(Err(GetRowError::NoPermission)),
            Self::SetRow(_, _, _) => OperationResult::SetRow(Err(SetRowError::NoPermission)),
            Self::AddRow(_, _) => OperationResult::AddRow(Err(AddRowError::NoPermission)),
            Self::RemoveRow(_, _) => OperationResult::RemoveRow(Err(RemoveRowError::NoPermission)),
            Self::AddUser(_, _, _) => OperationResult::AddUser(Err(AddUserError::NoPermission))
        }
    }
    pub fn get_table_id(&self) -> Option<usize> {
        match self {
            Self::GetRow(i, _) => Some(*i),
            Self::SetRow(i, _, _) => Some(*i),
            Self::AddRow(i, _) => Some(*i),
            Self::RemoveRow(i, _) => Some(*i),
            _ => None
        }
    }
}

pub enum OperationResult {
    GetRow(Result<Array<Data>, GetRowError>),
    SetRow(Result<(), SetRowError>),
    AddRow(Result<(), AddRowError>),
    RemoveRow(Result<Array<Data>, RemoveRowError>),
    AddUser(Result<u64, AddUserError>)
}
impl Serializable for OperationResult {
    fn serialize(&self) -> Vec<u8> {
        let mut result = Vec::new();
        match self {
            Self::GetRow(success) => {
                result.push(0);
                match success {
                    Ok(row) => {
                        result.push(0);
                        result.append(&mut serialize_data_array(row));
                    }
                    Err(e) => {
                        result.push(1);
                        result.append(&mut e.serialize());
                    }
                }
            }
            Self::SetRow(success) => {
                result.push(1);
                match success {
                    Ok(_) => {
                        result.push(0);
                    }
                    Err(e) => {
                        result.push(1);
                        result.append(&mut e.serialize());
                    }
                }
            }
            Self::AddRow(success) => {
                result.push(2);
                match success {
                    Ok(_) => {
                        result.push(0);
                    }
                    Err(e) => {
                        result.push(1);
                        result.append(&mut e.serialize());
                    }
                }
            }
            Self::RemoveRow(success) => {
                result.push(3);
                match success {
                    Ok(row) => {
                        result.push(0);
                        result.append(&mut serialize_data_array(row));
                    }
                    Err(e) => {
                        result.push(1);
                        result.append(&mut e.serialize());
                    }
                }
            }
            Self::AddUser(success) => {
                result.push(4);
                match success {
                    Ok(id) => {
                        result.push(0);
                        result.append(&mut id.to_be_bytes().to_vec());
                    }
                    Err(e) => {
                        result.push(1);
                        result.append(&mut e.serialize());
                    }
                }
            }
        }
        result
    }
}

#[derive(Debug, Clone)]
pub enum Data {
    U32(u32),
    U64(u64),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64)
}
impl Serializable for Data {
    fn serialize(&self) -> Vec<u8> {
        let mut result = Vec::new();
        match self {
            Self::U32(a) => {
                result.push(0);
                result.append(&mut a.to_be_bytes().to_vec())
            }
            Self::U64(a) => {
                result.push(1);
                result.append(&mut a.to_be_bytes().to_vec())
            }
            Self::I32(a) => {
                result.push(2);
                result.append(&mut a.to_be_bytes().to_vec())
            }
            Self::I64(a) => {
                result.push(3);
                result.append(&mut a.to_be_bytes().to_vec())
            }
            Self::F32(a) => {
                result.push(4);
                result.append(&mut a.to_be_bytes().to_vec())
            }
            Self::F64(a) => {
                result.push(5);
                result.append(&mut a.to_be_bytes().to_vec())
            }
        }
        result
    }
}
impl Data {
    pub fn deserialize(data: &[u8]) -> (usize, Self) {
        match data[0] {
            0 => {
                (5, Self::U32({
                    let mut a = [0u8; 4];
                    a.copy_from_slice(&data[1..5]);
                    u32::from_be_bytes(a)
                }))
            }
            1 => {
                (9, Self::U64({
                    let mut a = [0u8; 8];
                    a.copy_from_slice(&data[1..9]);
                    u64::from_be_bytes(a)
                }))
            }
            2 => {
                (5, Self::I32({
                    let mut a = [0u8; 4];
                    a.copy_from_slice(&data[1..5]);
                    i32::from_be_bytes(a)
                }))
            }
            3 => {
                (9, Self::I64({
                    let mut a = [0u8; 8];
                    a.copy_from_slice(&data[1..9]);
                    i64::from_be_bytes(a)
                }))
            }
            4 => {
                (5, Self::F32({
                    let mut a = [0u8; 4];
                    a.copy_from_slice(&data[1..5]);
                    f32::from_be_bytes(a)
                }))
            }
            5 => {
                (9, Self::F64({
                    let mut a = [0u8; 8];
                    a.copy_from_slice(&data[1..9]);
                    f64::from_be_bytes(a)
                }))
            }
            _ => panic!("Unsupported data type")
        }
    }
    pub fn compare(&self, other: &Self) -> bool {
        discriminant(self) == discriminant(other)
    }
}