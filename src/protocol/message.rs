use crate::*;

pub struct Authentication {
    pub id: u64,
    pub password: String
}
impl Deserializable for Authentication {
    fn deserialize(data: &[u8]) -> Self {
        let password_len = get_u64(&data[8..16]) as usize;
        let password = String::from_utf8(data[16..16 + password_len].to_vec()).unwrap_or("Non-UTF8 password".to_string());
        Self {
            id: get_u64(&data[..8]),
            password
        }
    }
}

pub struct Request {
    pub session: u64,
    pub operation: Operation
}
impl Deserializable for Request {
    fn deserialize(data: &[u8]) -> Self {
        Self {
            session: get_u64(&data[0..8]),
            operation: Operation::deserialize(&data[8..])
        }
    }
}

pub enum Answer {
    Operation(OperationResult),
    GiveSession(u64),
    WrongUser,
    WrongPassword,
    WrongSessionId
}
impl Serializable for Answer {
    fn serialize(&self) -> Vec<u8> {
        let mut result = Vec::new();
        match self {
            Self::Operation(or) => {
                result.push(0);
                result.append(&mut or.serialize());
            }
            Self::GiveSession(id) => {
                result.push(1);
                result.append(&mut id.to_be_bytes().to_vec());
            }
            Self::WrongUser => {
                result.push(2);
            }
            Self::WrongPassword => {
                result.push(3);
            }
            Self::WrongSessionId => {
                result.push(4);
            }
        }
        result
    }
}
