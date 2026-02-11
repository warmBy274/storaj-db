use tokio::{
    net::{TcpSocket, TcpListener, TcpStream},
    io::{AsyncReadExt, AsyncWriteExt},
    spawn
};
use little_collections::fn_map::FnMap;
use little_sync::rwlock::RwLock;
use kanal::AsyncSender;
use std::sync::Arc;
use rand::random;
use crate::*;

pub struct Listener {
    listener: TcpListener,
    users: Arc<RwLock<FnMap<User>>>,
    roles: Arc<RwLock<FnMap<Role>>>,
    manager: AsyncSender<(Operation, AsyncSender<OperationResult>)>
}
impl Listener {
    pub fn new(
        port: u16,
        users: Arc<RwLock<FnMap<User>>>,
        roles: Arc<RwLock<FnMap<Role>>>,
        manager: AsyncSender<(Operation, AsyncSender<OperationResult>)>
    ) -> Self {
        let socket = TcpSocket::new_v4().unwrap();
        socket.bind(format!("127.0.0.1:{}", port).parse().unwrap()).expect("Failed to bind port because is already in use");
        Self {
            listener: socket.listen(1024).unwrap(),
            users,
            roles,
            manager
        }
    }
    pub fn start(self) -> () {
        spawn(async move {
            loop {
                match self.listener.accept().await {
                    Ok((stream, addr)) => {
                        println!("(Listener): Accepted connection from {}", addr);
                        process_authentication(stream, self.users.clone(), self.roles.clone(), self.manager.clone());
                    }
                    Err(e) => {
                        eprintln!("(Listener): Failed to accept connection: {}", e);
                    }
                }
            }
        });
    }
}

fn process_authentication(
    mut stream: TcpStream,
    users: Arc<RwLock<FnMap<User>>>,
    roles: Arc<RwLock<FnMap<Role>>>,
    manager: AsyncSender<(Operation, AsyncSender<OperationResult>)>
) -> () {
    spawn(async move {
        let mut buffer = [0u8; 8192];
        let session = loop {
            match stream.read(&mut buffer).await {
                Ok(0) => {
                    println!("(Listener): Authentication aborted");
                    break None;
                }
                Ok(n) => {
                    let auth = Authentication::deserialize(&buffer[..n]);
                    if let Some(user) = users.read().get(auth.id as usize) {
                        if auth.password == user.password {
                            let id = random();
                            println!("(Listener): Successfill authentication");
                            if let Err(e) = stream.write(&Answer::GiveSession(id).serialize()).await {
                                eprintln!("\tFailed to write network packet: {}", e);
                                break None;
                            }
                            break Some((id, user.id));
                        }
                        else {
                            eprintln!("(Listener): Wrong Password");
                            if let Err(e) = stream.write(&Answer::WrongPassword.serialize()).await {
                                eprintln!("\tFailed to write network packet: {}", e);
                                break None;
                            }
                        }
                    }
                    else {
                        eprintln!("(Listener): Wrong User");
                        if let Err(e) = stream.write(&Answer::WrongUser.serialize()).await {
                            eprintln!("Failed to write network packet: {}", e);
                            break None;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("(Listener): Failed to authenticate user: {}", e);
                    break None;
                }
            }
        };
        if let Some((id, user)) = session {
            Session::new(id, user, users, roles, stream, manager).start();
        }
    });
}