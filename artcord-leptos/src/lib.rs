pub mod app;
use app::App;
use wasm_bindgen::prelude::wasm_bindgen;

struct WASMTracingLayer {
    pub spans: std::sync::Arc<std::sync::Mutex<indexmap::IndexMap<tracing::span::Id, String>>>,
    pub config: WASMTracingConfig,
}

struct WASMTracingConfig {
    pub target: bool,
    pub line: bool,
}

#[wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();

    // tracing_subscriber::fmt()
    //     .with_writer(
    //         // To avoide trace events in the browser from showing their
    //         // JS backtrace, which is very annoying, in my opinion
    //         tracing_subscriber_wasm::MakeConsoleWriter::default(), //.map_trace_level_to(tracing::Level::TRACE),
    //     )
    //     .with_ansi(false)
    //     .with_max_level(tracing::Level::TRACE)
    //     .with_env_filter(
    //         <tracing_subscriber::EnvFilter as std::str::FromStr>::from_str(
    //             "artcord_leptos=trace,artcord_leptos_web_sockets=trace",
    //         )
    //         .unwrap(),
    //     )
    //     .with_file(true)
    //     .with_line_number(true)
    //     .without_time()
    //     .with_thread_ids(true)
    //     .with_thread_names(true)
    //     // For some reason, if we don't do this in the browser, we get
    //     // a runtime error.
    //     .init();

    tracing::subscriber::set_global_default(
        tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt::with(
            tracing_subscriber::Registry::default(),
            WASMTracingLayer::new(WASMTracingConfig {
                line: false,
                target: false,
            }),
        ),
    )
    .unwrap();

    leptos::mount_to_body(App)
}

impl WASMTracingLayer {
    pub fn new(config: WASMTracingConfig,) -> Self {
        Self {
            spans: std::sync::Arc::new(std::sync::Mutex::new(indexmap::IndexMap::new())),
            config,
        }
    }
}

impl<S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>>
    tracing_subscriber::Layer<S> for WASMTracingLayer
{
    fn on_event(&self, event: &tracing::Event<'_>, ctx: tracing_subscriber::layer::Context<'_, S>) {
        let mut spans_combined = String::new();
        {
            let spans = &mut *self.spans.lock().unwrap();
            let len = spans.len();
            if len > 0 {
                spans_combined.push_str(" ");
            }
            for (i, span) in spans.values().enumerate() {
                spans_combined.push_str(span);
                if i + 1 != len {
                    spans_combined.push_str(", ")
                }
            }
            //output.push_str(": ");
        }
        let mut value = String::new();
        let writer = tracing_subscriber::fmt::format::Writer::new(&mut value);
        let mut visitor = tracing_subscriber::fmt::format::PrettyVisitor::new(writer, true);
        event.record(&mut visitor);

        let meta = event.metadata();
        let level = meta.level();
        let target = if self.config.target { format!(" {}", meta.target()) } else { "".to_string() };
        let origin = if self.config.line { meta
            .file()
            .and_then(|file| meta.line().map(|ln| format!(" {}:{}", file, ln)))
            .unwrap_or_default() } else { String::new() };
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
        let meta = attrs.metadata();
        let name = meta.name();
        let target = meta.target();
        if !target.contains("artcord") {
            return;
        }

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

    fn on_exit(&self, id: &tracing::span::Id, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let spans = &mut *self.spans.lock().unwrap();
        spans.swap_remove(id);
    }

    fn on_close(&self, id: tracing::span::Id, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let spans = &mut *self.spans.lock().unwrap();
        spans.swap_remove(&id);
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = performance)]
    fn mark(a: &str);
    #[wasm_bindgen(catch, js_namespace = performance)]
    fn measure(name: String, startMark: String) -> Result<(), wasm_bindgen::JsValue>;
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log1(message: String);
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log2(message1: &str, message2: &str);
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log3(message1: &str, message2: &str, message3: &str);
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log4(message1: String, message2: &str, message3: &str, message4: &str);
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log5(message1: String, message2: &str, message3: &str, message4: &str, message5: &str);
}

#[cfg(not(feature = "mark-with-rayon-thread-index"))]
#[inline]
fn thread_display_suffix() -> &'static str {
    ""
}
#[cfg(feature = "mark-with-rayon-thread-index")]
fn thread_display_suffix() -> String {
    let mut message = " #".to_string();
    match rayon::current_thread_index() {
        Some(idx) => message.push_str(&format!("{}", idx)),
        None => message.push_str("main"),
    }
    message
}
