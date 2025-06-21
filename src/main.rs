// TODO:
//
// > background thread runs server that listens for locations and responds with
// where to go next based on the travel_fsm
//
// > foreground fsm that creates 100 client connections to the server
// (each client sends a random Location and prints the response + the ping delay)

use std::thread::{self, JoinHandle};

use tokio::{io::AsyncReadExt, net::TcpListener, runtime::Builder};

/// some places to go
enum Location {
    HOME,
    CITY,
    WOODS,
    BEACH,
    FIELD,
    CAFE,
    SHOP,
    CATHEDRAL,
}

/// some arbitrary travel itinerary
fn travel_fsm(loc: Location) -> Location {
    match loc {
        Location::HOME => Location::CITY,
        Location::CITY => Location::WOODS,
        Location::WOODS => Location::BEACH,
        Location::BEACH => Location::FIELD,
        Location::FIELD => Location::CAFE,
        Location::CAFE => Location::SHOP,
        Location::SHOP => Location::CATHEDRAL,
        Location::CATHEDRAL => Location::HOME,
    }
}

const SERVER_PORT: usize = 8080;

/// Creates a background thread for the server component
fn server_thread() -> JoinHandle<()> {
    thread::spawn(|| {
        // create a runtime for the server thread
        let server_rt = Builder::new_current_thread().enable_all().build().unwrap();

        // block until a server listening on a port is created
        server_rt.block_on(async {
            let server = TcpListener::bind(format!("{}:{}", "0.0.0.0", SERVER_PORT))
                .await
                .unwrap();
            // handle multiple client connections
            loop {
                let (mut sock, _) = server.accept().await.unwrap();
                server_rt.spawn(async move {
                    let mut buffer = [0; 1024];
                    'inner: loop {
                        match sock.read(&mut buffer).await {
                            Ok(0) => break 'inner,
                            Ok(n) => {
                                // 1) deserialise the bytes into a Location enum
                                // 2) pass the parsed Location into the FSM
                                // 3) serialise and send the result back to the client
                            }
                            Err(_) => break 'inner,
                        }
                    }
                });
            }
        });
    })
}

fn main() {
    // create a runtime for creating client connections in the foreground thread
    let client_rt = Builder::new_current_thread().enable_all().build().unwrap();

    server_thread().join().unwrap();
}
