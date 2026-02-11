use kanal::{AsyncSender, AsyncReceiver};
use little_sync::rwlock::RwLock;
use std::sync::Arc;
use tokio::spawn;
use crate::*;

pub struct Actor {
    id: u64,
    table: Arc<RwLock<Table>>,
    rx: AsyncReceiver<(Operation, AsyncSender<OperationResult>)>
}
impl Actor {
    pub fn new(id: u64, table: Table, rx: AsyncReceiver<(Operation, AsyncSender<OperationResult>)>) -> Self {
        Self {
            id,
            table: Arc::new(RwLock::new(table)),
            rx
        }
    }
    pub fn start(self) -> () {
        spawn(async move {
            loop {
                if let Ok((operation, tx)) = self.rx.recv().await {
                    process_operation(self.id, operation, self.table.clone(), tx);
                }
                else {
                    eprintln!("(Actor #{}): \"Supervisor -> Actor #{}\" channel closed!", self.id, self.id);
                    break;
                }
            }
        });
    }
}

fn process_operation(
    id: u64,
    operation: Operation,
    table: Arc<RwLock<Table>>,
    result_tx: AsyncSender<OperationResult>
) -> () {
    spawn(async move {
        let result = match operation {
            Operation::GetRow(_, i) => {OperationResult::GetRow(table.read().get_row(i))}
            Operation::SetRow(_, i, row) => {OperationResult::SetRow(table.write().set_row(i, row))}
            Operation::AddRow(_, row) => {OperationResult::AddRow(table.write().add_row(row))}
            Operation::RemoveRow(_, i) => {OperationResult::RemoveRow(table.write().remove_row(i))}
            _ => unreachable!()
        };
        if let Err(_) = result_tx.send(result).await {
            eprintln!("(Actor #{}): \"Actor #{} -> Session\" channel closed!", id, id);
        }
    });
}