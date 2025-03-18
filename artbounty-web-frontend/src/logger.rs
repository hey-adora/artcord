use tracing::span;
use tracing_subscriber::field::RecordFields;
use tracing_subscriber::fmt::format::PrettyVisitor;
use tracing_subscriber::fmt::format::Writer;
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone)]
struct SpanBody(pub String);

pub struct WASMTracingLayer {
    pub config: WASMTracingConfig,
}

pub struct WASMTracingConfig {
    pub target: bool,
    pub line: bool,
}

pub fn simple_logger_init() {
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
}

impl WASMTracingLayer {
    pub fn new(config: WASMTracingConfig) -> Self {
        Self { config }
    }
}

impl<S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>>
    tracing_subscriber::Layer<S> for WASMTracingLayer
{
    fn on_event(&self, event: &tracing::Event<'_>, ctx: tracing_subscriber::layer::Context<'_, S>) {
        let mut spans_combined = String::new();
        {
            let mut span_text: Vec<String> = Vec::new();
            let mut current_span = ctx.current_span().id().and_then(|id| ctx.span(id));

            while let Some(span) = current_span {
                let name = span.metadata().name();
                let extensions = span.extensions();
                let span_body = extensions.get::<SpanBody>();

                if let Some(span_body) = span_body {
                    span_text.push(format!("{}({})", &name, span_body.0));
                } else {
                    span_text.push(name.to_string());
                }

                current_span = span.parent();
            }

            if !span_text.is_empty() {
                spans_combined = span_text.iter().rev().fold(String::from(" "), |mut a, b| {
                    a += b;
                    a += " ";
                    a
                });
            }
        }

        let mut value = String::new();
        {
            let writer = Writer::new(&mut value);
            let mut visitor = PrettyVisitor::new(writer, true);
            event.record(&mut visitor);
        }

        let meta = event.metadata();
        let level = *meta.level();
        let target = if self.config.target {
            format!(" {}", meta.target())
        } else {
            "".to_string()
        };
        let origin = if self.config.line
            || level == tracing::Level::ERROR
            || level == tracing::Level::WARN
        {
            meta.file()
                .and_then(|file| meta.line().map(|ln| format!(" {}:{}", file, ln)))
                .unwrap_or_default()
        } else {
            String::new()
        };

        log5(
            format!("%c{level}%c{spans_combined}%c{target}{origin}%c: {value}"),
            match level {
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
        attrs: &span::Attributes<'_>,
        id: &span::Id,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut span_body = String::new();
        let writer = Writer::new(&mut span_body);
        let mut visitor = PrettyVisitor::new(writer, true);
        attrs.record(&mut visitor);
        if !span_body.is_empty() {
            ctx.span(id)
                .unwrap()
                .extensions_mut()
                .insert(SpanBody(span_body));
        }
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    pub fn log5(message1: String, message2: &str, message3: &str, message4: &str, message5: &str);
}
