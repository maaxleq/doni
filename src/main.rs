use std::env;
use std::sync::Arc;

use dashmap::DashMap;
use dotenvy::dotenv;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

/// Type alias for the key-value store
type KvStore = DashMap<String, Vec<u8>>;

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    match dotenv() {
        Ok(_) => {}
        Err(_) => eprintln!("Failed to load .env file, continuing"),
    }

    // Retrieve environment variables
    let secret_key = env::var("DONI_SECRET_KEY").expect("DONI_SECRET_KEY not set");
    let host = env::var("DONI_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("DONI_PORT").expect("DONI_PORT not set");

    let address = format!("{}:{}", host, port);

    // Initialize the in-memory key-value store
    let store: Arc<KvStore> = Arc::new(DashMap::new());

    // Bind the TCP listener
    let listener = TcpListener::bind(&address)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to address {}", address));

    println!("Doni running on {}", address);

    loop {
        // Accept incoming connections
        let (socket, addr) = match listener.accept().await {
            Ok((sock, addr)) => (sock, addr),
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
                continue;
            }
        };

        println!("New connection from {}", addr);

        // Clone references for the handler
        let store = Arc::clone(&store);
        let secret_key = secret_key.clone();

        // Spawn a new task to handle the connection
        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket, &secret_key, store).await {
                eprintln!("Error handling connection from {}: {}", addr, e);
            }
            println!("Connection from {} closed", addr);
        });
    }
}

/// Handles an individual client connection.
/// Reads commands line by line, processes them, and writes responses.
async fn handle_connection(
    mut socket: TcpStream,
    secret_key: &str,
    store: Arc<KvStore>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (reader, writer) = socket.split();
    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();

    // Use a mutex to synchronize writes to the socket
    let writer = Arc::new(Mutex::new(writer));

    loop {
        line.clear();
        let bytes_read = buf_reader.read_line(&mut line).await?;

        if bytes_read == 0 {
            // Connection closed
            break;
        }

        // Trim newline and carriage return
        let command_line = line.trim_end_matches(&['\n', '\r'][..]);

        // Skip empty lines
        if command_line.is_empty() {
            continue;
        }

        // Parse the command
        let parts: Vec<&str> = command_line.splitn(4, ' ').collect();

        if parts.len() < 3 {
            write_response(&writer, b"ERROR InvalidCommand\n").await;
            continue;
        }

        let auth_key = parts[0];
        let command = parts[1];
        let key = parts[2];

        // Authenticate
        if auth_key != secret_key {
            write_response(&writer, b"ERROR AuthenticationFailed\n").await;
            continue;
        }

        match command.to_uppercase().as_str() {
            "PUT" => {
                if parts.len() != 4 {
                    write_response(&writer, b"ERROR InvalidCommand\n").await;
                    continue;
                }

                // Parse the value length
                let value_length: usize = match parts[3].parse() {
                    Ok(len) => len,
                    Err(_) => {
                        write_response(&writer, b"ERROR InvalidCommand\n").await;
                        continue;
                    }
                };

                // Read the exact number of bytes for the value
                let mut value = vec![0u8; value_length];
                let mut total_read = 0;
                while total_read < value_length {
                    match buf_reader.read(&mut value[total_read..]).await? {
                        0 => break, // Connection closed unexpectedly
                        n => total_read += n,
                    }
                }

                if total_read != value_length {
                    write_response(&writer, b"ERROR IncompleteValue\n").await;
                    continue;
                }

                // Store the key-value pair
                store.insert(key.to_string(), value);
                write_response(&writer, b"OK\n").await;
            }
            "GET" => {
                if parts.len() != 3 {
                    write_response(&writer, b"ERROR InvalidCommand\n").await;
                    continue;
                }

                match store.get(key) {
                    Some(v) => {
                        let value = v.value();
                        let value_length = value.len();
                        let header = format!("OK {}\n", value_length);
                        let mut response = header.into_bytes();
                        response.extend_from_slice(value);
                        write_raw_response(&writer, &response).await;
                    }
                    None => {
                        write_response(&writer, b"ERROR KeyNotFound\n").await;
                    }
                }
            }
            "DELETE" => {
                if parts.len() != 3 {
                    write_response(&writer, b"ERROR InvalidCommand\n").await;
                    continue;
                }
                if store.remove(key).is_some() {
                    write_response(&writer, b"OK\n").await;
                } else {
                    write_response(&writer, b"ERROR KeyNotFound\n").await;
                }
            }
            _ => {
                write_response(&writer, b"ERROR InvalidCommand\n").await;
            }
        }
    }

    Ok(())
}

/// Writes a response line to the client.
async fn write_response(writer: &Arc<Mutex<tokio::net::tcp::WriteHalf<'_>>>, response: &[u8]) {
    let mut writer = writer.lock().await;
    if let Err(e) = writer.write_all(response).await {
        eprintln!("Failed to write response: {}", e);
    }
}

/// Writes raw bytes to the client.
async fn write_raw_response(writer: &Arc<Mutex<tokio::net::tcp::WriteHalf<'_>>>, response: &[u8]) {
    let mut writer = writer.lock().await;
    if let Err(e) = writer.write_all(response).await {
        eprintln!("Failed to write raw response: {}", e);
    }
}
