use little_collections::heap_array::HeapArray as Array;
use crate::*;

pub struct Table {
    pub name: String,
    pub id: u64,
    columns: Array<(String, Data)>,
    backend: Box<dyn Backend>
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
    pub fn get_row(&self, i: usize) -> Result<Array<Data>, GetRowError> {
        if let Ok(row) = self.backend.get_row(i) {Ok(row)}
        else {Err(GetRowError::NotFound)}
    }
    pub fn set_row(&mut self, i: usize, row: Array<Data>) -> Result<(), SetRowError> {
        if self.columns.iter().map(|x| &x.1).zip(row.iter()).all(|(a, b)| a.compare(b)) {
            if let Ok(_) = self.backend.set_row(i, row) {Ok(())}
            else {Err(SetRowError::NotFound)}
        }
        else {Err(SetRowError::InappropriateRows)}
    }
    pub fn add_row(&mut self, row: Array<Data>) -> Result<(), AddRowError> {
        if self.columns.iter().map(|x| &x.1).zip(row.iter()).all(|(a, b)| a.compare(b)) {
            if let Ok(_) = self.backend.add_row(row) {Ok(())}
            else {Err(AddRowError::Overflow)}
        }
        else {Err(AddRowError::InappropriateRows)}
    }
    pub fn remove_row(&mut self, i: usize) -> Result<Array<Data>, RemoveRowError> {
        if let Ok(row) = self.backend.remove_row(i) {Ok(row)}
        else {Err(RemoveRowError::NotFound)}
    }
}