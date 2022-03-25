// So, that program behaves just like its sync counterpart, 
// as in: it only handles one connection at a time. 
// If client A connects, client B won't be able to connect until client A 
// closes its connection (or has its connection closed by the server):

// But unlike the sync version of our code, 
// it's not blocked on the read syscall. 
// It's, again, blocked on epoll_wait
//
// So next is to logically spawn threads that can epoll_wait for non-blocking io


use std::{error::Error, net::SocketAddr};

use tracing_subscriber::EnvFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Registry};
use tracing_tree::HierarchicalLayer;
use console_subscriber;


use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use tracing::{debug, info, info_span, Instrument};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let tracer =
        opentelemetry_jaeger::new_pipeline().install_batch(opentelemetry::runtime::Tokio)?;

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let (console, server) = console_subscriber::ConsoleLayer::builder().build();

    tokio::spawn(async move {
        // This is a GRPC server for tokio-console
        server.serve().await.unwrap();
    });

    Registry::default()
        .with(EnvFilter::from_default_env())
        .with(
            HierarchicalLayer::new(2)
                .with_targets(true)
                .with_bracketed_fields(true),
        )
        .with(telemetry)
        .with(console)
        .init();

    //run_server().await?;

    // Wrap our server in a task to see connection tasks
    let handle = tokio::task::spawn(async { run_server().await.unwrap() });
    handle.await?;

    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}

// 👇 here we use an attribute macro
#[tracing::instrument]
async fn run_server() -> Result<(), Box<dyn Error>> {
    let addr: SocketAddr = "0.0.0.0:3779".parse()?;
    info!("Listening on http://{}", addr);
    let listener = TcpListener::bind(addr).await?;
    loop {
        let (stream, addr) = listener.accept().instrument(info_span!("accept")).await?;
        tokio::spawn(async move {
            if let Err(err) = handle_connection(stream, addr).await {
                tracing::error!(%err, "Error handling connection" );
            }
        });
    }
}

#[tracing::instrument(skip(stream))]
async fn handle_connection(mut stream: TcpStream, addr: SocketAddr) -> Result<(), Box<dyn Error>> {
    let req = read_http_request(&mut stream).await?;
    debug!(%req, "Got HTTP request");
    write_http_response(&mut stream).await?;

    Ok(())
}

#[tracing::instrument(skip(stream))]
async fn read_http_request(mut stream: impl AsyncRead + Unpin) -> Result<String, Box<dyn Error>> {
    let mut incoming = vec![];

    loop {
        let mut buf = vec![0u8; 1024];
        let read = stream.read(&mut buf).await?;
        incoming.extend_from_slice(&buf[..read]);

        if incoming.len() > 4 && &incoming[incoming.len() - 4..] == b"\r\n\r\n" {
            break;
        }
    }

    Ok(String::from_utf8(incoming)?)
}

#[tracing::instrument(skip(stream))]
async fn write_http_response(mut stream: impl AsyncWrite + Unpin) -> Result<(), Box<dyn Error>> {
    stream.write_all(b"HTTP/1.1 200 OK\r\n").await?;
    stream.write_all(b"\r\n").await?;
    stream.write_all(b"Hello from plaque!\n").await?;
    Ok(())
}
