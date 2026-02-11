use kanal::{AsyncSender, AsyncReceiver};
use little_collections::fn_map::FnMap;
use little_sync::rwlock::RwLock;
use rand::random;
use tokio::spawn;
use crate::*;

pub struct Manager {
    rx: AsyncReceiver<(Operation, AsyncSender<OperationResult>)>,
    supervisor: AsyncSender<(usize, Operation, AsyncSender<OperationResult>)>,
    users: Arc<RwLock<FnMap<User>>>,
    roles: Arc<RwLock<FnMap<Role>>>
}
impl Manager {
    pub fn new(
        rx: AsyncReceiver<(Operation, AsyncSender<OperationResult>)>,
        supervisor: AsyncSender<(usize, Operation, AsyncSender<OperationResult>)>,
        users: Arc<RwLock<FnMap<User>>>,
        roles: Arc<RwLock<FnMap<Role>>>
    ) -> Self {
        Self {
            rx,
            supervisor,
            users,
            roles
        }
    }
    pub fn start(self) -> () {
        spawn(async move {
            loop {
                if let Ok((operation, tx)) = self.rx.recv().await {
                    if let Some(id) = operation.get_table_id() {
                        if let Err(_) = self.supervisor.send((id, operation, tx)).await {
                            eprintln!("(Manager): \"Manager -> Supervisor\" channel closed!");
                        }
                    }
                    else {process_global_operation(operation, tx, self.users.clone(), self.roles.clone())}
                }
                else {
                    eprintln!("(Manager): \"Sessions + Listener -> Manager\" channel closed!")
                }
            }
        });
    }
}

fn process_global_operation(
    operation: Operation,
    result_tx: AsyncSender<OperationResult>,
    users: Arc<RwLock<FnMap<User>>>,
    roles: Arc<RwLock<FnMap<Role>>>
) -> () {
    spawn(async move {
        match operation {
            Operation::AddUser(name, password, role) => {
                if roles.read().get(role as usize).is_some() {
                    let id = random();
                    users.write().insert(User::new(name, id, password, role));
                    if let Err(_) = result_tx.send(OperationResult::AddUser(Ok(id))).await {
                        eprintln!("(Manager): \"Manager -> Session\" channel closed!")
                    }
                }
                else {
                    if let Err(_) = result_tx.send(OperationResult::AddUser(Err(AddUserError::RoleNotFound))).await {
                        eprintln!("(Manager): \"Manager -> Session\" channel closed!")
                    }
                }
            }
            _ => unreachable!()
        }
    });
}