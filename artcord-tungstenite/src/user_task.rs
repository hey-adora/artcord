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

type InputFnType<InputType> =
    Arc<dyn Fn() -> Pin<Box<dyn Future<Output = InputType> + Send>> + Send + Sync>;
type OutputFnType<InputType> =
    Arc<dyn Fn(InputType) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

#[derive(Clone)]
pub enum UserTaskMsg<
    InputType: Send + Sized + Clone, // InputTaskReturnType: Send,
                                     // // OutputTaskReturnType: Send,
                                     // OutputTaskFutureType: Future<Output = ()> + Send,
                                     // OutputTaskType: Fn(InputTaskReturnType) -> OutputTaskFutureType + Send + Clone + 'static,
                                     // InputFutureType: Future<Output = InputTaskReturnType> + Send,
                                     // InputTaskType: Fn() -> InputFutureType + Send + Clone + 'static,
> {
    Start,
    Stop,
    SetInputTask(InputFnType<InputType>),
    SetOutputTask(OutputFnType<InputType>),
}

// pub struct Handle {
//     send: mpsc::Sender<bool>,
// }
//
// impl Handle {
//     pub fn new() -> (Self, mpsc::Receiver<bool>) {
//         let (send, recv) = mpsc::channel::<bool>(1);
//         (Self { send }, recv)
//     }
// }

#[derive(Clone, Debug)]
pub struct UserTask<
    InputType: Send + Sized + Clone,
    // InputTaskReturnType: Send,
    // // OutputTaskReturnType: Send,
    // OutputTaskFutureType: Future<Output = ()> + Send,
    // OutputTaskType: Fn(InputTaskReturnType) -> OutputTaskFutureType + Send + Clone + 'static,
    // InputFutureType: Future<Output = InputTaskReturnType> + Send,
    // InputTaskType: Fn() -> InputFutureType + Send + Clone + 'static,
> {
    // tracker: TaskTracker,
    // // task_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    // task_handle: Option<JoinHandle<()>>,
    // main_handle: JoinHandle<()>,
    // // channel_recv: mpsc::Receiver<bool>,
    channel_send: mpsc::Sender<
        UserTaskMsg<
            InputType, // InputTaskReturnType,
                       // OutputTaskFutureType,
                       // OutputTaskType,
                       // InputFutureType,
                       // InputTaskType,
        >,
    >,
    // callback: Option<OutputTaskType>,
    // interval: Duration,
    // input_task: PhantomData<InputTaskType>,
}

// impl<R, T: Future<Output = R> + Send, C: Fn() -> T + Send + Clone + 'static> Clone
//     for UserTask<R, T, C>
// {
//     fn clone(&self) -> Self {
//         Self {
//             tracker: self.tracker.clone(),
//             handle: self.handle.clone(),
//             channel_recv: self.channel_recv.resubscribe(),
//             channel_send: self.channel_send.clone(),
//             callback: self.callback.clone(),
//             interval: self.interval.clone(),
//         }
//     }
// }

