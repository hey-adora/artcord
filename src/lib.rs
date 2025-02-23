use std::{rc::Rc, sync::Arc};

use leptos_toolbox::{use_event_listener, use_event_listener_dragover};
use leptos_use::{use_drop_zone_with_options, UseDropZoneOptions, UseDropZoneReturn};
// pub mod app;
// pub mod error_template;
// pub mod errors;
// #[cfg(feature = "ssr")]
// pub mod middleware;
use tracing::{error, trace};

use leptos::{
    ev::{self, DragEvent},
    html::{Div, HtmlElement, Main},
    prelude::*,
    task::spawn_local,
};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{js_sys::Function, HtmlDivElement};

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
            <body class="bg-gray-950 py-1 px-1">
                <App/>
            </body>
        </html>
    }
}

pub mod leptos_toolbox {
    use std::{
        any::Any,
        cell::{LazyCell, RefCell, UnsafeCell},
        collections::HashMap,
        rc::Rc,
        sync::{Arc, LazyLock},
    };

    use leptos::{ev::{self, EventDescriptor}, html::ElementType, prelude::*};
    use reactive_stores::Store;
    use tracing::{trace, trace_span};
    use uuid::Uuid;
    use wasm_bindgen::{convert::FromWasmAbi, prelude::*};
    use web_sys::{
        js_sys::{Function, Object},
        AddEventListenerOptions, HtmlElement,
    };

    thread_local! {
        static WEB_SYS_STORE: RefCell<HashMap<uuid::Uuid, Rc<Box<dyn Any>>>> = RefCell::new(HashMap::default());
    }

    fn store_fn_set<T, F>(id: Uuid, f: F) where 
        T: FromWasmAbi + 'static,
        F: FnMut(T) + 'static,
     {
        let span_leptos_toolbox = trace_span!("LeptosToolbox", "{}", id).entered();
        trace!("inserting");
        let closure = Rc::new(Box::new(Closure::<dyn FnMut(_)>::new(f)) as Box<dyn Any>);
        WEB_SYS_STORE.with(|v| v.borrow_mut().insert(id, closure));
        span_leptos_toolbox.exit();
    }

    fn store_fn_with<T: FromWasmAbi + 'static, F: FnMut(&Function)>(id: &Uuid, mut f: F) {
        let span_leptos_toolbox = trace_span!("LeptosToolbox", "{}", id).entered();
        trace!("reading");
        WEB_SYS_STORE.with(|v| {
            let store = v.borrow();
            let rc = store.get(&id).unwrap();
            let closure = rc.downcast_ref::<Closure<dyn FnMut(T)>>().unwrap();
            let closure = closure.as_ref().as_ref().unchecked_ref::<Function>();
            f(closure);
        });
        span_leptos_toolbox.exit();
    }

    fn store_rm(id: &Uuid) {
        let span_leptos_toolbox = trace_span!("LeptosToolbox", "{}", id).entered();
        trace!("removing");
        WEB_SYS_STORE.with(|v| {
            let mut store = v.borrow_mut();
            let rc = store.remove(id).unwrap();
            let weak_count = Rc::weak_count(&rc);
            let strong_count = Rc::strong_count(&rc);
            assert!(weak_count == 0 && strong_count == 1);
        });

        span_leptos_toolbox.exit();
    }

    pub fn use_event_listener<E, T, F>(event: T, f: F) -> NodeRef<E>
    where
        E: ElementType,
        E::Output: JsCast + Clone + 'static + Into<HtmlElement>,
        T: EventDescriptor + 'static,
        F: FnMut(<T as EventDescriptor>::EventType),
    {
        let node = NodeRef::<E>::new();
        let id: Uuid = Uuid::new_v4();

        Effect::new(move || {
            let Some(node) = node.get() else {
                return;
            };
            let node: HtmlElement = node.into();
            store_fn_set(id, |event: <T as EventDescriptor>::EventType| {
                trace!("nova");
            });
            store_fn_with::<<T as EventDescriptor>::EventType, _>(&id, |closure| {
                node.add_event_listener_with_callback(&event.name(), closure)
                    .unwrap();
            });
        });

        Owner::on_cleanup(move || {
            store_rm(&id);
        });

        node
    }

    pub fn use_event_listener_dragover<E, F>(f: F) -> NodeRef<E>
    where
        E: ElementType,
        E::Output: JsCast + Clone + 'static + Into<HtmlElement>,
        F: FnMut(web_sys::DragEvent),
    {
        use_event_listener(ev::dragover, f)
    }
}

