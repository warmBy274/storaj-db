use little_collections::{fn_map::FnMap, heap_array::HeapArray as Array};
use little_sync::rwlock::RwLock;
use kanal::unbounded_async;
use std::sync::Arc;

mod storage;
use storage::*;

mod protocol;
use protocol::*;

mod network;
use network::*;

mod access;
use access::*;

mod distribution;
use distribution::*;

mod util;
use util::*;

#[tokio::main(worker_threads = 3)]
async fn main() {
    // need to make arguments parsing and loading DB
    start(13500, vec![], vec![], vec![Table::new("Aaa".to_string(), 123, Array::new(("".to_string(), Data::U32(123)), 0), MemoryBackend::new(65535, 8192))]);
}

fn start(port: u16, users: Vec<User>, roles: Vec<Role>, tables: Vec<Table>) -> () {
    let mut users_map = FnMap::new(|u: &User| u.id as usize);
    for user in users {
        users_map.insert(user);
    }
    let shared_users = Arc::new(RwLock::new(users_map));
    let mut roles_map = FnMap::new(|r: &Role| r.id as usize);
    for role in roles {
        roles_map.insert(role);
    }
    let shared_roles = Arc::new(RwLock::new(roles_map));
    let (ms_tx, ms_rx) = unbounded_async();
    let (lm_tx, lm_rx) = unbounded_async();
    Supervisor::new(ms_rx, tables).start();
    Manager::new(lm_rx, ms_tx, shared_users.clone(), shared_roles.clone()).start();
    Listener::new(port, shared_users, shared_roles, lm_tx).start();
}