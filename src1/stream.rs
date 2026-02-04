use tokio::runtime::Builder as RuntimeBuilder;
use std::thread::Builder as ThreadBuilder;
use tokio::{
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener,
        TcpStream
    },
    sync::mpsc::{
        UnboundedSender,
        UnboundedReceiver,
        unbounded_channel
    },
    io::{AsyncWriteExt, AsyncReadExt},
    sync::Mutex,
    task::spawn
};
use std::sync::Arc;
use::runtime_array::RuntimeArray as Array;
use crate::structures::*;

pub fn handle_connection_listener(
    port: u16,
    connection_sender: UnboundedSender<TcpStream>
) -> () {
    build_thread_runtime("Connection Listener", async move {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await.expect(&format!("Failed to bind port {} to listen for connections", port));
        loop {
            match listener.accept().await {
                Ok((stream, address)) => {
                    println!("New connection {}:{}", address.ip(), address.port());
                    connection_sender.send(stream)
                        .expect("Connection channel closed")
                }
                Err(e) => eprintln!("Failed to establish new connection {}", e)
            }
        }
    });
}

pub fn handle_stream_reader(
    mut connection_receiver: UnboundedReceiver<TcpStream>,
    stream_writer_sender: UnboundedSender<(OwnedWriteHalf, UnboundedReceiver<Array<u8>>)>,
    read_queue: UnboundedSender<(usize, usize, u32, UnboundedSender<Array<u8>>)>,
    write_queue: UnboundedSender<(Operation, u32, UnboundedSender<Array<u8>>)>
) -> () {
    build_thread_runtime("Stream Reader", async move {
        loop {
            let stream = connection_receiver.recv().await
                .expect("Stream queue channel closed");
            let (reader, writer) = stream.into_split();
            let (data_sender, data_receiver) = unbounded_channel::<Array<u8>>();
            stream_writer_sender.send((writer, data_receiver))
                .expect("Stream writer channel closed");
            spawn(handle_message(reader, read_queue.clone(), write_queue.clone(), data_sender));
        }
    });
}

pub fn handle_read_queue(
    raw_table: Arc<Mutex<Vec<Table>>>,
    mut read_queue: UnboundedReceiver<(usize, usize, u32, UnboundedSender<Array<u8>>)>
) -> () {
    build_thread_runtime("Read", async move {
        loop {
            let (table_index, i, transaction, data_sender) = read_queue.recv().await
                .expect("Read queue channel closed");
            let table = raw_table.clone();
            spawn(async move {
                let row = table.lock().await[table_index].get_row(i);
                let answer = AnswerMessage::new(Answer::Give(row), transaction);
                data_sender.send(Array::from_slice(&answer.serialize()))
                    .expect("Data channel closed");
            });
        }
    });
}

pub fn handle_write_queue(
    raw_tables: Arc<Mutex<Vec<Table>>>,
    mut write_queue: UnboundedReceiver<(Operation, u32, UnboundedSender<Array<u8>>)>
) -> () {
    build_thread_runtime("Write", async move {
        loop {
            let (operation, transaction, data_sender) = write_queue.recv().await
                .expect("Write queue channel closed");
            let raw_tables_clone = raw_tables.clone();
            spawn(async move {
                let mut tables = raw_tables_clone.lock().await;
                match operation {
                    Operation::Read(_, _) => unreachable!(),
                    Operation::Append(table_index, row) => {
                        let answer;
                        match tables[table_index as usize].add_row(row) {
                            Ok(_) => {
                                answer = AnswerMessage::new(Answer::AppendSuccess, transaction);
                            }
                            Err(e) => {
                                eprintln!("{}", e);
                                answer = AnswerMessage::new(Answer::AppendError(e), transaction);
                            }
                        }
                        data_sender.send(Array::from_slice(&answer.serialize()))
                            .expect("Data channel closed");
                    }
                }
            });
        }
    });
}

pub fn handle_stream_writer(
    mut stream_writer_receiver: UnboundedReceiver<(OwnedWriteHalf, UnboundedReceiver<Array<u8>>)>
) -> () {
    build_thread_runtime("Stream Writer", async move {
        loop {
            let (mut writer, mut data_receiver) = stream_writer_receiver.recv().await
                .expect("Stream writer queue channel closed");
            spawn(async move {
                loop {
                    match data_receiver.recv().await {
                        Some(data) => {
                            if let Err(e) = writer.write(data.as_slice()).await {
                                eprintln!("Failed to send answer in TCP stream: {}", e)
                            }
                        }
                        None => break
                    }
                }
            });
        }
    });
}

async fn handle_message(
    mut reader: OwnedReadHalf,
    read_queue: UnboundedSender<(usize, usize, u32, UnboundedSender<Array<u8>>)>,
    write_queue: UnboundedSender<(Operation, u32, UnboundedSender<Array<u8>>)>,
    data_sender: UnboundedSender<Array<u8>>
) -> () {
    let mut buffer = [0u8; 8192];
    loop {
        match reader.read(&mut buffer).await {
            Ok(n) => {
                if n == 0 {
                    println!("Connection closed");
                    break;
                }
                match RequestMessage::deserialize(&buffer[..n]) {
                    Ok(message) => {
                        match message.operation {
                            Operation::Read(table, index) => {
                                read_queue.send((table as usize, index as usize, message.transaction, data_sender.clone()))
                                    .expect("Read queue channel closed");
                            }
                            _ => {
                                write_queue.send((message.operation, message.transaction, data_sender.clone()))
                                    .expect("Write queue channel closed");
                            }
                        }
                    }
                    Err(e) => eprintln!("Failed to deserialize message: {}", e)
                }
            }
            Err(e) => {
                eprintln!("Failed to read request from TCP stream: {}", e);
                break;
            }
        }
    }
}

fn build_thread_runtime<F>(name: &str, f: F) -> ()
where
    F: Future<Output = ()> + Send + 'static
{
    let thread_name = format!("{} Thread", name);
    let thread_failure = format!("Failed to create thread for {} handling", name);
    let runtime_name = format!("{} Runtime", name);
    let runtime_failure = format!("Failed to create tokio runtime for {} handling", name);
    ThreadBuilder::new()
        .name(thread_name)
        .spawn(move || {
            RuntimeBuilder::new_current_thread()
                .enable_all()
                .thread_name(runtime_name)
                .build()
                .expect(&runtime_failure)
                .block_on(f)
        })
        .expect(&thread_failure);
}