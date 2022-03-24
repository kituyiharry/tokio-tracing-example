use std::{
    error::Error,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
};

fn main() -> Result<(), Box<dyn Error>> {
    let addr: SocketAddr = "0.0.0.0:3779".parse()?;
    let listener = TcpListener::bind(addr)?;
    println!("Listening on http://{}", addr);
    loop {
        let (stream, addr) = listener.accept()?;
        std::thread::spawn(move || {
            if let Err(e) = handle_connection(stream, addr) {
                println!("Error handling connection {addr}: {e}");
            }
        });
    }
}

fn handle_connection(mut stream: TcpStream, addr: SocketAddr) -> Result<(), Box<dyn Error>> {
    println!("Accepted connection from {addr}");

    let mut incoming = vec![];

    loop {
        let mut buf = vec![0u8; 1024];
        let read = stream.read(&mut buf)?;
        incoming.extend_from_slice(&buf[..read]);

        if incoming.len() > 4 && &incoming[incoming.len() - 4..] == b"\r\n\r\n" {
            break;
        }
    }

    let incoming = std::str::from_utf8(&incoming)?;
    println!("Got HTTP request:\n{}", incoming);
    stream.write_all(b"HTTP/1.1 200 OK\r\n")?;
    stream.write_all(b"\r\n")?;
    stream.write_all(b"Hello from plaque!\n")?;
    println!("Closing connection for {addr}");

    Ok(())
}
