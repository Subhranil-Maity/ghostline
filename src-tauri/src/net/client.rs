use tokio::{io, net::TcpStream, sync::mpsc};

use crate::net::{packet::handshake::PeerIdentity, Connection, ConnectionEvent};

pub struct Client {
    address: String,
    local_peer: PeerIdentity,
}

impl Client {
    pub fn new(addr: impl Into<String>, local_peer: PeerIdentity) -> Self {
        Self {
            address: addr.into(),
            local_peer,
        }
    }

    // pub async fn connect<F>(&self) -> io::Result<Connection>
    pub async fn connect(&self) -> io::Result<(Connection, mpsc::Receiver<ConnectionEvent>)>
    // where
    //     F: Fn(String) + Send + Sync + 'static,
    {
        let stream = TcpStream::connect(&self.address).await?;

        let (reader, writer) = stream.into_split();

        let conn = Connection::new(reader, writer, self.local_peer.clone());

        Ok(conn)
    }
}
