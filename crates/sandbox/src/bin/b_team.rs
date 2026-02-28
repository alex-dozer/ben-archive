use anyhow::Result;
use ben_wire::Envelope;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "127.0.0.1:54321";
    let listener = TcpListener::bind(addr).await?;
    println!("b_team: listening on {}", addr);

    let (mut socket, peer) = listener.accept().await?;
    println!("b_team: accepted connection from {}", peer);

    let mut len_buf = [0u8; 4];
    socket.read_exact(&mut len_buf).await?;
    let payload_len = u32::from_le_bytes(len_buf) as usize;

    println!("b_team: expecting payload of {} bytes", payload_len);

    let mut payload = vec![0u8; payload_len];
    socket.read_exact(&mut payload).await?;

    println!("b_team: received payload, decoding...");

    let evt = Envelope::parse(&payload)?;
    println!("b_team: got event hash {:x?}", evt.evt_hash);
    println!("b_team: got event hash {:x?}", evt.epoch);
    println!("b_team: got event hash {:x?}", evt.encoding);
    println!("b_team: got event hash {:x?}", evt.payload);

    Ok(())
}
