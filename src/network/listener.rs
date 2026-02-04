use tokio::{
    net::{TcpSocket, TcpListener, TcpStream},
    io::{AsyncReadExt, AsyncWriteExt},
    sync::{mpsc::UnboundedSender, RwLock},
    spawn
};
use std::{sync::Arc, collections::HashMap};
use rand::random;
use crate::*;

pub struct Listener {
    listener: TcpListener,
    users: Arc<RwLock<HashMap<String, String>>>,
    operation_tx: UnboundedSender<(Operation, UnboundedSender<OperationResult>)>,

}
impl Listener {
    pub fn new(port: u16, users: Arc<RwLock<HashMap<String, String>>>, operation_tx: UnboundedSender<(Operation, UnboundedSender<OperationResult>)>) -> Self {
        let socket = TcpSocket::new_v4().unwrap();
        socket.bind(format!("127.0.0.1:{}", port).parse().unwrap()).expect("Failed to bind port because is already in use");
        Self {
            listener: socket.listen(1024).unwrap(),
            users,
            operation_tx
        }
    }
    pub fn start(self) -> () {
        spawn(async move {
            loop {
                if let Ok((stream, addr)) = self.listener.accept().await {
                    println!("Accepted connection from {}", addr);
                    authentificate(stream, self.users.clone(), self.operation_tx.clone());
                }
            }
        });
    }
}

fn authentificate(mut stream: TcpStream, users: Arc<RwLock<HashMap<String, String>>>, operation_tx: UnboundedSender<(Operation, UnboundedSender<OperationResult>)>) -> () {
    spawn(async move {
        let mut buffer = [0u8; 8192];
        let net = &mut stream;
        let session = loop {
            if let Ok(n) = net.read(&mut buffer).await {
                if n == 0 {
                    println!("Authentication aborted");
                    break None;
                }
                let auth = Authentication::deserialize(&buffer[..n]);
                if let Some(password) = users.read().await.get(&auth.name) {
                    if auth.password == *password {
                        let session = random();
                        if let Err(e) = net.write(&Answer::GiveSession(session).serialize()).await {
                            eprintln!("Failed to write successfull authentification packet: {}", e);
                            break None;
                        }
                        break Some(session);
                    }
                    else {
                        if let Err(e) = net.write(&Answer::WrongPassword.serialize()).await {
                            eprintln!("Failed to write wrong password authentification packet: {}", e);
                            break None;
                        }
                    }
                }
                else {
                    if let Err(e) = net.write(&Answer::WrongUser.serialize()).await {
                        eprintln!("Failed to write wrong user authentification packet: {}", e);
                        break None;
                    }
                }
            }
            else {
                println!("Authentication aborted");
                break None;
            }
        };
        if let Some(id) = session {
            Session::new(id, stream, operation_tx).start();
        }
    });
}