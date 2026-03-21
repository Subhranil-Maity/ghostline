use tokio::{io, net::TcpListener, sync::mpsc};

use crate::net::{packet::handshake::PeerIdentity, Connection, ConnectionEvent};

pub struct Server {
    address: String,
    local_peer: PeerIdentity,
}

impl Server {
    pub fn new(addr: impl Into<String>, local_peer: PeerIdentity) -> Self {
        Self {
            address: addr.into(),
            local_peer,
        }
    }

    pub async fn start<F>(&self, on_connection: F) -> io::Result<()>
    where
        F: Fn(Connection, mpsc::Receiver<ConnectionEvent>, String) + Send + Sync + 'static + Clone,
    {
        let listener = TcpListener::bind(&self.address).await?;

        loop {
            let (socket, addr) = listener.accept().await?;

            let (reader, writer) = socket.into_split();

            let (conn, event_rx) = Connection::new(reader, writer, self.local_peer.clone());

            on_connection(conn, event_rx, addr.to_string());
        }
    }
    
    pub fn get_address(&self) -> String {
        self.address.clone()
    }
}
