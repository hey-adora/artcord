// pub mod app;
// pub mod error_template;
// pub mod errors;
// #[cfg(feature = "ssr")]
// pub mod middleware;
use tracing::{error, trace};

use leptos::{prelude::*, task::spawn_local};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <meta name="color-scheme" content="dark light"/>
                <link rel="shortcut icon" type="image/ico" href="/favicon.ico"/>
                <link rel="stylesheet" id="leptos" href="/pkg/heyadora_art.css"/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    //let local_resource = LocalResource::new(move || get_server_data());

    let data_resource = OnceResource::new(get_server_data());
    let data_resource2 = Resource::new(|| (), |_| get_server_data());
    let data_resource3 = Resource::new(|| (), |_| async move { () });

    let (get_data, set_data) = signal::<String>(String::from("loading..."));
    Effect::new(move || {
        spawn_local(async move {
            let data = get_server_data().await.unwrap_or("err".to_string());
            set_data.set(data);
            error!("hello");
            trace!("wtf");
        });
    });
    // let get_data = move || {
    //     match data_resource.get() {
    //         Some(data) => {
    //             match data.as_deref() {
    //                 Ok(data) => {
    //                     data.to_string()
    //                 }
    //                 Err(err) => String::from("server error")
    //             }
    //         }
    //         None => String::from("loading...")
    //     }
    // };

    view! {
        <main>
            <nav class="text-gray-">
                <h3>"hello"</h3>
            </nav>
        </main>
        // <div class="bg-green-600">"hello9e0"</div>
        // <div class="bg-dark-night">"hello9e0"</div>
        // // <div class="bg-red-600">{ get_data }</div>
        // <Suspense
        //     fallback=move || view! {"hello"}
        // >
        //     <div class="bg-blue-600">{ move || data_resource.get() }</div>
        // </Suspense>

        // <Await
        //     future=get_server_data()
        //     let:data
        //     >
        //     { data.clone().unwrap_or(String::from("loading")) }
        // </Await>

       // <div class="bg-green-600">{ move || local_resource.get().as_deref().map(|v| v.clone()).unwrap_or(Ok("loading".to_string())) }</div>
    }
}

#[server(endpoint = "/wtf")]
pub async fn get_server_data() -> Result<String, ServerFnError> {
    Ok(std::fs::read_to_string("/home/hey/github/artcord/Dockerfile").unwrap())
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use tracing::{span, trace_span};

    console_error_panic_hook::set_once();
    wasmlog::simple_logger_init();
    trace!("one");

    {
        trace_span!("huh");
        trace!("zero");
    }
    let span1 = tracing::span!(tracing::Level::TRACE, "HANDLE-RECV",).entered();

    trace!("two");
    {
        let span2 = tracing::span!(tracing::Level::TRACE, "WOWZA",).entered();

        trace!("three");

        span2.exit();
    }
    span1.exit();
    leptos::mount::hydrate_body(App);
}

pub mod wasmlog {
    // use std::io;
    // use std::io::Write;

    use wasm_bindgen::prelude::*;

    #[derive(Debug, Clone)]
    struct Spanner(u32);

    impl Default for Spanner {
        fn default() -> Self {
            Self(0)
        }
    }

    // pub struct MakeConsoleWriter;

    // impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for MakeConsoleWriter {
    //     type Writer = ConsoleWriter;

    //     fn make_writer(&'a self) -> Self::Writer {
    //         ConsoleWriter::default()
    //     }

    //     fn make_writer_for(&'a self, meta: &tracing::Metadata<'_>) -> Self::Writer {
    //         ConsoleWriter::default()
    //     }
    //   }

    // pub struct ConsoleWriter(tracing::Level, Vec<u8>);

    // impl Default for ConsoleWriter {
    //     fn default() -> Self {
    //         Self(tracing::Level::TRACE, Vec::new())
    //     }
    // }

    // impl io::Write for ConsoleWriter {
    //     fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    //         self.1.write(buf)
    //     }

    //     fn flush(&mut self) -> io::Result<()> {
    //         use tracing::Level;

    //         let data = std::str::from_utf8(&self.1)
    //             .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "data not UTF-8"))?;

