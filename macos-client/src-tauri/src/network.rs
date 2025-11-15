use anyhow::Result;
use samesame_protocol::{InputEvent, Message};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tracing::{debug, error};

/// Connect to Windows server
pub async fn connect(server_ip: &str, port: u16) -> Result<TcpStream> {
    let addr = format!("{}:{}", server_ip, port);
    let stream = TcpStream::connect(&addr).await?;

    // Set TCP_NODELAY for low latency
    stream.set_nodelay(true)?;

    tracing::info!("Connected to Windows server at {}", addr);

    Ok(stream)
}

/// Send an event to the Windows server
pub async fn send_event(stream: &mut TcpStream, message: Message) -> Result<()> {
    let bytes = message.to_bytes()?;

    // Write the serialized message
    stream.write_all(&bytes).await?;
    stream.flush().await?;

    debug!("Sent event: {:?}", message.event);

    Ok(())
}

/// Send a ping to check connection
pub async fn send_ping(stream: &mut TcpStream, sequence: u64) -> Result<()> {
    let message = Message::new(sequence, InputEvent::Ping);
    send_event(stream, message).await
}
