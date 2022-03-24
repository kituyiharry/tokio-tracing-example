use std::{error::Error, net::SocketAddr};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};


// So, that program behaves just like its sync counterpart, 
// as in: it only handles one connection at a time. 
// If client A connects, client B won't be able to connect until client A 
// closes its connection (or has its connection closed by the server):

// But unlike the sync version of our code, 
// it's not blocked on the read syscall. 
// It's, again, blocked on epoll_wait
//
// So next is to logically spawn threads that can epoll_wait for non-blocking io
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr: SocketAddr = "0.0.0.0:3779".parse()?;
    println!("Listening on http://{}", addr);
    let listener = TcpListener::bind(addr).await?;
    loop {
        let (mut stream, addr) = listener.accept().await?;
        println!("Accepted connection from {addr}");

        let mut incoming = vec![];

        loop {
            let mut buf = vec![0u8; 1024];
            let read = stream.read(&mut buf).await?;
            incoming.extend_from_slice(&buf[..read]);

            if incoming.len() > 4 && &incoming[incoming.len() - 4..] == b"\r\n\r\n" {
                break;
            }
        }

        let incoming = std::str::from_utf8(&incoming)?;
        println!("Got HTTP request:\n{}", incoming);
        stream.write_all(b"HTTP/1.1 200 OK\r\n").await?;
        stream.write_all(b"\r\n").await?;
        stream.write_all(b"Hello from plaque!\n").await?;
        println!("Closing connection for {addr}");
    }
}
