use std::collections::HashMap;

use artcord_leptos_web_sockets::{WsPackage, WsRouteKey};
use artcord_state::{global, backend};
use thiserror::Error;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tracing::debug;
use tracing::{error, trace};


#[derive(Debug, Clone)]
pub struct ThrottleStatsListenerTracker {
    pub cons: HashMap<global::TempConIdType, (WsRouteKey, mpsc::Sender<backend::ConMsg>)>,
}

// #[derive(Debug, Clone)]
// pub enum ConTrackerResult {
//     Success,
//     AlreadyExisted,
// }

impl ThrottleStatsListenerTracker {
    pub fn new() -> Self {
        Self {
             cons: HashMap::new(),
        }
    }

    pub async fn send(
        &mut self,
        msg_org: global::ServerMsg,
    ) -> Result<(), ConTrackerErr> {
        // if self.cons.is_empty() {
        //     return Ok(());
        // }

        let mut to_remove: Vec<global::TempConIdType> = Vec::new();
        trace!("sending {:#?} to listeners: {:#?}", &msg_org, &self.cons);
        for (con_key, (ws_key, tx)) in self.cons.iter() {
            let msg: WsPackage<global::ServerMsg> = (ws_key.clone(), msg_org.clone());
            let msg = global::ServerMsg::as_bytes(msg)?;
            let msg = Message::binary(msg);
            trace!("sending {:#?} to listener: {}", &msg_org, &con_key);
            let send_result = tx.send(backend::ConMsg::Send(msg)).await;
            trace!("finished sending to listener");
            if let Err(err) = send_result {
                debug!(
                    "ws throttle: failed to send on_con update to {} {}",
                    con_key, err
                );
                to_remove.push(*con_key);
            }
        }
        for con_key in to_remove {
            trace!("removing listener: {}", &con_key);
            self.cons.remove(&con_key);
        }
        trace!("sending msg to listeners finished");
        Ok(())
    }

    // pub fn total_blocks() ->

    // pub fn is_empty(&self) -> bool {
    //     self.cons.is_empty()
    // }

    // pub fn add(
    //     &mut self,
    //     con_key: TempConIdType,
    //     ws_key: WsRouteKey,
    //     tx: mpsc::Sender<ConMsg>,
    // ) -> Option<(TempConIdType, mpsc::Sender<ConMsg>)> {
    //     trace!("ws_app: listener added: {}", con_key);

    //     self.cons.insert(con_key, (ws_key, tx))
    // }

    // pub fn remove(
    //     &mut self,
    //     con_key: &TempConIdType,
    // ) -> Option<(TempConIdType, mpsc::Sender<ConMsg>)> {
    //     trace!("ws: con removed: {}", con_key);

    //     self.cons.remove(con_key)
    // }

    // pub fn remove(&mut self) {

    // }
}

#[derive(Error, Debug)]
pub enum ConTrackerErr {
    #[error("MainGallery error: {0}")]
    Serialization(#[from] bincode::Error),
}
