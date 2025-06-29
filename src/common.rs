use std::fmt::Display;
use serde::{Deserialize, Serialize};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};

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

// Represents a unit of conversation
#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    header: Header,
    data: Payload 
}

// Contains additional information that's not directly related to the conversation
#[derive(Debug, Serialize, Deserialize)]
pub enum Header {}

// The types of payloads that can be sent between travellers and tour-guides
#[derive(Debug, Serialize, Deserialize)]
pub enum Payload {
    Request(Request),
    Response(Response),
}

// conversational units that a traveller can send to a tour-guide
#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    List,
    Put(Location),
    Del(Location),
    Mov(Location, u32),
    Current,  // where are we now?
    Next,  // where are we going next?
}

// conversational units that a tour-guide can send to a traveller
#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Success,
    Failure,
    List(Vec<Location>),
    Where(Location)
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
pub async fn tx(stream: &mut TcpStream, msg: Message) -> Result<(), Error> {
    // try to serialize the payload
    let content = serde_json::to_string::<Message>(&msg).map_err(Error::Parse)?;

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
pub async fn rx(stream: &mut TcpStream) -> Result<Message, Error> {
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
