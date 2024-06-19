use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> io::Result<()> {
    // Connect to the server
    let stream = TcpStream::connect("127.0.0.1:6666").await?;
    let (mut rd, mut wr) = io::split(stream);

    let write_task = tokio::spawn(async move {
        let msg = b"Hello world";
        _ = wr.write_all(&msg[..]).await;
    });

    let read_task = tokio::spawn(async move {
        let mut buf = [0; 1024];
        let mut response: Vec<u8> = vec![];
        loop {
            match rd.read(&mut buf[..]).await {
                Ok(0) => break,
                Ok(n) => response.extend(buf.iter().take(n)),
                Err(e) => eprintln!("Received error {e}"),
            }
        }
        println!("Received: {}", String::from_utf8(response).unwrap())
    });

    write_task.await?;
    read_task.await?;
    Ok(())
}
