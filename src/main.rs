use tokio::sync::{RwLock, mpsc::unbounded_channel};
use std::{
    fs::{File, read_dir, remove_dir_all, remove_file},
    collections::HashMap,
    path::PathBuf,
    process::exit,
    env::args,
    sync::Arc
};

mod storage;
use storage::*;

mod protocol;
use protocol::*;

mod network;
use network::*;

mod access;
use access::*;

mod util;
use util::*;

#[tokio::main(worker_threads = 3)]
async fn main() {
    start_work(13100, Vec::new(), Arc::new(RwLock::new(HashMap::new())));
}

fn start_work(port: u16, tables: Vec<Table>, users: Arc<RwLock<HashMap<String, String>>>) -> () {
    let (operation_tx, operaion_rx) = unbounded_channel();
    Supervisor::new(tables, operaion_rx).start();
    Listener::new(port, users, operation_tx).start();
}

fn help() -> () {}