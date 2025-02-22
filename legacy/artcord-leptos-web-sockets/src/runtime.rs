use std::marker::PhantomData;
use std::num::{NonZeroU16, NonZeroU64};
use std::rc::Rc;
use std::time::Duration;
use std::{collections::HashMap, fmt::Debug};

use cfg_if::cfg_if;

use chrono::{DateTime, TimeDelta, Utc};
use futures::{SinkExt, StreamExt};
use leptos::*;
use leptos_use::use_window;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, error, info, trace, warn};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{ErrorEvent, MessageEvent, WebSocket};

use crate::channel::{
   WsChannelType, WsRecvResult,
};
use crate::channel_builder::ChannelBuilder;
use crate::{get_ws_url, ConnectError, KeyGen, Receive, Send, WsPackage, WsRouteKey, TIMEOUT_SECS};



// impl<ServerMsg: Clone + Receive + Debug + 'static, ClientMsg: Clone + Send + Debug + 'static> Copy
//     for WsRuntime<ServerMsg, ClientMsg>
// {
// }






#[cfg(test)]
mod test2{
    use crate::{WsRuntime, Send, Receive};
    use wasm_bindgen::JsCast;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[derive(Debug, Clone)]
    enum ServerMsg {
        One
    }

    #[derive(Debug, Clone)]
    enum ClientMsg {
        One
    }

    impl Send for ClientMsg {
        fn send_as_vec(package: &crate::WsPackage<Self>) -> Result<Vec<u8>, String>
            where
                Self: Clone {
            Ok(Vec::new())
        }
    }

    impl Receive for ServerMsg {
        fn recv_from_vec(bytes: &[u8]) -> Result<crate::WsPackage<Self>, String>
            where
                Self: std::marker::Sized + Clone {
            Ok((0, ServerMsg::One))
        }
    }

    #[wasm_bindgen_test::wasm_bindgen_test]
    async fn pass2() {
        console_error_panic_hook::set_once();

        tracing_subscriber::fmt()
            .with_writer(
                // To avoide trace events in the browser from showing their
                // JS backtrace, which is very annoying, in my opinion
                tracing_subscriber_wasm::MakeConsoleWriter::default(), //.map_trace_level_to(tracing::Level::TRACE),
            )
            .with_ansi(false)
            .with_max_level(tracing::Level::TRACE)
            .with_env_filter(
                <tracing_subscriber::EnvFilter as std::str::FromStr>::from_str(
                    "artcord_leptos=trace,artcord_leptos_web_sockets=trace",
                )
                .unwrap(),
            )
            .with_file(true)
            .with_line_number(true)
            .without_time()
            .with_thread_ids(true)
            .with_thread_names(true)
            // For some reason, if we don't do this in the browser, we get
            // a runtime error.
            .init();

        // let _ = tracing_subscriber::fmt()
        // .event_format(
        //     tracing_subscriber::fmt::format()
        //         .with_file(true)
        //         .with_line_number(true),
        // )
        // .with_env_filter(
        //     env::var("RUST_LOG")
        //         .map(|data| tracing_subscriber::EnvFilter::from_str(&data).unwrap())
        //         .unwrap_or(tracing_subscriber::EnvFilter::from_str("artcord=trace").unwrap()),
        // )
        // .try_init();

        let document = leptos::document();
        let test_wrapper = document.create_element("section").unwrap();
        let _ = document.body().unwrap().append_child(&test_wrapper);

        leptos::mount_to(
            test_wrapper.clone().unchecked_into(),
            || leptos::view! { <h1>"aaa"</h1> },
        );

        let (tx, rx) = futures::channel::oneshot::channel::<u32>();

        let runtime = leptos::create_runtime();

        tx.send(69).unwrap();

        let ws = WsRuntime::<ServerMsg, ClientMsg>::new();
        ws.connect(3420).unwrap();

        ws.close();

        let r = rx.await.unwrap();
        tracing::trace!("hello: {r}");

        //runtime.dispose();


        
        //assert_eq!(1, 1);
    }
}


// #[cfg(wasm_bindgen_test::wasm_bindgen_test)]
// mod Tests {
    
// }
