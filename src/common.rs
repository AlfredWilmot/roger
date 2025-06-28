use std::fmt::Display;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};
use serde::{Deserialize, Serialize};

pub static LOCALHOST: &str = "127.0.0.1";
pub static ALL_INTERFACES: &str = "0.0.0.0";

#[derive(Default, Debug, Serialize, Deserialize)]
// All the places the tour-guide knows about
pub enum Location {
    #[default]
    HOME,
    CITY,
    WOODS,
    BEACH,
    FIELD,
    CAFE,
    SHOP,
    CHURCH,
}

// Actions that a traveller can send to the tour-guide to interact with the itinerary
pub enum Request {
    Put(Location),
    Del(Location),
    Mov(Location, u32),
    List,
    Current,  // where are we now?
    Next,  // where are we going next?
}

/// Used to handle all relevant error-states using '?' short-circuiting
#[derive(Debug)]
pub enum Error {
    Parse(serde_json::Error),
    Url(std::net::AddrParseError),
    Io(std::io::Error),
}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for Error {}

/// Transmit data over a TcpStream
/// the first 4 bytes correspond to the byte-count of the serialized data
pub async fn transmit(stream: &mut TcpStream, msg: Location) -> Result<(), Error> {
    // try to serialize the payload
    let content = serde_json::to_string::<Location>(&msg).map_err(Error::Parse)?;

    // tell the receiver how large the payload is
    stream
        .write_all(&(content.len() as u32).to_be_bytes())
        .await
        .map_err(Error::Io)?;

    // then send the actual payload
    stream
        .write_all(content.as_bytes())
        .await
        .map_err(Error::Io)?;

    // ensure all data is sent
    stream.flush().await.map_err(Error::Io)?;

    Ok(())
}

/// Receive data over a TcpStream
/// assume the byte-count of the data to be deserialized is provided in the first 4 bytes
pub async fn receive(stream: &mut TcpStream) -> Result<Location, Error> {
    // 4 bytes encode the payload size
    let mut content_length_buffer: [u8; 4] = [0; 4];
    stream
        .read_exact(&mut content_length_buffer)
        .await
        .map_err(Error::Io)?;
    let content_length = u32::from_be_bytes(content_length_buffer) as usize;

    // determine the remaining bytes to read using the size
    let mut content_buffer: Vec<u8> = vec![0; content_length];
    stream
        .read_exact(&mut content_buffer)
        .await
        .map_err(Error::Io)?;

    // ensure all data is sent
    stream.flush().await.map_err(Error::Io)?;

    // decode and return the response to the caller
    serde_json::from_slice(&content_buffer).map_err(Error::Parse)
}
