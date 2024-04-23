use std::marker::PhantomData;

use chrono::TimeDelta;
use leptos::{RwSignal, StoredValue};
use std::fmt::Debug;
use web_sys::WebSocket;

use crate::{
    channel::{WsChannel, WsChannelsType, WsRecvResult, WsResourcSendResult},
    Receive, Send, WsError,
};

#[derive(Clone, Debug, Copy)]
pub struct ChannelBuilder<
    ServerMsg: Clone + Receive + Debug + 'static,
    ClientMsg: Clone + Send + Debug + 'static,
> {
    ws_url: StoredValue<Option<String>>,
    channels: WsChannelsType<ServerMsg>,
    ws: StoredValue<Option<WebSocket>>,
    pending_client_msgs: StoredValue<Vec<Vec<u8>>>,
    is_connected: RwSignal<bool>,
    phantom: PhantomData<ClientMsg>,
    prop_timeout: Option<TimeDelta>,
    prop_detach: bool,
    // cancellable: bool,
    prop_persistant: bool,
    prop_single_fire: bool,
    key: Option<u128>,
}

impl<ServerMsg: Clone + Receive + Debug + 'static, ClientMsg: Clone + Send + Debug + 'static>
    ChannelBuilder<ServerMsg, ClientMsg>
{
    pub fn new(
        ws_url: StoredValue<Option<String>>,
        channels: WsChannelsType<ServerMsg>,
        ws: StoredValue<Option<WebSocket>>,
        pending_client_msgs: StoredValue<Vec<Vec<u8>>>,
        is_connected: RwSignal<bool>,
    ) -> Self {
        Self {
            ws_url,
            channels,
            ws,
            pending_client_msgs,
            phantom: PhantomData,
            is_connected,
            prop_timeout: None,
            prop_detach: false,
            // cancellable: false,
            prop_persistant: false,
            prop_single_fire: false,
            key: None,
        }
    }

    pub fn key(mut self, key: u128) -> Self {
        self.key = Some(key);
        self
    }

    pub fn timeout(mut self, timeout_secs: i64) -> Self {
        self.prop_timeout = Some(TimeDelta::microseconds(timeout_secs * 1000 * 1000));
        self
    }

    pub fn detach(mut self) -> Self {
        self.prop_detach = true;
        self
    }

    // pub fn cancellable(mut self) -> Self {
    //     self.cancellable = true;
    //     self
    // }

    pub fn peresistant(mut self) -> Self {
        self.prop_persistant = true;
        self
    }

    pub fn single_fire(mut self) -> Self {
        self.prop_single_fire = true;
        self
    }

    #[track_caller]
    pub fn start(self) -> ChannelInterface<ServerMsg, ClientMsg> {
        let channel = WsChannel::new(
            self.ws_url,
            self.channels,
            self.ws,
            self.pending_client_msgs,
            self.prop_single_fire,
            self.prop_timeout,
            self.prop_persistant,
            self.is_connected,
            self.key,
        );
        ChannelInterface::new(channel)
    }
}

#[derive(Clone, Debug, Copy)]
pub struct ChannelInterface<
    ServerMsg: Clone + Receive + Debug + 'static,
    ClientMsg: Clone + crate::Send + Debug + 'static,
> {
    channel: WsChannel<ServerMsg, ClientMsg>,
}

impl<
        ServerMsg: Clone + Receive + Debug + 'static,
        ClientMsg: Clone + crate::Send + Debug + 'static,
    > ChannelInterface<ServerMsg, ClientMsg>
{
    pub fn new(channel: WsChannel<ServerMsg, ClientMsg>) -> Self {
        Self { channel }
    }
    // on_receive: impl Fn(&WsRecvResult<ServerMsg>) -> bool + 'static,
    // pub fn recv(on_receive: impl Fn(&WsRecvResult<ServerMsg>) -> bool + 'static) {
    //
    // }

    pub fn sender(&self) -> ChannelSendBuilder<ServerMsg, ClientMsg> {
        ChannelSendBuilder::<ServerMsg, ClientMsg>::new(self.channel)
    }

    pub fn recv(&self) -> ChannelRecvBuilder<ServerMsg, ClientMsg> {
        ChannelRecvBuilder::<ServerMsg, ClientMsg>::new(self.channel)
    }
}

