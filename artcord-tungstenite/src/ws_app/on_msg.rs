use crate::ws_app::ws_throttle::WsThrottle;
use crate::ws_app::WsAppMsg;
use tracing::trace;

pub async fn on_ws_msg(msg: Option<WsAppMsg>, throttle: &mut WsThrottle) -> bool {
    let Some(msg) = msg else {
        trace!("ws_recv channel closed");
        return true;
    };
    match msg {
        WsAppMsg::Disconnected(ip) => {
            throttle.disconnected(&ip).await;
        }
        WsAppMsg::Stop => {
            return true;
        }
        WsAppMsg::AddListener { connection_key, tx, ws_key } => {
            trace!("ws_app: listener added: {}", connection_key);
            throttle.listener_list.insert(connection_key, (ws_key, tx));
        }
        WsAppMsg::RemoveListener { connection_key } => {
            trace!("ws_app: listener removed: {}", connection_key);
            throttle.listener_list.remove(&connection_key);
        }
    }
    false
}
