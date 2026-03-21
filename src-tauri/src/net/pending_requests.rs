use std::collections::HashMap;
use tokio::sync::{Mutex, oneshot};

use crate::net::packet::responce::ResponsePacket;


type RequestId = u64;

struct PendingRequests {
    map: Mutex<HashMap<RequestId, oneshot::Sender<ResponsePacket>>>,
}
