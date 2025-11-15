use anyhow::Result;
use samesame_protocol::{InputEvent, Message};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, warn, error};

#[cfg(windows)]
mod input_simulator;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let addr: SocketAddr = "0.0.0.0:24800".parse()?;
    let listener = TcpListener::bind(addr).await?;

    info!("SameSame Windows Server listening on {}", addr);
    info!("Waiting for macOS client to connect...");

    loop {
        match listener.accept().await {
            Ok((stream, peer_addr)) => {
                info!("Client connected from {}", peer_addr);
                tokio::spawn(async move {
                    if let Err(e) = handle_client(stream, peer_addr).await {
                        error!("Error handling client {}: {}", peer_addr, e);
                    }
                    info!("Client {} disconnected", peer_addr);
                });
            }
            Err(e) => {
                error!("Failed to accept connection: {}", e);
            }
        }
    }
}

async fn handle_client(mut stream: TcpStream, peer_addr: SocketAddr) -> Result<()> {
    let mut buffer = vec![0u8; 8192];

    loop {
        let n = stream.read(&mut buffer).await?;

        if n == 0 {
            info!("Client {} closed connection", peer_addr);
            break;
        }

        // Try to deserialize the message
        // Note: This is a simplified version that assumes each read contains a complete message
        // For production, you would want proper framing (e.g., length-prefixed messages)
        match Message::from_bytes(&buffer[..n]) {
            Ok(message) => {
                handle_message(message, &mut stream).await?;
            }
            Err(e) => {
                warn!("Failed to deserialize message: {:?}", e);
                // Continue reading
            }
        }
    }

    Ok(())
}

async fn handle_message(message: Message, stream: &mut TcpStream) -> Result<()> {
    match message.event {
        InputEvent::Ping => {
            // Respond with Pong
            let pong = Message::new(message.sequence, InputEvent::Pong);
            let bytes = pong.to_bytes()?;
            stream.write_all(&bytes).await?;
        }
        InputEvent::Pong => {
            // Ignore pong messages
        }
        event => {
            // Simulate the input event on Windows
            #[cfg(windows)]
            {
                if let Err(e) = input_simulator::simulate_event(&event) {
                    warn!("Failed to simulate event: {}", e);
                }
            }
            #[cfg(not(windows))]
            {
                warn!("Input simulation only works on Windows. Received: {:?}", event);
            }
        }
    }

    Ok(())
}
