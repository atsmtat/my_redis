use tokio::io;
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6666").await?;
    println!("Listening...");
    loop {
        let (stream, addr) = listener.accept().await?;
        println!("Accepted {addr}");
        tokio::spawn(async move { echo(stream).await });
    }
}

async fn echo(mut stream: TcpStream) {
    let (mut rd, mut wr) = stream.split();
    if io::copy(&mut rd, &mut wr).await.is_err() {
        eprintln!("Failed to copy");
    }
}