// #[derive(Clone, Debug, Copy)]
// pub struct ChannelTimeoutBuilder<
//     ServerMsg: Clone + Receive + Debug + 'static,
//     ClientMsg: Clone + Send + Debug + 'static,
// > {
//     ws_url: StoredValue<Option<String>>,
//     global_msgs_closures: WsChannelsType<ServerMsg>,
//     ws: StoredValue<Option<WebSocket>>,
//     global_pending_client_msgs: StoredValue<Vec<Vec<u8>>>,
//     phantom: PhantomData<ClientMsg>,
// }
//
// impl<ServerMsg: Clone + Receive + Debug + 'static, ClientMsg: Clone + Send + Debug + 'static>
//     ChannelTimeoutBuilder<ServerMsg, ClientMsg>
// {
//     pub fn new(
//         ws_url: StoredValue<Option<String>>,
//         global_msgs_closures: WsChannelsType<ServerMsg>,
//         ws: StoredValue<Option<WebSocket>>,
//         global_pending_client_msgs: StoredValue<Vec<Vec<u8>>>,
//     ) -> Self {
//         Self {
//             ws_url,
//             global_msgs_closures,
//             ws,
//             global_pending_client_msgs,
//             phantom: PhantomData,
//         }
//     }
//
//     pub fn basic(self) -> ChannelModeBuilder<ServerMsg, ClientMsg> {
//         ChannelModeBuilder::new(self, None)
//     }
//
//     pub fn timeout(self, timeout_secs: i64) -> ChannelModeBuilder<ServerMsg, ClientMsg> {
//         ChannelModeBuilder::new(
//             self,
//             Some(TimeDelta::microseconds(timeout_secs * 1000 * 1000)),
//         )
//     }
// }
//
// // #[derive(Clone, Debug, Copy)]
// // pub struct ChannelTimeoutBuilder<
// //     ServerMsg: Clone + Receive + Debug + 'static,
// //     ClientMsg: Clone + crate::Send + Debug + 'static,
// // > {
// //     ws_builder: WsBuilder<ServerMsg, ClientMsg>,
// //     timeout: Option<TimeDelta>,
// // }
// //
// // impl<
// //         ServerMsg: Clone + Receive + Debug + 'static,
// //         ClientMsg: Clone + crate::Send + Debug + 'static,
// //     > ChannelTimeoutBuilder<ServerMsg, ClientMsg>
// // {
// //     pub fn new(ws_builder: WsBuilder<ServerMsg, ClientMsg>, timeout: Option<TimeDelta>) -> Self {
// //         Self {
// //             ws_builder,
// //             timeout,
// //         }
// //     }
// //     pub fn multi(mut self) -> Self {
// //         self.skip_if_awaiting_response = false;
// //         self
// //     }
// //
// //     #[track_caller]
// //     pub fn build(self) -> ChannelWithTimeoutInterface<ServerMsg, ClientMsg> {
// //         let channel = WsChannel::new(
// //             self.ws_builder.ws_url,
// //             self.ws_builder.global_msgs_closures,
// //             self.ws_builder.ws,
// //             self.ws_builder.global_pending_client_msgs,
// //             self.skip_if_awaiting_response,
// //             self.timeout,
// //         );
// //         ChannelWithTimeoutInterface::new(channel)
// //     }
// // }
//
// #[derive(Clone, Debug, Copy)]
// pub struct ChannelModeBuilder<
//     ServerMsg: Clone + Receive + Debug + 'static,
//     ClientMsg: Clone + crate::Send + Debug + 'static,
// > {
//     ws_builder: ChannelTimeoutBuilder<ServerMsg, ClientMsg>,
//     skip_if_awaiting_response: bool,
//     timeout: Option<TimeDelta>,
// }
//
// impl<
//         ServerMsg: Clone + Receive + Debug + 'static,
//         ClientMsg: Clone + crate::Send + Debug + 'static,
//     > ChannelModeBuilder<ServerMsg, ClientMsg>
// {
//     pub fn new(
//         ws_builder: ChannelTimeoutBuilder<ServerMsg, ClientMsg>,
//         timeout: Option<TimeDelta>,
//     ) -> Self {
//         Self {
//             ws_builder,
//             skip_if_awaiting_response: false,
//             timeout,
//         }
//     }
//     pub fn single_fire(mut self) -> Self {
//         self.skip_if_awaiting_response = true;
//         self
//     }
//
//     pub fn multi(mut self) -> Self {
//         self.skip_if_awaiting_response = false;
//         self
//     }
//
//     #[track_caller]
//     pub fn build(self) -> ChannelWithTimeoutInterface<ServerMsg, ClientMsg> {
//         let channel = WsChannel::new(
//             self.ws_builder.ws_url,
//             self.ws_builder.global_msgs_closures,
//             self.ws_builder.ws,
//             self.ws_builder.global_pending_client_msgs,
//             self.skip_if_awaiting_response,
//             self.timeout,
//         );
//         ChannelWithTimeoutInterface::new(channel)
//     }
// }
//
// #[derive(Clone, Debug, Copy)]
// pub struct ChannelCancelBuilder<
//     ServerMsg: Clone + Receive + Debug + 'static,
//     ClientMsg: Clone + crate::Send + Debug + 'static,
// > {
//     ws_builder: ChannelTimeoutBuilder<ServerMsg, ClientMsg>,
//     skip_if_awaiting_response: bool,
//     timeout: Option<TimeDelta>,
//     cancellable: bool,
// }
//
// impl<
//         ServerMsg: Clone + Receive + Debug + 'static,
//         ClientMsg: Clone + crate::Send + Debug + 'static,
//     > ChannelModeBuilder<ServerMsg, ClientMsg>
// {
//     pub fn new(
//         ws_builder: ChannelTimeoutBuilder<ServerMsg, ClientMsg>,
//         timeout: Option<TimeDelta>,
//     ) -> Self {
//         Self {
//             ws_builder,
//             skip_if_awaiting_response: false,
//             timeout,
//         }
//     }
//     pub fn cancellable(mut self) -> Self {
//         self.skip_if_awaiting_response = true;
//         self
//     }
//
//     // pub fn multi(mut self) -> Self {
//     //     self.skip_if_awaiting_response = false;
//     //     self
//     // }
//
//     #[track_caller]
//     pub fn build(self) -> ChannelWithTimeoutInterface<ServerMsg, ClientMsg> {
//         let channel = WsChannel::new(
//             self.ws_builder.ws_url,
//             self.ws_builder.global_msgs_closures,
//             self.ws_builder.ws,
//             self.ws_builder.global_pending_client_msgs,
//             self.skip_if_awaiting_response,
//             self.timeout,
//         );
//         ChannelWithTimeoutInterface::new(channel)
//     }
// }
//
#[derive(Clone, Debug, Copy)]
pub struct ChannelRecvBuilder<
    ServerMsg: Clone + Receive + Debug + 'static,
    ClientMsg: Clone + crate::Send + Debug + 'static,
