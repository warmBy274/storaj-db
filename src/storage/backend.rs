use std::{
    fs::{File, OpenOptions, read_dir},
    collections::VecDeque,
    path::PathBuf,
    ffi::OsStr,
    io::Read
};
use little_collections::heap_array::HeapArray as Array;
use crate::*;

pub trait Backend: Send + Sync + 'static {
    fn force_save(&self, table: &Table) -> Result<(), ()>;
    fn get_row(&self, i: usize) -> Result<Array<Data>, ()>;
    fn set_row(&mut self, i: usize, row: Array<Data>) -> Result<(), ()>;
    fn add_row(&mut self, row: Array<Data>) -> Result<(), ()>;
    fn remove_row(&mut self, i: usize) -> Result<Array<Data>, ()>;
}

pub struct MemoryBackend {
    rows: VecDeque<Array<Data>>,
    max_rows_count: usize,
    emergency_rows_extension: usize
}
impl MemoryBackend {
    pub fn new(max_rows_count: usize, emergency_rows_extension: usize) -> Self {
        Self {
            rows: VecDeque::new(),
            max_rows_count,
            emergency_rows_extension
        }
    }
}
impl Backend for MemoryBackend {
    fn force_save(&self, _: &Table) -> Result<(), ()> {Ok(())}
    fn get_row(&self, i: usize) -> Result<Array<Data>, ()> {
        self.rows.get(i).ok_or(()).cloned()
    }
    fn set_row(&mut self, i: usize, row: Array<Data>) -> Result<(), ()> {
        if self.rows.len() > i {
            self.rows[i] = row;
            Ok(())
        }
        else {Err(())}
    }
    fn add_row(&mut self, row: Array<Data>) -> Result<(), ()> {
        if self.rows.len() == self.max_rows_count {Err(())}
        else {
            self.rows.push_front(row);
            Ok(())
        }
    }
    fn remove_row(&mut self, i: usize) -> Result<Array<Data>, ()> {
        if let Some(row) = self.rows.remove(i) {Ok(row)}
        else {Err(())}
    }
}

pub struct StorageBackend {
    path: PathBuf,
    part_size: usize,
    current_part: File,
    rows_len: usize
}
impl StorageBackend {
    pub fn new(path: PathBuf, part_size: usize) -> Self {
        let mut existing_parts: Vec<usize> = read_dir(&path)
            .expect("Failed to read direction")
            .filter_map(|x| x.ok())
            .filter(|x| if let Ok(filetype) = x.file_type() {filetype.is_file()} else {false})
            .filter(|x| if let Some(extension) = x.path().extension() {extension == "datapart"} else {false})
            .filter_map(|x| usize::from_str_radix(x.path().file_stem().unwrap_or(&OsStr::new("")).to_str().unwrap_or(""), 10).ok())
            .collect();
        existing_parts.sort();
        let mut options = OpenOptions::new();
        options.read(true);
        options.append(true);
        options.create_new(true);
        let mut file = options.open(path.join(format!("{}.datapart", get_next_part_number(existing_parts)))).expect("Failed to open datapart file");
        let rows_len = {
            let mut a = [0u8; 8];
            file.read_exact(&mut a).expect("Failed to read datapart file");
            u64::from_be_bytes(a) as usize
        };
        Self {
            path,
            part_size,
            current_part: file,
            rows_len
        }
    }
}

pub struct CombinedBackend {
    rows: VecDeque<Array<Data>>,
    max_rows_count: usize,
    emergency_rows_extension: usize,
    path: PathBuf
}
impl CombinedBackend {
    pub fn new(max_rows_count: usize, emergency_rows_extension: usize, path: PathBuf) -> Self {
        Self {
            rows: VecDeque::new(),
            max_rows_count,
            emergency_rows_extension,
            path
        }
    }
}

fn get_next_part_number(existing_parts: Vec<usize>) -> usize {
    if existing_parts.len() == 0 {
        return 0;
    }
    if existing_parts.last().unwrap() + 1 == existing_parts.len() {
        return existing_parts.len();
    }
    if existing_parts.len() == 1 {
        return 0;
    }
    for i in 0..existing_parts.len() - 1 {
        if existing_parts[i + 1] - existing_parts[i] != 1 {
            return existing_parts[i] + 1;
        }
    }
    unreachable!()
}