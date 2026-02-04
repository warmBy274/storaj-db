use tokio::{
    net::{
        TcpStream,
        tcp::{OwnedReadHalf, OwnedWriteHalf}
    },
    sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
    io::AsyncWriteExt,
    spawn
};
use crate::*;

pub struct Session {
    id: u64,
    net_tx: OwnedWriteHalf,
    net_rx: OwnedReadHalf,
    operation_tx: UnboundedSender<(Operation, UnboundedSender<OperationResult>)>,
    result_tx: UnboundedSender<OperationResult>,
    result_rx: UnboundedReceiver<OperationResult>
}
impl Session {
    pub fn new(id: u64, stream: TcpStream, operation_tx: UnboundedSender<(Operation, UnboundedSender<OperationResult>)>) -> Self {
        let (net_rx, net_tx) = stream.into_split();
        let (result_tx, result_rx) = unbounded_channel();
        Self {
            id,
            net_tx,
            net_rx,
            operation_tx,
            result_tx,
            result_rx
        }
    }
    pub fn start(mut self) -> () {
        spawn(async move {
            loop {
                let mut buffer = [0u8; 8192];
                if let Ok(n) = self.net_rx.try_read(&mut buffer) {
                    if n == 0 {
                        println!("Session #{} closed", self.id);
                        break;
                    }
                    let data = &buffer[..n];
                    let request = Request::deserialize(&data[0..]);
                    if request.session != self.id {
                        if let Err(e) = self.net_tx.write_all(&Answer::WrongSessionId.serialize()).await {
                            eprintln!("Failed to write network packet: {}", e);
                            break;
                        }
                    }
                    if let Err(_) = self.operation_tx.send((request.operation, self.result_tx.clone())) {
                        eprintln!("\"Session -> supervisor\" operation channel closed");
                        break;
                    }
                }
                if let Ok(or) = self.result_rx.try_recv() {
                    if let Err(e) = self.net_tx.write_all(&Answer::Operation(or).serialize()).await {
                        eprintln!("Failed to write network packet: {}", e);
                        break;
                    }
                }
            }
        });
    }
}