use little_collections::heap_array::HeapArray as Array;
use crate::*;

pub struct Table {
    pub name: String,
    pub id: u64,
    columns: Array<(String, Data)>,
    pub backend: Box<dyn Backend>
}
impl Table {
    pub fn new(name: String, id: u64, columns: Array<(String, Data)>, backend: impl Backend) -> Self {
        Self {
            name,
            id,
            columns,
            backend: Box::new(backend)
        }
    }
}