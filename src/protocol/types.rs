use little_collections::heap_array::HeapArray as Array;
use crate::*;

pub enum Operation {
    GetRow(usize, usize),
    SetRow(usize, usize, Array<Data>),
    AddRow(usize, Array<Data>),
    RemoveRow(usize, usize)
}
impl Operation {
    pub fn get_table_index(&self) -> Option<usize> {
        match self {
            Self::GetRow(i, _) => Some(*i),
            Self::SetRow(i, _, _) => Some(*i),
            Self::AddRow(i, _) => Some(*i),
            Self::RemoveRow(i, _) => Some(*i)
        }
    }
    pub fn deserialize(data: &[u8]) -> Self {
        match data[0] {
            0 => {Self::GetRow(get_u64(&data[1..9]) as usize, get_u64(&data[9..17]) as usize)}
            1 => {Self::SetRow(get_u64(&data[1..9]) as usize, get_u64(&data[9..17]) as usize, deserialize_data_array(&data[17..]))}
            2 => {Self::AddRow(get_u64(&data[1..9]) as usize, deserialize_data_array(&data[9..]))}
            3 => {Self::RemoveRow(get_u64(&data[1..9]) as usize, get_u64(&data[9..17]) as usize)}
            _ => panic!("Unsupported operation")
        }
    }
}

pub enum OperationResult {
    GetRow(Option<Array<Data>>),
    SetRow(bool),
    AddRow(bool),
    RemoveRow(bool)
}
impl OperationResult {
    pub fn serialize(self) -> Vec<u8> {
        let mut result = Vec::new();
        match self {
            Self::GetRow(option) => {
                result.push(0);
                if let Some(row) = option {
                    result.push(0);
                    result.append(&mut serialize_data_array(row));
                }
                else {
                    result.push(1);
                }
            }
            Self::SetRow(success) => {
                result.push(1);
                if success {result.push(0);}
                else {result.push(1);}
            }
            Self::AddRow(success) => {
                result.push(2);
                if success {result.push(0);}
                else {result.push(1);}
            }
            Self::RemoveRow(success) => {
                result.push(3);
                if success {result.push(0);}
                else {result.push(1);}
            }
        }
        result
    }
}

#[derive(Clone)]
pub enum Data {
    U32(u32),
    U64(u64),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64)
}
impl Data {
    pub fn serialize(self) -> Vec<u8> {
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
}