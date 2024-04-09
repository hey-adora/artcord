use artcord_state::message::prod_server_msg::UserTaskState;
use futures::{Future, FutureExt, SinkExt};
use std::{marker::PhantomData, pin::Pin, rc::Rc, sync::Arc, time::Duration};
use thiserror::Error;
use tracing::{debug, error, trace};

use tokio::{
    select, signal,
    sync::{broadcast, mpsc, Mutex},
    task::{spawn_local, JoinError, JoinHandle},
    time,
};
use tokio_util::task::TaskTracker;

type UserTaskType = Arc<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

#[derive(Clone)]
pub enum UserTaskMsg {
    Start,
    Stop,
    SetTask(UserTaskType),
}

#[derive(Clone, Debug)]
pub struct UserTask {
    channel_send: mpsc::Sender<UserTaskMsg>,
}

impl UserTask {
    pub fn new(tracker: TaskTracker) -> Self {
        let aa = async {};
        let aa = aa.boxed();
        let a = || async {};
        let (channel_send, channel_recv) = mpsc::channel::<UserTaskMsg>(1);
        let main_handle = tracker.spawn(Self::main_proc(channel_recv));
        Self { channel_send }
    }

    pub async fn start(&mut self) {
        let send_result = self.channel_send.send(UserTaskMsg::Start).await;
        if let Err(err) = send_result {
            error!("user_task: start error: {}", err);
        }
    }

    pub async fn stop(&mut self) {
        let send_result = self.channel_send.send(UserTaskMsg::Stop).await;
        if let Err(err) = send_result {
            error!("user_task: stop error: {}", err);
        }
    }

    pub async fn set_input_task(
        &mut self,
        input_task: impl Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync + 'static,
    ) {
        let send_result = self
            .channel_send
            .send(UserTaskMsg::SetTask(Arc::new(input_task)))
            .await;
        if let Err(err) = send_result {
            error!("user_task: set input task error: {}", err);
        }
    }

    pub async fn set_output_task(
        &mut self,
        input_task: impl Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync + 'static,
    ) {
        let send_result = self
            .channel_send
            .send(UserTaskMsg::SetTask(Arc::new(input_task)))
            .await;
        if let Err(err) = send_result {
            error!("user_task: set input task error: {}", err);
        }
    }

    async fn main_proc(mut recv: mpsc::Receiver<UserTaskMsg>) {
        let mut user_task: Option<UserTaskType> = None;
        let mut run = false;

        loop {
            if run {
                if let Some(user_task) = user_task.as_mut() {
                    select! {
                        result_input = user_task().boxed() => {}
                        result_settings = recv.recv() => {
                            if let Some(msg) = result_settings {
                                match msg {
                                    UserTaskMsg::Stop => {
                                        run = false;
                                    }
                                    UserTaskMsg::Start => {
                                        run = true;
                                    }
                                    UserTaskMsg::SetTask(new_user_task) => {
                                        *user_task = new_user_task;
                                    }
                                }
                            } else {
                                break;
                            }
                        }
                        _ = signal::ctrl_c() => {
                            break;
                        }
                    }
                    continue;
                } else {
                    error!("user_task: input task is not set");
                }
            }
            select! {
                result_settings = recv.recv() => {
                    if let Some(msg) = result_settings {
                        match msg {
                            UserTaskMsg::Stop => {
                                run = false;
                            }
                            UserTaskMsg::Start => {
                                run = true;
                            }
                            UserTaskMsg::SetTask(new_user_task) => {
                                user_task = Some(new_user_task);
                            }
                        }
                    } else {
                        break;
                    }
                }
                _ = signal::ctrl_c() => {
                    break;
                }
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum UserTaskError {
    #[error("Tokio JoinError error: {0}")]
    JoinError(#[from] JoinError),
}
