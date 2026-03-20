use tokio::{io, net::TcpListener, sync::mpsc};

use crate::net::{Connection, ConnectionEvent};

pub struct Server {
    address: String,
}

impl Server {
    pub fn new(addr: impl Into<String>) -> Self {
        Self { address: addr.into() }
    }

    pub async fn start<F>(&self, on_connection: F) -> io::Result<()>
    where
        F: Fn(Connection, mpsc::Receiver<ConnectionEvent>, String) + Send + Sync + 'static + Clone,
    {
        let listener = TcpListener::bind(&self.address).await?;

        loop {
            let (socket, addr) = listener.accept().await?;

            let (reader, writer) = socket.into_split();

            let (conn, event_rx) = Connection::new(reader, writer);

            on_connection(conn, event_rx, addr.to_string());
        }
    }
    
    pub fn get_address(&self) -> String {
        self.address.clone()
    }
}
