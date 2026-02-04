use crate::*;

pub struct Authentication {
    pub name: String,
    pub password: String
}
impl Authentication {
    pub fn deserialize(data: &[u8]) -> Self {
        let name_len = get_u64(&data[..8]) as usize;
        let password_len = get_u64(&data[8..16]) as usize;
        let name = String::from_utf8(data[16..16 + name_len].to_vec()).expect("Non-UTF8 username");
        let password = String::from_utf8(data[16 + name_len..16 + name_len + password_len].to_vec()).expect("Non-UTF8 password");
        Self {
            name,
            password
        }
    }
}

pub struct Request {
    pub session: u64,
    pub operation: Operation
}
impl Request {
    pub fn deserialize(data: &[u8]) -> Self {
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
impl Answer {
    pub fn serialize(self) -> Vec<u8> {
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
