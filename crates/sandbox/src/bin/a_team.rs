// use anyhow::Result;
// use ben_sandbox::SandboxEvent;
// use ben_wire::rowbinary::EncodeQuic;
// use tokio::io::AsyncWriteExt;
// use tokio::net::TcpStream;

// #[tokio::main]
// async fn main() -> Result<()> {
//     let addr = "127.0.0.1:41000";

//     let mut stream = TcpStream::connect(addr).await?;

//     let evt = SandboxEvent {
//         id: 42,
//         kind: "sandbox.demo".into(),
//         ts: 1_700_000_000,
//         value: 123.45,
//         flag: true,
//     };

//     // Encode RowBinary payload
//     let mut payload = Vec::new();
//     evt.encode_quic(&mut payload);

//     // Prepend u32 length prefix
//     let len = payload.len() as u32;
//     let mut frame = Vec::new();
//     frame.extend_from_slice(&len.to_le_bytes());
//     frame.extend_from_slice(&payload);

//     stream.write_all(&frame).await?;
//     stream.flush().await?;

//     Ok(())
// }

fn main() {
    println!("Hello, world!");
}
