use tokio::{io, net::TcpStream, sync::mpsc};

use crate::net::{Connection, ConnectionEvent};

pub struct Client {
    address: String,
}

impl Client {
    pub fn new(addr: impl Into<String>) -> Self {
        Self { address: addr.into() }
    }

    // pub async fn connect<F>(&self) -> io::Result<Connection>
    pub async fn connect(&self) -> io::Result<(Connection, mpsc::Receiver<ConnectionEvent>)>
    // where
    //     F: Fn(String) + Send + Sync + 'static,
    {
        let stream = TcpStream::connect(&self.address).await?;

        let (reader, writer) = stream.into_split();

        let conn = Connection::new(reader, writer);

        Ok(conn)
    }
}
