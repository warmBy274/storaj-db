use std::{
    fs::{File, read_dir, remove_dir_all, remove_file},
    io::{Read, Write},
    path::PathBuf,
    sync::Arc,
    env::args
};
use tokio::sync::{mpsc::unbounded_channel, Mutex};
use rand::random;

mod structures;
use structures::*;

mod stream;
use stream::*;

#[tokio::main]
async fn main() -> () {
    let mut settings = Settings::new(
        13100,
        PathBuf::new(),
        vec![],
        vec![Role::new("admin".to_string(), true, true, vec![7; 1])],
        vec![User::new("admin".to_string(), format!("{:x}", random::<u128>()), 0)]
    );
    let args = Vec::from_iter(args());
    if args.len() == 3 {
        match args[1].as_str() {
            "new" => {
                let path = PathBuf::from(&args[2]);
                if !path.exists() {
                    eprintln!("The path does not exist");
                    return;
                }
                if !path.is_dir() {
                    eprintln!("The path must be a directory");
                    return;
                }
                read_dir(&path)
                    .unwrap()
                    .filter_map(|x| x.ok())
                    .for_each(|x| {
                        if x.path().is_dir() {remove_dir_all(x.path()).unwrap()}
                        else {remove_file(x.path()).unwrap()}
                    });
                settings.main_directory = path;
            }
            "open" => {
                let path = PathBuf::from(&args[2]);
                if !path.exists() {
                    eprintln!("The path does not exist");
                    return;
                }
                if !path.is_dir() {
                    eprintln!("The path must be a directory");
                    return;
                }
                settings.main_directory = path.clone();
                match File::open(path.join("settings")) {
                    Ok(mut f) => {
                        let mut buffer = [0u8; 8192];
                        if let Err(e) = f.read(&mut buffer) {
                            eprintln!("Failed to read settings file: {}", e);
                            return;
                        }
                        settings.port = u16::from_be_bytes([buffer[0], buffer[1]]);
                        let mut offset = 6;
                        settings.roles = {
                            let count = {
                                let mut a = [0u8; 4];
                                a.copy_from_slice(&buffer[2..6]);
                                u32::from_be_bytes(a) as usize
                            };
                            let mut roles = Vec::<Role>::with_capacity(count);
                            for _ in 0..count {
                                let (role_offset, role) = Role::deserialize(&buffer[offset..]);
                                offset += role_offset;
                                roles.push(role);
                            }
                            Arc::new(Mutex::new(roles))
                        };
                        settings.users = {
                            let count = {
                                let mut a = [0u8; 4];
                                a.copy_from_slice(&buffer[offset..offset + 4]);
                                u32::from_be_bytes(a) as usize
                            };
                            offset += 4;
                            let mut users = Vec::<User>::with_capacity(count);
                            for _ in 0..count {
                                let (user_offset, user) = User::deserialize(&buffer[offset..]);
                                offset += user_offset;
                                users.push(user);
                            }
                            Arc::new(Mutex::new(users))
                        };

                    }
                    Err(e) => {
                        eprintln!("Failed to open settings file: {}", e);
                        return;
                    }
                }
                match File::open(path.join("tables")) {
                    Ok(mut f) => {
                        let mut buffer = [0u8; 8192];
                        if let Err(e) = f.read(&mut buffer) {
                            eprintln!("Failed to read settings file: {}", e);
                            return;
                        }
                        // парсинг таблиц и их конфигов
                    }
                    Err(e) => {
                        eprintln!("Failed to open settings file: {}", e);
                        return;
                    }
                }
            }
            _ => {
                help();
                return;
            }
        }
    }
    else {
        eprintln!("Type 'new' or 'open' and path then to create new or open existing database\n\tnote: specify directory path where you want database to be\n\teven if you open existing database specify directory where it is located\n\tnot the settings file path or sth else");
        return;
    }
    start_main_work(&settings);
}

fn start_main_work(settings: &Settings) -> () {
    let (connection_sender, connection_receiver) = unbounded_channel();
    let (stream_writer_sender, stream_writer_receiver) = unbounded_channel();
    let (read_sender, read_receiver) = unbounded_channel();
    let (write_sender, write_receiver) = unbounded_channel();
    handle_connection_listener(settings.port, connection_sender);
    handle_stream_reader(connection_receiver, stream_writer_sender, read_sender, write_sender);
    handle_stream_writer(stream_writer_receiver);
    handle_read_queue(settings.tables.clone(), read_receiver);
    handle_write_queue(settings.tables.clone(), write_receiver);
}

async fn save(settings: &Settings) -> bool {
    match File::create(settings.main_directory.join("settings")) {
        Ok(mut f) => {
            let mut result = Vec::with_capacity(1000);
            result.append(&mut settings.port.to_be_bytes().to_vec());
            println!("Settings saving process started\n\tLocking roles mutex...");
            let roles = settings.roles.lock().await;
            println!("\tPreparing roles...");
            result.append(&mut (roles.len() as u32).to_be_bytes().to_vec());
            for role in roles.iter() {
                result.append(&mut role.serialize());
            }
            println!("\tPrepared {} roles\n\tLocking users mutex...", roles.len());
            let users = settings.users.lock().await;
            println!("\tPreparing users...");
            result.append(&mut (users.len() as u32).to_be_bytes().to_vec());
            for user in users.iter() {
                result.append(&mut user.serialize());
            }
            println!("\tPrepared {} users\n\tSaving to file...", users.len());
            if let Err(e) = f.write_all(&result) {
                eprintln!("Failed to save settings: {}", e);
                return false
            }
            println!("Settings saved successfully!");
        }
        Err(e) => {
            eprintln!("Failed to create/open settings file to save settings: {}", e);
            return false
        }
    }
    match File::create(settings.main_directory.join("tables")) {
        Ok(mut f) => {
            let mut result = Vec::with_capacity(1000);
            println!("Table settings saving process started\n\tLocking tables mutex...");
            let tables = settings.tables.lock().await;
            result.append(&mut (tables.len() as u32).to_be_bytes().to_vec());
            for table in tables.iter() {

            }
            if let Err(e) = f.write_all(&result) {
                eprintln!("Failed to save table settings: {}", e);
                return false
            }
            println!("Settings saved successfully!");
        }
        Err(e) => {
            eprintln!("Failed to create/open tables file to save table settings: {}", e);
            return false
        }
    }
    true
}

fn help() -> () {

}