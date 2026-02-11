use tokio::{
    net::{
        TcpStream,
        tcp::{OwnedReadHalf, OwnedWriteHalf}
    },
    io::{AsyncWriteExt, AsyncReadExt},
    select,
    spawn
};
use kanal::{unbounded_async, AsyncSender, AsyncReceiver};
use crate::*;

pub struct Session {
    id: u64,
    user: u64,
    users: Arc<RwLock<FnMap<User>>>,
    roles: Arc<RwLock<FnMap<Role>>>,
    net_tx: OwnedWriteHalf,
    net_rx: OwnedReadHalf,
    manager: AsyncSender<(Operation, AsyncSender<OperationResult>)>,
    result_tx: AsyncSender<OperationResult>,
    result_rx: AsyncReceiver<OperationResult>
}
impl Session {
    pub fn new(
        id: u64,
        user: u64,
        users: Arc<RwLock<FnMap<User>>>,
        roles: Arc<RwLock<FnMap<Role>>>,
        stream: TcpStream,
        manager: AsyncSender<(Operation, AsyncSender<OperationResult>)>
    ) -> Self {
        let (net_rx, net_tx) = stream.into_split();
        let (result_tx, result_rx) = unbounded_async();
        Self {
            id,
            user,
            users,
            roles,
            net_tx,
            net_rx,
            manager,
            result_tx,
            result_rx
        }
    }
    pub fn start(mut self) -> () {
        spawn(async move {
            let mut buffer = [0u8; 8192];
            loop {
                select! {
                    biased;
                    read = self.net_rx.read(&mut buffer) => {
                        match read {
                            Ok(0) => {
                                println!("(Session #{}): Closed", self.id);
                                break;
                            }
                            Ok(n) => {
                                let data = &buffer[..n];
                                let request = Request::deserialize(&data[0..]);
                                if request.session != self.id {
                                    eprintln!("(Session #{}): Wrong Session Id: {}", self.id, request.session);
                                    if let Err(e) = self.net_tx.write_all(&Answer::WrongSessionId.serialize()).await {
                                        eprintln!("\tFailed to write network packet: {}", e);
                                        break;
                                    }
                                }
                                if let Some(user) = self.users.read().get(self.user as usize) {
                                    if let Some(role) = self.roles.read().get(user.role as usize) {
                                        if role.can_perform(&request.operation) {
                                            if let Err(_) = self.manager.send((request.operation, self.result_tx.clone())).await {
                                                eprintln!("(Session #{}): \"Session #{} -> Manager\" channel closed!", self.id, self.id);
                                                break;
                                            }
                                        }
                                        else {
                                            eprintln!("(Session #{}): {}(User #{}) can't perform this operation({}), it is may be an API substitution", self.id, user.name, user.id, request.operation);
                                            if let Err(e) = self.net_tx.write_all(&Answer::Operation(request.operation.get_no_permission()).serialize()).await {
                                                eprintln!("(Session #{}): Failed to write network packet: {}", self.id, e);
                                                break;
                                            }
                                        }
                                    }
                                    else {
                                        eprintln!("(Session #{}): Role #{} for {}(User #{}) not found!", self.id, user.role, user.name, user.id);
                                        break;
                                    }
                                }
                                else {
                                    eprintln!("(Session #{}): User #{} not found!", self.id, self.user);
                                    break;
                                }
                            }
                            Err(e) => {
                                eprintln!("(Session #{}): Failed to read network packet: {}", self.id, e);
                                break;
                            }
                        }
                    }
                    recv = self.result_rx.recv() => {
                        if let Ok(or) = recv {
                            if let Err(e) = self.net_tx.write_all(&Answer::Operation(or).serialize()).await {
                                eprintln!("(Session #{}): Failed to write network packet: {}", self.id, e);
                                break;
                            }
                        }
                        else {
                            eprintln!("(Session #{}): \"Actor | Manager -> Session #{}\" channel closed!", self.id, self.id);
                        }
                    }
                }
            }
        });
    }
}