use tokio::{
    sync::{
        mpsc::{UnboundedSender, UnboundedReceiver}
    },
    spawn
};
use std::sync::{Arc, RwLock};
use crate::*;

pub struct Actor {
    table: Arc<RwLock<Table>>,
    rx: UnboundedReceiver<(Operation, UnboundedSender<OperationResult>)>
}
impl Actor {
    pub fn new(table: Table, rx: UnboundedReceiver<(Operation, UnboundedSender<OperationResult>)>) -> Self {
        Self {
            table: Arc::new(RwLock::new(table)),
            rx
        }
    }
    pub fn start(mut self) -> () {
        spawn(async move {
            loop {
                match self.rx.recv().await {
                    Some((operation, tx)) => {
                        let table = self.table.clone();
                        spawn(async move {
                            let result = match operation {
                                Operation::GetRow(_, i) => {
                                    if let Ok(row) = table.read().unwrap().backend.get_row(i) {OperationResult::GetRow(Some(row))}
                                    else {OperationResult::GetRow(None)}
                                }
                                Operation::SetRow(_, i, row) => {
                                    if let Ok(_) = table.write().unwrap().backend.set_row(i, row) {OperationResult::SetRow(true)}
                                    else {OperationResult::SetRow(false)}
                                }
                                Operation::AddRow(_, row) => {
                                    if let Ok(_) = table.write().unwrap().backend.add_row(row) {OperationResult::AddRow(true)}
                                    else {OperationResult::AddRow(false)}
                                }
                                Operation::RemoveRow(_, i) => {
                                    if let Ok(_) = table.write().unwrap().backend.remove_row(i) {OperationResult::RemoveRow(true)}
                                    else {OperationResult::RemoveRow(false)}
                                }
                                _ => unreachable!()
                            };
                            tx.send(result).unwrap();
                        });
                    }
                    None => {
                        eprintln!("Operation channel closed!");
                        break;
                    }
                }
            }
        });
    }
}