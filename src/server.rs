use crate::common::{ALL_INTERFACES, Error, Message, rx, tx};
use std::thread::{self, JoinHandle};
use tokio::{net::TcpListener, runtime::Builder};

pub fn travel_guide<T: Clone + Send + 'static>(
    port: u16,
    itinerary: T,
    response_rules: fn(&Message, T) -> Message,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let rt = match Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(Error::Io)
        {
            Ok(runtime) => runtime,
            Err(err) => {
                panic!("Async Runtime Error: {:?}", err);
            }
        };
        rt.block_on(async {
            let server = TcpListener::bind(format!("{}:{}", ALL_INTERFACES, port))
                .await
                .map_err(Error::Io)
                .unwrap();
            loop {
                let mut stream = match server.accept().await.map_err(Error::Io) {
                    Ok((stream, _)) => stream,
                    Err(e) => {
                        eprintln!("Couldn't start conversation with traveller: {:?}", e);
                        continue;
                    }
                };
                let itinerary = itinerary.clone();
                rt.spawn(async move {
                    match rx(&mut stream).await {
                        Ok(msg) => {
                            let reply = response_rules(&msg, itinerary);
                            tx(&mut stream, reply).await.unwrap();
                        }
                        Err(err) => {
                            eprintln!("The traveller could not hear my response: {:?}", err);
                        }
                    }
                });
            }
        });
    })
}
