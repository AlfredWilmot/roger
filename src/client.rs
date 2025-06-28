use std::time::Duration;

use tokio::net::TcpStream;

static RETRY_DELAY: u64 = 10; //ms

// Someone looking for answers
pub struct Traveller {}

impl Traveller {

    // Try to get the travel-guide's attention
    pub async fn connect(url: &str, port: u16) -> TcpStream {
        let addr = &format!("{}:{}", url, port)[..];

        let conversation = loop {
            match TcpStream::connect(addr).await {
                Ok(convo) => break convo,
                Err(err) => match err.kind() {
                    std::io::ErrorKind::ConnectionRefused => {
                        tokio::time::sleep(Duration::from_millis(RETRY_DELAY)).await;
                    }
                    _ => {
                        panic!("Seems we have a breakdown in communication! {:?}", err);
                    }
                },
            }
        };
        conversation
    }
}