> {
    channel: WsChannel<ServerMsg, ClientMsg>,
    persistant: bool,
}

impl<
        ServerMsg: Clone + Receive + Debug + 'static,
        ClientMsg: Clone + crate::Send + Debug + 'static,
    > ChannelRecvBuilder<ServerMsg, ClientMsg>
{
    pub fn new(channel: WsChannel<ServerMsg, ClientMsg>) -> Self {
        Self {
            channel,
            persistant: false,
        }
    }

    pub fn persistant(mut self) -> Self {
        self.persistant = true;
        self
    }

    // pub fn send(&self, msg: ClientMsg) {
    //     self.channel.send(client_msg)
    // }

    #[track_caller]
    pub fn start(&self, on_receive: impl Fn(&WsRecvResult<ServerMsg>, &mut bool) + 'static) {
        self.channel.start_recv(on_receive, self.persistant)
    }
}

#[derive(Clone, Debug, Copy)]
pub struct ChannelSendBuilder<
    ServerMsg: Clone + Receive + Debug + 'static,
    ClientMsg: Clone + crate::Send + Debug + 'static,
> {
    channel: WsChannel<ServerMsg, ClientMsg>,
    prop_farewell: Option<ClientMsg>,
    prop_resend_on_reconnect: bool,
    // detach: bool,
}

impl<
        ServerMsg: Clone + Receive + Debug + 'static,
        ClientMsg: Clone + crate::Send + Debug + 'static,
    > ChannelSendBuilder<ServerMsg, ClientMsg>
{
    pub fn new(channel: WsChannel<ServerMsg, ClientMsg>) -> Self {
        Self {
            channel,
            prop_farewell: None,
            prop_resend_on_reconnect: false,
            // detach: false,
        }
    }

    pub fn on_cleanup(mut self, msg: ClientMsg) -> Self {
        self.prop_farewell = Some(msg);
        self
    }

    pub fn resend_on_reconnect(mut self) -> Self {
        self.prop_resend_on_reconnect = true;
        self
    }

    // pub fn detach(mut self) -> Self {
    //     self.detach = true;
    //     self
    // }

    // pub fn send(&self, msg: ClientMsg) {
    //     self.channel.send(client_msg)
    // }

    #[track_caller]
    pub fn send(self, msg: ClientMsg) -> Result<WsResourcSendResult, WsError> {
        self.channel
            .send(msg, self.prop_farewell, self.prop_resend_on_reconnect)
    }
}
//

// end - timeout channel
