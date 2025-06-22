// TODO:
//
// > background thread runs server that listens for locations and responds with
// where to go next based on the travel_fsm
//
// > foreground fsm that creates 100 client connections to the server
// (each client sends a random Location and prints the response + the ping delay)

use std::{
    thread::{self, JoinHandle},
    time::Duration,
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    runtime::Builder,
};

/// some places to go
/// (using literal strings for now as these'll be easier to encode/decode than enums)
const HOME: &str = "HOME";
const CITY: &str = "CITY";
const WOODS: &str = "WOODS";
const BEACH: &str = "BEACH";
const FIELD: &str = "FIELD";
const CAFE: &str = "CAFE";
const SHOP: &str = "SHOP";
const CATHEDRAL: &str = "CATHEDRAL";

/// some arbitrary travel itinerary
fn travel_fsm(loc: &str) -> &str {
    // remove trailing newlines
    match loc.trim() {
        HOME => CITY,
        CITY => WOODS,
        WOODS => BEACH,
        BEACH => FIELD,
        FIELD => CAFE,
        CAFE => SHOP,
        SHOP => CATHEDRAL,
        CATHEDRAL => HOME,
        _ => {
            println!("Unknown location: '{}'", loc);
            HOME
        }
    }
}

const SERVER_PORT: u16 = 8080;

/// Creates a background thread for the server component
fn server_thread() -> JoinHandle<()> {
    thread::spawn(|| {
        // create a runtime for the server thread
        let server_rt = Builder::new_current_thread().enable_all().build().unwrap();

        // runtime context for the server
        server_rt.block_on(async {
            let server = TcpListener::bind(format!("{}:{}", "0.0.0.0", SERVER_PORT))
                .await
                .unwrap();
            // handle multiple client connections
            loop {
                let (mut sock, client) = server.accept().await.unwrap();
                server_rt.spawn(async move {
                    let mut buffer = [0; 1024];
                    'inner: loop {
                        let response = match sock.read(&mut buffer).await {
                            Ok(0) => break 'inner,
                            Ok(n) => {
                                let request = &String::from_utf8_lossy(&buffer[..n]).to_string();
                                print!("{} is at the {}", client, request);
                                let response = String::from(travel_fsm(request)) + "\n";
                                print!("Next stop: {}", response);
                                response
                            }
                            Err(_) => break 'inner,
                        };
                        sock.write_all(response.as_bytes()).await.unwrap();
                    }
                });
            }
        });
    })
}

/// Create a background thread for a new client connection
fn client_thread() -> JoinHandle<()> {
    thread::spawn(|| {
        // create a runtime for this client thread
        let client_rt = Builder::new_current_thread().enable_all().build().unwrap();

        let mut payload = String::from(HOME);

        // runtime context for the client
        client_rt.block_on(async move {
            match TcpStream::connect(format!("{}:{}", "127.0.0.1", SERVER_PORT)).await {
                Ok(mut client) => {
                    tokio::time::sleep(Duration::from_millis(2000)).await;
                    client.writable().await.unwrap();
                    let mut buffer: [u8; 1024] = [0; 1024];
                    'inner: loop {
                        let _ = client.try_write(payload.as_bytes());
                        payload = match client.read(&mut buffer).await {
                            Ok(0) => break 'inner,
                            Ok(n) => String::from_utf8_lossy(&buffer[..n]).to_string(),
                            Err(_) => break 'inner,
                        }
                    }
                }
                Err(err) => {
                    println!("{:?}", err);
                }
            }
        });
    })
}

fn main() {
    // create a runtime for creating client connections in the foreground thread
    println!("Creating Server Thread");
    let server = server_thread();

    // create three threads for three unique clients
    client_thread();
    client_thread();
    client_thread();

    // exit the main thread when server thread exits
    server.join().unwrap();
}
