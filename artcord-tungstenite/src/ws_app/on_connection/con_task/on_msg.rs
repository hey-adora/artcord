use futures::{stream::SplitSink, SinkExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use tracing::{debug, trace};

use super::ConMsg;

pub async fn on_msg(
    msg: Option<ConMsg>,
    client_out: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
) -> bool {
    let Some(msg) = msg else {
        trace!("connection msg channel closed");
        return true;
    };

    match msg {
        ConMsg::Send(msg) => {
            let send_result = client_out.send(msg).await;
            if let Err(err) = send_result {
                debug!("failed to send msg: {}", err);
                return true;
            }
        }
        ConMsg::Stop => {
            return true;
        }
    }

    false
}