#[component]
pub fn DragTest() -> impl IntoView {
    let drag_ref = use_event_listener_dragover(|e| {
        trace!("wowza");
    });

    view! {
        <div node_ref=drag_ref  class="p-10 bg-red-600">"tab2"</div>
    }
}

// #[component]
// pub fn DragTest2() -> impl IntoView {
//     let target = NodeRef::new();
//     leptos_use::use_event_listener(target, ev::dragover, |e| {
//         trace!("hello");
//     });

//     view! {
//         <div node_ref=target  class="p-10 bg-red-600">"tab2"</div>
//     }
// }

#[component]
pub fn App() -> impl IntoView {
    let tab = RwSignal::new(false);
    let switch_tab = move |e| {
        tab.update(|v| *v = !*v);
    };

    view! {
        <main   >
            <nav class="text-gray-200 pb-1">
                <h3 class="font-black text-xl">"ArtBounty"</h3>
            </nav>
            <div>
                <img draggable="true" src="/assets/sword_lady.webp" />
                <button on:click=switch_tab class="font-black text-xl text-white">"switch tab"</button>
                <Show
                    when = move || { tab.get() }
                    fallback=|| view!( <div class="p-10 bg-green-600" >"tab1"</div> )
                >
                    <DragTest />
                    // <DragTest2 />
                </Show>

            </div>
        </main>
    }
}

#[server(endpoint = "/wtf")]
pub async fn get_server_data() -> Result<String, ServerFnError> {
    Ok(std::fs::read_to_string("/home/hey/github/artcord/Dockerfile").unwrap())
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    wasmlog::simple_logger_init();
    leptos::mount::hydrate_body(App);
}

pub mod hook {
    use std::fmt::Debug;

    use tracing::debug;

    pub const NEW_IMG_HEIGHT: u32 = 250;

    pub trait GalleryImg {
        fn get_size(&self) -> (u32, u32);
        fn get_pos(&self) -> (f32, f32);
        fn set_pos(&mut self, left: f32, top: f32, new_width: f32, new_height: f32);
    }

    pub fn resize_img<T: GalleryImg + Debug>(
        top: &mut f32,
        max_width: u32,
        new_row_start: usize,
        new_row_end: usize,
        imgs: &mut [T],
    ) {
        let mut total_ratio: f32 = 0f32;

        for i in new_row_start..(new_row_end + 1) {
            let (width, height) = imgs[i].get_size();
            total_ratio += width as f32 / height as f32;
        }
        let optimal_height: f32 = max_width as f32 / total_ratio;
        let mut left: f32 = 0.0;

        for i in new_row_start..(new_row_end + 1) {
            let (width, height) = imgs[i].get_size();
            let new_width = optimal_height * (width as f32 / height as f32);
            let new_height = optimal_height;
            imgs[i].set_pos(left, *top, new_width, new_height);
            left += new_width;
        }
        *top += optimal_height;
    }

    pub fn resize_img2<T: GalleryImg + Debug>(
        top: &mut f32,
        max_width: u32,
        new_row_start: usize,
        new_row_end: usize,
        imgs: &mut [T],
    ) {
        let mut optimal_count =
            (max_width as i32 / NEW_IMG_HEIGHT as i32) - (new_row_end - new_row_start) as i32;
        if optimal_count < 0 {
            optimal_count = 0;
        }
        let mut total_ratio: f32 = optimal_count as f32;
        if max_width < NEW_IMG_HEIGHT * 3 {
            total_ratio = 0.0;
        }

        for i in new_row_start..(new_row_end + 1) {
            let (width, height) = imgs[i].get_size();
            total_ratio += width as f32 / height as f32;
        }
        let optimal_height: f32 = max_width as f32 / total_ratio;
        let mut left: f32 = 0.0;

        for i in new_row_start..(new_row_end + 1) {
            let (width, height) = imgs[i].get_size();
            let new_width = optimal_height * (width as f32 / height as f32);
            let new_height = optimal_height;
            imgs[i].set_pos(left, *top, new_width, new_height);
            left += new_width;
        }

        *top += optimal_height;
    }