    //         // log1("wtf 1".to_string());
    //         // log1(data.to_string());
    //         // leptos::logging::
    //         // log5(
    //         //     format!("%c{level}%c{spans_combined}%c{target}{origin}%c: {value}"),
    //         //     match *level {
    //         //         tracing::Level::TRACE => "color: dodgerblue; background: #444",
    //         //         tracing::Level::DEBUG => "color: lawngreen; background: #444",
    //         //         tracing::Level::INFO => "color: whitesmoke; background: #444",
    //         //         tracing::Level::WARN => "color: orange; background: #444",
    //         //         tracing::Level::ERROR => "color: red; background: #444",
    //         //     },
    //         //     "color: inherit; font-weight: bold",
    //         //     "color: gray; font-style: italic",
    //         //     "color: inherit",
    //         // );

    //         match self.0 {
    //             Level::TRACE => gloo::console::trace!(data),
    //             Level::DEBUG => gloo::console::debug!(data),
    //             Level::INFO => gloo::console::log!(data),
    //             Level::WARN => gloo::console::warn!(data),
    //             Level::ERROR => gloo::console::error!(data),
    //         }

    //         Ok(())
    //     }
    // }

    // impl Drop for ConsoleWriter {
    //     fn drop(&mut self) {
    //         let _ = self.flush();
    //     }
    // }

    struct WASMTracingLayer {
        pub spans: std::sync::Arc<std::sync::Mutex<indexmap::IndexMap<tracing::span::Id, String>>>,
        pub config: WASMTracingConfig,
    }

    struct WASMTracingConfig {
        pub target: bool,
        pub line: bool,
    }

    // pub fn original_log_init() {
    //     tracing_subscriber::fmt()
    //         .with_writer(
    //             // To avoide trace events in the browser from showing their
    //             // JS backtrace, which is very annoying, in my opinion
    //             MakeConsoleWriter, //.map_trace_level_to(tracing::Level::TRACE),
    //         )
    //         .with_ansi(false)
    //         .with_max_level(tracing::Level::TRACE)
    //         .with_env_filter(
    //             <tracing_subscriber::EnvFilter as std::str::FromStr>::from_str(
    //                 "heyadora_art=trace,artcord_leptos_web_sockets=trace",
    //             )
    //             .unwrap(),
    //         )
    //         .with_file(true)
    //         .with_line_number(true)
    //         .without_time()
    //         .with_thread_ids(true)
    //         .with_thread_names(true)
    //         // For some reason, if we don't do this in the browser, we get
    //         // a runtime error.
    //         .init();
    // }

    pub fn simple_logger_init() {
        tracing::subscriber::set_global_default(
            tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt::with(
                tracing_subscriber::Registry::default(),
                WASMTracingLayer::new(WASMTracingConfig {
                    line: true,
                    target: true,
                }),
            ),
        )
        .unwrap();
    }

    impl WASMTracingLayer {
        pub fn new(config: WASMTracingConfig) -> Self {
            Self {
                spans: std::sync::Arc::new(std::sync::Mutex::new(indexmap::IndexMap::new())),
                config,
            }
        }
    }

