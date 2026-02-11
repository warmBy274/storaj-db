use little_collections::heap_array::HeapArray as Array;
use crate::*;

pub trait Serializable {
    fn serialize(&self) -> Vec<u8>;
}

pub trait Deserializable {
    fn deserialize(data: &[u8]) -> Self;
}

pub fn get_u64(data: &[u8]) -> u64 {
    let mut a = [0u8; 8];
    a.copy_from_slice(data);
    u64::from_be_bytes(a)
}

pub fn deserialize_data_array(data: &[u8]) -> Array<Data> {
    let length = get_u64(&data[..4]) as usize;
    let mut offset = 4;
    let mut result = Array::new(Data::U32(0), length);
    for i in  0..length {
        let (data_offset, data) = Data::deserialize(&data[offset..]);
        result[i] = data;
        offset += data_offset;
    }
    result
}

pub fn serialize_data_array(array: &Array<Data>) -> Vec<u8> {
    let mut result = Vec::new();
    result.append(&mut (array.len() as u64).to_be_bytes().to_vec());
    for data in array {
        result.append(&mut data.serialize());
    }
    result
}