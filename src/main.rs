use std::env;
use std::sync::Arc;

use dashmap::DashMap;
use dotenvy::dotenv;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenv().expect("Failed to load .env file");

    // Retrieve environment variables
    let secret_key = env::var("DONI_SECRET_KEY").expect("DONI_SECRET_KEY not set");
    let host = env::var("DONI_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("DONI_PORT").expect("DONI_PORT not set");

    let address = format!("{}:{}", host, port);

    // Initialize the in-memory key-value store
    let store = Arc::new(DashMap::new());

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

        // Clone references for the handler
        let store = Arc::clone(&store);
        let secret_key = secret_key.clone();

        // Spawn a new task to handle the connection
        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket, &secret_key, store).await {
                eprintln!("Error handling connection from {}: {}", addr, e);
            }
        });
    }
}

/// Handles an individual client connection.
/// Reads commands line by line, processes them, and writes responses.
async fn handle_connection(
    socket: TcpStream,
    secret_key: &str,
    store: Arc<DashMap<String, String>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (reader, mut writer) = socket.into_split();
    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();

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
            writer
                .write_all(b"ERROR InvalidCommand\n")
                .await
                .unwrap_or(());
            continue;
        }

        let auth_key = parts[0];
        let command = parts[1];
        let key = parts[2];
        let value = if parts.len() == 4 { parts[3] } else { "" };

        // Authenticate
        if auth_key != secret_key {
            writer
                .write_all(b"ERROR AuthenticationFailed\n")
                .await
                .unwrap_or(());
            continue;
        }

        match command.to_uppercase().as_str() {
            "PUT" => {
                if parts.len() != 4 {
                    writer
                        .write_all(b"ERROR InvalidCommand\n")
                        .await
                        .unwrap_or(());
                    continue;
                }
                store.insert(key.to_string(), value.to_string());
                writer.write_all(b"OK\n").await.unwrap_or(());
            }
            "GET" => {
                if parts.len() != 3 {
                    writer
                        .write_all(b"ERROR InvalidCommand\n")
                        .await
                        .unwrap_or(());
                    continue;
                }
                match store.get(key) {
                    Some(v) => {
                        let response = format!("OK {}\n", v.value());
                        writer.write_all(response.as_bytes()).await.unwrap_or(());
                    }
                    None => {
                        writer
                            .write_all(b"ERROR KeyNotFound\n")
                            .await
                            .unwrap_or(());
                    }
                }
            }
            "DELETE" => {
                if parts.len() != 3 {
                    writer
                        .write_all(b"ERROR InvalidCommand\n")
                        .await
                        .unwrap_or(());
                    continue;
                }
                if store.remove(key).is_some() {
                    writer.write_all(b"OK\n").await.unwrap_or(());
                } else {
                    writer
                        .write_all(b"ERROR KeyNotFound\n")
                        .await
                        .unwrap_or(());
                }
            }
            _ => {
                writer
                    .write_all(b"ERROR InvalidCommand\n")
                    .await
                    .unwrap_or(());
            }
        }
    }

    Ok(())
}
