use std::{sync::Arc, time::Duration};

use artcord_leptos_web_sockets::{WsPackage, WsRouteKey};
use artcord_mongodb::database::DB;
use artcord_state::message::{
    prod_perm_key::ProdMsgPermKey,
    prod_server_msg::{AdminThrottleResponse, ServerMsg},
};
use futures::channel::oneshot::Cancellation;
use thiserror::Error;
use tokio::{
    select,
    sync::{broadcast, oneshot, Mutex, RwLock},
    task::{JoinError, JoinHandle},
    time::{self, sleep},
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{debug, trace};

pub async fn ws_hadnle_admin_throttle(
    db: Arc<DB>,
    listener_state: bool,
    task_tracker: TaskTracker,
    is_admin_throttle_listener_active: Arc<Mutex<Option<JoinHandle<()>>>>,
    mut cancel_recv: broadcast::Receiver<bool>,
    cancel_send: broadcast::Sender<bool>,
    // admin_throttle_listener_recv_close: oneshot::Receiver<bool>,
) -> Result<
    artcord_state::message::prod_server_msg::AdminThrottleResponse,
    WsHandleAdminThrottleError,
> {
    let is_open = &mut *is_admin_throttle_listener_active.lock().await;
    if listener_state {
        if is_open.is_some() {
            trace!("ws_admin_throttle: listener already open");
            Ok(AdminThrottleResponse::AlreadyStarted)
        } else {
            trace!("ws_admin_throttle: listener opened");
            let handle = task_tracker.spawn({
                // let cancelation_token = cancelation_token.clone();
                async move {
                    let mut interval = time::interval(Duration::from_secs(1));
                    let main = move || async move {
                        debug!("sending stuff");
                    };

                    let cancel = |mut cancel_recv: broadcast::Receiver<bool>| async move {
                        loop {
                            let result = cancel_recv.recv().await;
                            match result {
                                Ok(result) => {
                                    if result {
                                        break;
                                    }
                                }
                                Err(err) => match err {
                                    broadcast::error::RecvError::Lagged(_) => {
                                        trace!("ws_admin_throttle: cancel lagged");
                                    }
                                    broadcast::error::RecvError::Closed => {
                                        break;
                                    }
                                },
                            }
                        }
                    };
                    loop {
                        select! {
                            _ = interval.tick() => {
                                main().await;
                            },
                            _ = cancel(cancel_recv.resubscribe()) => {
                                debug!("SHOULD BE CANCELED");
                                break;
                            },
                        };
                    }

                    // admin_throttle_listener_recv_close.await;
                }
            });
            *is_open = Some(handle);
            Ok(AdminThrottleResponse::Started)
        }
    } else {
        if is_open.is_some() {
            trace!("ws_admin_throttle: listener stopped");
            // task_tracker.close();
            // task_tracker.wait().await;
            if let Some(handle) = is_open {
                cancel_send.send(true);
                // cancel_recv.cancel();
                handle.await?;
                *is_open = None;
                Ok(AdminThrottleResponse::Stopped)
            } else {
                trace!("ws_admin_throttle: listener stopped");
                Ok(AdminThrottleResponse::AlreadyStopped)
            }
        } else {
            trace!("ws_admin_throttle: listener stopped");
            Ok(AdminThrottleResponse::AlreadyStopped)
        }
    }
    // let result = db.user_find_one(&user_id).await?;
    //
    // let Some(result) = result else {
    //     return Ok(artcord_state::message::prod_server_msg::UserResponse::UserNotFound);
    // };
}

#[derive(Error, Debug)]
pub enum WsHandleAdminThrottleError {
    #[error("Mongodb error: {0}")]
    MongoDB(#[from] mongodb::error::Error),

    #[error("Tokio JoinError error: {0}")]
    JoinError(#[from] JoinError),
}
