use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::{env, process, thread};

fn main() {
    let client_args = get_client_args();

    if client_args.listener {
        create_listener(client_args.addr)
    } else {
        create_connection(client_args.addr)
    }
}

#[derive(Debug)]
struct Client {
    listener: bool,
    addr: String,
}

// Gets and parses client arguments
fn get_client_args() -> Client {
    let args: Vec<String> = env::args().collect();

    if (args.len() != 3) {
        println!("Invalid Arguments");
        println!("Exaple Usage: rust-chat -l/-c <Address:Port>");

        process::exit(1);
    }

    let is_listener: bool;

    match args[1].as_str() {
        "-l" => is_listener = true,
        "-c" => is_listener = false,
        _ => {
            println!("Invalid client type!");
            println!("Please use -l to listen for connetions, -c to connect to a listening server");
            println!("Exaple Usage: rust-chat -l/-c <Address:Port>");

            process::exit(1);
        }
    }

    let client = Client {
        listener: is_listener,
        addr: String::from(&args[2]),
    };

    client
}

fn create_listener(address: String) {
    let listener = TcpListener::bind(&address).unwrap();
    // Clear the CLI window
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    println!("Server listening on {}", address);
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let stream_clone = stream.try_clone().unwrap();

                thread::spawn(move || handle_receiver(stream));

                handle_sender(stream_clone)
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
    drop(listener);
}

fn create_connection(address: String) {
    let stream = TcpStream::connect(&address).unwrap();
    // Clear the CLI window
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    println!("Connected to {}", address);

    let stream_clone = stream.try_clone().unwrap();

    thread::spawn(move || handle_receiver(stream));

    handle_sender(stream_clone);
}

fn handle_receiver(mut stream: TcpStream) {
    let mut buffer = [0 as u8; 50];
    while match stream.read(&mut buffer) {
        Ok(size) => {
            let string = String::from_utf8_lossy(&buffer[0..size]);
            if &string == "" {
                println!("The other peer has disconnected");
                process::exit(1);
            }
            println!("Peer: {}", string);
            true
        }
        Err(_) => {
            println!(
                "An error occurred, terminating connection with {}",
                stream.peer_addr().unwrap()
            );
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {}
}

fn handle_sender(mut stream: TcpStream) {
    loop {
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();

        let msg = buffer.trim().to_string();

        if msg.as_str() == "!quit" {
            println!("Quitting...");
            break;
        }

        stream
            .write(msg.as_bytes())
            .expect("Unable to send the message");
    }
}
