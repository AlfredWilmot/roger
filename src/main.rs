use std::{
    error, fmt, net::SocketAddr, thread::{self, JoinHandle}, time::Duration
};

use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpSocket, TcpStream},
    runtime::Builder,
};


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

fn main() {
    // create a runtime for creating client connections in the foreground thread
    println!("Creating Server Thread");
    let server = server_thread();

    // exit the main thread when server thread exits
    server.join().unwrap();
}
