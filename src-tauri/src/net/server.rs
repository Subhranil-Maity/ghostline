use std::sync::Arc;

use tokio::{io, net::TcpListener};

use crate::net::Connection;

pub struct Server {
    address: String,
}

impl Server {
    pub fn new(addr: impl Into<String>) -> Self {
        Self { address: addr.into() }
    }

    pub async fn start<F>(&self, on_connection: F) -> io::Result<()>
    where
        F: Fn(Connection, String) + Send + Sync + 'static + Clone,
    {
        let listener = TcpListener::bind(&self.address).await?;

        loop {
            let (socket, addr) = listener.accept().await?;

            let (reader, writer) = socket.into_split();

            let conn = Connection::new(reader, writer);

            on_connection(conn, addr.to_string());
        }
    }
    
    pub fn get_address(&self) -> String {
        self.address.clone()
    }
}
