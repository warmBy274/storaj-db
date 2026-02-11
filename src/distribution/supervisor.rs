use kanal::{unbounded_async, AsyncSender, AsyncReceiver};
use little_collections::fn_map::FnMap;
use tokio::spawn;
use crate::*;

pub struct Supervisor {
    rx: AsyncReceiver<(usize, Operation, AsyncSender<OperationResult>)>,
    actors: FnMap<(u64, AsyncSender<(Operation, AsyncSender<OperationResult>)>)>
}
impl Supervisor {
    pub fn new(
        rx: AsyncReceiver<(usize, Operation, AsyncSender<OperationResult>)>,
        tables: Vec<Table>
    ) -> Self {
        Self {
            rx,
            actors: {
                let mut result = FnMap::new(|x: &(u64, AsyncSender<(Operation, AsyncSender<OperationResult>)>)| x.0 as usize);
                for actor in tables.into_iter().map(|t| {
                    let (tx, rx) = unbounded_async();
                    let id = t.id;
                    Actor::new(id, t, rx).start();
                    (id, tx)
                }) {
                    result.insert(actor);
                }
                result
            }
        }
    }
    pub fn start(self) -> () {
        spawn(async move {
            loop {
                if let Ok((id, operation, tx)) = self.rx.recv().await {
                    if let Err(_) = self.actors[id].1.send((operation, tx)).await {
                        eprintln!("(Supervisor): \"Supervisor -> Actor #{}\" channel closed!", id);
                    }
                }
                else {
                    eprintln!("(Supervisor): \"Manager -> Supervisor\" channel closed!");
                    break;
                }
            }
        });
    }
}