    pub fn resize_imgs<T: GalleryImg + Debug>(
        new_height: u32,
        max_width: u32,
        imgs: &mut [T],
    ) -> () {
        debug!("utils: resizing started: count: {}", imgs.len());
        let loop_start = 0;
        let loop_end = imgs.len();
        let mut new_row_start: usize = 0;
        let mut new_row_end: usize = if loop_end > 0 { loop_end - 1 } else { 0 };
        let mut current_row_filled_width: u32 = 0;
        let mut top: f32 = 0.0;

        for index in loop_start..loop_end {
            let org_img = &mut imgs[index];
            let (width, height) = org_img.get_size();
            let ratio: f32 = width as f32 / height as f32;
            let height_diff: u32 = if height < new_height {
                0
            } else {
                height - new_height
            };
            let new_width: u32 = width - (height_diff as f32 * ratio) as u32;
            if (current_row_filled_width + new_width) <= max_width {
                current_row_filled_width += new_width;
                new_row_end = index;
                if index == loop_end - 1 {
                    resize_img2(&mut top, max_width, new_row_start, new_row_end, imgs);
                }
            } else {
                if index != 0 {
                    resize_img(&mut top, max_width, new_row_start, new_row_end, imgs);
                }
                new_row_start = index;
                new_row_end = index;
                current_row_filled_width = new_width;
                if index == loop_end - 1 {
                    resize_img2(&mut top, max_width, new_row_start, new_row_end, imgs);
                }
            }
        }

        debug!("utils: resizing ended: count: {}", imgs.len());
    }

    pub fn calc_fit_count(width: u32, height: u32) -> u32 {
        (width * height) / (NEW_IMG_HEIGHT * NEW_IMG_HEIGHT)
    }
}

pub mod wasmlog {
    use std::ops::Deref;

    use leptos::logging::log;
    use tracing::{
        field::Visit,
        span::{self, Record},
    };
    use tracing_subscriber::field::RecordFields;
    use tracing_subscriber::fmt::format::PrettyVisitor;
    use tracing_subscriber::fmt::format::Writer;
    use wasm_bindgen::prelude::*;

    #[derive(Debug, Clone)]
    struct SpanBody(pub String);

    struct WASMTracingLayer {
        pub config: WASMTracingConfig,
    }

    struct WASMTracingConfig {
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

    // impl Deref for SpanBody {
    //     type Target = String;

    //     fn deref(&self) -> &Self::Target {
    //         &self.0
    //     }
    // }

    impl WASMTracingLayer {
        pub fn new(config: WASMTracingConfig) -> Self {
            Self { config }
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
            // let spans_combined = ctx
            //     .current_span()
            //     .id()
            //     .and_then(|id| ctx.span(id))
            //     .and_then(|span| span.extensions().get::<SpanBody>().cloned())
            //     .map(|data| data.0)
            //     .unwrap_or_default();

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

            // let spans_combined = ctx
            //     .current_span()
            //     .id()
            //     .and_then(|id| ctx.span(id))
            //     .map(|span| {
            //         span.scope().fold(String::from(" "), |mut a, b| {
            //             let name = span.metadata().name();
            //             let extensions = span.extensions();
            //             let span_body = extensions.get::<SpanBody>();

            //             if let Some(span_body) = span_body {
            //                 a.push_str(&name);
            //                 a.push_str("(");
            //                 a.push_str(&span_body);
            //                 a.push_str(")");
            //             } else {
            //                 a.push_str(&name);
            //             }

            //             a
            //         })
            //     })
            //     .unwrap_or_default();

            let mut value = String::new();
            {
                let writer = Writer::new(&mut value);
                let mut visitor = PrettyVisitor::new(writer, true);
                event.record(&mut visitor);
            }

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
        pub fn log5(
            message1: String,
            message2: &str,
            message3: &str,
            message4: &str,
            message5: &str,
        );
    }
}