    impl<S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>>
        tracing_subscriber::Layer<S> for WASMTracingLayer
    {
        fn on_event(
            &self,
            event: &tracing::Event<'_>,
            ctx: tracing_subscriber::layer::Context<'_, S>,
        ) {
            let s = ctx.current_span();
            if let Some(metadata) = s.metadata() {
                let a = metadata.name();
                log1(format!("im in event with span: {}", a));
            } else {
                log1(format!("im in event"));
            }

            let mut spans_combined = String::new();
            {
                let mut current_span = ctx.current_span().id().and_then(|id| ctx.span(id));

                while let Some(span) = current_span {
                    let name = span.metadata().name();
                    spans_combined.push_str(name);
                    spans_combined.push_str(", ");
                    current_span = span.parent();
                }
                // loop {
                //     let Some(pan_id) = current_span else {
                //         break;
                //     };
                //     let Some(span) = ctx.span(pan_id) else {
                //         break;
                //     };
                //     let Some(parent_id) = span.parent() else {
                //         break;
                //     };

                //     current_span = 
                // }
            }

            // let a = ctx.current_span().id().unwrap();
            // let b = ctx.span(a).unwrap().metadata();

            // let mut spans_combined = String::new();
            // {
            //     let spans = &mut *self.spans.lock().unwrap();
            //     let len = spans.len();
            //     if len > 0 {
            //         spans_combined.push_str(" ");
            //     }
            //     for (i, span) in spans.values().enumerate() {
            //         spans_combined.push_str(span);
            //         if i + 1 != len {
            //             spans_combined.push_str(", ")
            //         }
            //     }
            //     //output.push_str(": ");
            // }
            let mut value = String::new();
            let writer = tracing_subscriber::fmt::format::Writer::new(&mut value);
            let mut visitor = tracing_subscriber::fmt::format::PrettyVisitor::new(writer, true);
            event.record(&mut visitor);

            let meta = event.metadata();
            let level = meta.level();
            let target = if self.config.target {
                format!(" {}", meta.target())
            } else {
                "".to_string()
            };
            let origin = if self.config.line {
                meta.file()
                    .and_then(|file| meta.line().map(|ln| format!(" {}:{}", file, ln)))
                    .unwrap_or_default()
            } else {
                String::new()
            };
            //let thread_suffix = thread_display_suffix();

            log5(
                format!("%c{level}%c{spans_combined}%c{target}{origin}%c: {value}"),
                match *level {
                    tracing::Level::TRACE => "color: dodgerblue; background: #444",
                    tracing::Level::DEBUG => "color: lawngreen; background: #444",
                    tracing::Level::INFO => "color: whitesmoke; background: #444",
                    tracing::Level::WARN => "color: orange; background: #444",
                    tracing::Level::ERROR => "color: red; background: #444",
                },
                "color: inherit; font-weight: bold",
                "color: gray; font-style: italic",
                "color: inherit",
            );
        }

        fn on_new_span(
            &self,
            attrs: &tracing::span::Attributes<'_>,
            id: &tracing::span::Id,
            ctx: tracing_subscriber::layer::Context<'_, S>,
        ) {
         
            // if let Some(span) = ctx.span(id) {
            //     let data = span.metadata();
            //     let name = data.name();
            //     log1(format!("new ctx get span: {}", name));
            // }
            // attrs.
            let meta = attrs.metadata();
            let name = meta.name();
            log1(format!("new span?: {}", name));
            let target = meta.target();
            // if !target.contains("artcord") {
            //     return;
            // }

            let mut body = String::new();
            let writer = tracing_subscriber::fmt::format::Writer::new(&mut body);
            let mut visitor = tracing_subscriber::fmt::format::PrettyVisitor::new(writer, true);
            attrs.record(&mut visitor);

            let has_name = !name.is_empty() && name != "{}";
            let has_body = !body.is_empty() && body != "{}";

            let output = match (has_name, has_body) {
                (true, false) => name.to_string(),
                (true, true) => {
                    format!("{} = {}", name, body)
                }
                (false, false) => String::from("{}"),
                (false, true) => body.to_string(),
            };

            let spans = &mut *self.spans.lock().unwrap();
            spans.insert(id.clone(), output);
        }

        fn on_exit(&self, id: &tracing::span::Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
            // let a = _ctx.span(id).unwrap().extensions_mut();
            // a.insert(val);
            let a = ctx.metadata(id).unwrap().name();
            log1(format!("exit span?: {}", a));
            let spans = &mut *self.spans.lock().unwrap();
            spans.swap_remove(id);
        }

        fn on_close(&self, id: tracing::span::Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
            let a = ctx.metadata(&id).unwrap().name();
            log1(format!("close span?: {}", a));

            let spans = &mut *self.spans.lock().unwrap();
            spans.swap_remove(&id);
        }
    }

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = performance)]
        pub fn mark(a: &str);
        #[wasm_bindgen(catch, js_namespace = performance)]
        pub fn measure(name: String, startMark: String) -> Result<(), wasm_bindgen::JsValue>;
        #[wasm_bindgen(js_namespace = console, js_name = log)]
        pub fn log1(message: String);
        #[wasm_bindgen(js_namespace = console, js_name = log)]
        pub fn log2(message1: &str, message2: &str);
        #[wasm_bindgen(js_namespace = console, js_name = log)]
        pub fn log3(message1: &str, message2: &str, message3: &str);
        #[wasm_bindgen(js_namespace = console, js_name = log)]
        pub fn log4(message1: String, message2: &str, message3: &str, message4: &str);
        #[wasm_bindgen(js_namespace = console, js_name = log)]
        pub fn log5(
            message1: String,
            message2: &str,
            message3: &str,
            message4: &str,
            message5: &str,
        );
    }
}
