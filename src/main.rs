use std::error::Error;
use std::net::SocketAddr;

use tokio::net::TcpListener;


fn main() -> Result<(), Box<dyn Error>> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(async {
        let addr: SocketAddr = "0.0.0.0:3779".parse()?; 
        println!("Listening on http://{}", addr);
        let listener = TcpListener::bind(addr).await?;
        loop {
            let (stream, addr) = listener.accept().await?;
            println!("Accepted connection from {addr}");
            // do nothing for now, it's a simple example
            drop(stream)
        }
    })
}
