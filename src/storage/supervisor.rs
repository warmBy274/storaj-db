use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    spawn
};
use crate::*;

pub struct Supervisor {
    rx: UnboundedReceiver<(Operation, UnboundedSender<OperationResult>)>,
    actors: Vec<UnboundedSender<(Operation, UnboundedSender<OperationResult>)>>
}
impl Supervisor {
    pub fn new(tables: Vec<Table>, rx: UnboundedReceiver<(Operation, UnboundedSender<OperationResult>)>) -> Self {
        Self {
            rx,
            actors: tables.into_iter().map(|t| {
                let (tx, rx) = unbounded_channel();
                let actor = Actor::new(t, rx);
                actor.start();
                tx
            }).collect()
        }
    }
    pub fn start(mut self) -> () {
        spawn(async move {
            loop {
                match self.rx.recv().await {
                    Some((operation, tx)) => {
                        if let Some(i) = operation.get_table_index() {
                            if let Err(_) = self.actors[i].send((operation, tx)) {
                                eprintln!("Actor #{} operation channel closed", i);
                            }
                        }
                        else {} // match global operations here
                    }
                    None => {
                        eprintln!("Supervisor channel closed!");
                        break;
                    }
                }
            }
        });
    }
}