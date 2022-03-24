use std::{error::Error, net::SocketAddr};

use tokio::net::TcpListener;

// 👇 this will take care of building the runtime
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    // note: our function is now `async fn`

    let addr: SocketAddr = "0.0.0.0:3779".parse()?;
    println!("Listening on http://{}", addr);
    //                       we can await from there!  👇
    let listener = TcpListener::bind(addr).await?;
    loop {
        let (stream, addr) = listener.accept().await?;
        println!("Accepted connection from {addr}");
        // just do nothing, it's a simple example
        drop(stream)
    }
}