impl<InputType: Send + Sized + Clone + 'static>
    UserTask<
        InputType, // InputTaskReturnType,
                   // // OutputTaskReturnType,
                   // OutputTaskFutureType,
                   // OutputTaskType,
                   // InputFutureType,
                   // InputTaskType,
    >
{
    pub fn new(tracker: TaskTracker) -> Self {
        let aa = async {};
        let aa = aa.boxed();
        let a = || async {};
        let (channel_send, channel_recv) = mpsc::channel::<
            UserTaskMsg<
                InputType, // InputTaskReturnType,
                           // OutputTaskFutureType,
                           // OutputTaskType,
                           // InputFutureType,
                           // InputTaskType,
            >,
        >(1);
        let main_handle = tracker.spawn(Self::main_proc(channel_recv));
        Self {
            // tracker,
            // task_handle: Arc::new(Mutex::new(None)),
            // main_handle,
            channel_send,
            // callback: None,
            // interval,
            // input_task: PhantomData,
        }
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

    // async fn f3() {}
    // // async fn set_input_task<A: Future<Output = InputType> + Send, F: Fn() -> A>(
    // //     &self,
    // //     input_task: F,
    // // ) {
    // // }
    //
    // async fn my_fn2<A: Future<Output = ()> + Send, F: Fn() -> A>(&self, task: F) {
    //     // spawn_local()
    //     let u = Self::f3;
    //     let storage: Rc<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>>> = Rc::new(task);
    // }

    pub async fn set_input_task(
        &mut self,
        input_task: impl Fn() -> Pin<Box<dyn Future<Output = InputType> + Send>> + Send + Sync + 'static,
    ) {
        // input_task().await;
        // let oof = Rc::new(input_task);
        // oof().await;

        // let f = input_task;
        // let f = Rc::new(f);
        // let f2 = Rc::new(|| async {});
        // let f3 = Rc::new(Self::f3);
        let send_result = self
            .channel_send
            .send(UserTaskMsg::SetInputTask(Arc::new(input_task)))
            .await;
        if let Err(err) = send_result {
            error!("user_task: set input task error: {}", err);
        }
    }

    pub async fn set_output_task(
        &mut self,
        input_task: impl Fn(InputType) -> Pin<Box<dyn Future<Output = ()> + Send>>
            + Send
            + Sync
            + 'static,
    ) {
        // input_task().await;
        // let oof = Rc::new(input_task);
        // oof().await;

        // let f = input_task;
        // let f = Rc::new(f);
        // let f2 = Rc::new(|| async {});
        // let f3 = Rc::new(Self::f3);
        let send_result = self
            .channel_send
            .send(UserTaskMsg::SetOutputTask(Arc::new(input_task)))
            .await;
        if let Err(err) = send_result {
            error!("user_task: set input task error: {}", err);
        }
    }

    // async fn set_output_task(&self, input_task: OutputTaskType) {
    //     let send_result = self
    //         .channel_send
    //         .send(UserTaskMsg::SetOutputTask(input_task))
    //         .await;
    //     if let Err(err) = send_result {
    //         error!("user_task: set output task error: {}", err);
    //     }
    // }

    async fn main_proc(
        mut recv: mpsc::Receiver<
            UserTaskMsg<
                InputType, // InputTaskReturnType,
                           // OutputTaskFutureType,
                           // OutputTaskType,
                           // InputFutureType,
                           // InputTaskType,
            >,
        >,
    ) {
        let mut input_task: Option<InputFnType<InputType>> = None;
        let mut output_task: Option<OutputFnType<InputType>> = None;
        let mut run = false;
        // let mut handle: Option<JoinHandle<OutputTaskReturnType>> = None;
        // let proc = move |msg: UserTaskMsg<
        //     InputTaskReturnType,
        //     OutputTaskFutureType,
        //     OutputTaskType,
        //     InputFutureType,
        //     InputTaskType,
        // >| async move {
        //     // match desired_state {
        //     //     false if handle.is_some() => {}
        //     //     true => {}
        //     // }
        //
        //     match msg {
        //         UserTaskMsg::Stop => {
        //             run = false;
        //         }
        //         UserTaskMsg::Start => {
        //             run = true;
        //         }
        //         UserTaskMsg::SetInputTask(new_input_task) => {
        //             input_task = Some(new_input_task);
        //         }
        //         UserTaskMsg::SetOutputTask(new_output_task) => {
        //             output_task = Some(new_output_task);
        //         }
        //     }
        //     // if desired_state {
        //     //     // if handle.is_none() {
        //     //     //     if let Some(output_task) = output_task {
        //     //     //         handle = Some(tracker.spawn(output_task()));
        //     //     //     }
        //     //     // }
        //     // } else {
        //     // }
        // };
        loop {
            if run {
                if let Some(input_task) = input_task.as_mut() {
                    if let Some(output_task) = output_task.as_mut() {
                        select! {
                            result_input = input_task().boxed() => {
                                output_task(result_input).await;
                            }
                            result_settings = recv.recv() => {
                                if let Some(msg) = result_settings {
                                    match msg {
                                        UserTaskMsg::Stop => {
                                            run = false;
                                        }
                                        UserTaskMsg::Start => {
                                            run = true;
                                        }
                                        UserTaskMsg::SetInputTask(new_input_task) => {
                                            *input_task = new_input_task;
                                        }
                                        UserTaskMsg::SetOutputTask(new_output_task) => {
                                            *output_task = new_output_task;
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
                        error!("user_task: output task is not set");
                    }
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
                            UserTaskMsg::SetInputTask(new_input_task) => {
                                input_task = Some(new_input_task);
                            }
                            UserTaskMsg::SetOutputTask(new_output_task) => {
                                output_task = Some(new_output_task);
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

    // pub fn set_task() {}
    //
    // pub async fn toggle(&mut self, desired_state: bool) -> Result<UserTaskState, UserTaskError> {
    //     let current_state = &mut *self.task_handle.lock().await;
    //     if desired_state {
    //         Ok(self.start(current_state).await)
    //     } else {
    //         self.stop(current_state).await
    //     }
    // }
    //
    // pub async fn clean_up(&self) -> Result<UserTaskState, UserTaskError> {
    //     let current_state = &mut *self.task_handle.lock().await;
    //     self.stop(current_state).await
    // }
    //
    // async fn stop(
    //     &self,
    //     current_state: &mut Option<JoinHandle<()>>,
    // ) -> Result<UserTaskState, UserTaskError> {
    //     if current_state.is_some() {
    //         if let Some(handle) = current_state {
    //             let send_result = self.channel_send.send(true);
    //             if let Err(err) = send_result {
    //                 trace!("user_task: send cancel err: {}", err);
    //             }
    //             handle.await?;
    //             *current_state = None;
    //             trace!("user_task: stopped");
    //             Ok(UserTaskState::Stopped)
    //         } else {
    //             trace!("user_task: already stopped");
    //             Ok(UserTaskState::AlreadyStopped)
    //         }
    //     } else {
    //         trace!("user_task: already  stopped");
    //         Ok(UserTaskState::AlreadyStopped)
    //     }
    // }
    //
    // async fn start(&self, current_state: &mut Option<JoinHandle<()>>) -> UserTaskState {
    //     let Some(callback) = &self.callback else {
    //         return UserTaskState::TaskIsNotSet;
    //     };
    //
    //     if current_state.is_some() {
    //         trace!("user_task: already open");
    //         return UserTaskState::AlreadyStarted;
    //     }
    //     let handle = self.tracker.spawn({
    //         let mut interval = time::interval(self.interval);
    //         let channel_recv = self.channel_recv.resubscribe();
    //         let callback = callback.clone();
    //         async move {
    //             let cancel = |mut channel_recv: broadcast::Receiver<bool>| async move {
    //                 loop {
    //                     let result = channel_recv.recv().await;
    //                     match result {
    //                         Ok(result) => {
    //                             if result {
    //                                 break;
    //                             }
    //                         }
    //                         Err(err) => match err {
    //                             broadcast::error::RecvError::Lagged(_) => {
    //                                 trace!("user_task:lagged");
    //                             }
    //                             broadcast::error::RecvError::Closed => {
    //                                 break;
    //                             }
    //                         },
    //                     }
    //                 }
    //             };
    //             loop {
    //                 select! {
    //                     _ = interval.tick() => {
    //                         callback().await;
    //                     },
    //                     _ = cancel(channel_recv.resubscribe()) => {
    //                         debug!("user_task: closed");
    //                         break;
    //                     },
    //                 };
    //             }
    //         }
    //     });
    //     *current_state = Some(handle);
    //     trace!("user_task: opened");
    //     UserTaskState::Started
    // }
}

#[derive(Error, Debug)]
pub enum UserTaskError {
    #[error("Tokio JoinError error: {0}")]
    JoinError(#[from] JoinError),
}
