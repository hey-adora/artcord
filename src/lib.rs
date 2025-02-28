use std::{rc::Rc, sync::Arc};

use leptos_toolbox::{global::init_toolbox, prelude::*};
use server_fn::codec::Rkyv;
// pub mod app;
// pub mod error_template;
// pub mod errors;
// #[cfg(feature = "ssr")]
// pub mod middleware;
use tracing::{error, trace};

use leptos::prelude::*;
use wasm_bindgen::{prelude::Closure, JsCast, UnwrapThrowExt};
use web_sys::{
    js_sys::{self, Function, Math::random},
    Blob, HtmlDivElement,
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>

                <HydrationScripts options/>
                // <meta http-equiv="Content-Security-Policy" content="default-src *; style-src 'self' 'unsafe-inline'; script-src 'self' 'unsafe-inline' 'unsafe-eval' http://localhost:3000/"/>
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
    pub mod prelude {
        pub use super::dropzone::{self, AddDropZone, GetFileData, GetFiles};
        pub use super::event_listener;
        pub use super::global;
        pub use super::observer;
    }

    pub mod global {
        use std::{
            any::Any,
            cell::{Ref, RefCell},
            collections::HashMap,
            marker::PhantomData,
            ops::Deref,
            pin::Pin,
            rc::Rc,
            sync::LazyLock,
        };

        use leptos::prelude::Effect;
        use tracing::{trace, trace_span};
        use uuid::Uuid;
        use wasm_bindgen::prelude::Closure;
        use wasm_bindgen::JsCast;
        use web_sys::{js_sys::Array, ResizeObserver};

        pub struct AppState {
            pub resize_observer: ResizeObserver,
            pub resize_observer_closure: Closure<dyn FnMut(Array, ResizeObserver)>,
            pub resize_observer_clients: HashMap<Uuid, Box<dyn Fn()>>,
        }

        thread_local! {
            // pub static STORE: RefCell<HashMap<uuid::Uuid, Box<dyn Any>>> = RefCell::new(HashMap::default());
            pub static STORE: RefCell<Option<AppState>> = RefCell::new(None);
        }

        // pub struct AutoDowncastRef<T: Any + 'static> {
        //     data: Rc<Box<dyn Any>>,
        //     phantom: PhantomData<T>,
        // }

        // impl Deref for AutoDowncastRef<>
        // impl Default for AppState {
        //     fn default() -> Self {
        //         Self {
        //             resize_observer
        //         }
        //     }
        // }

        pub fn init_toolbox() {
            Effect::new(move || {
                let f = |entries: Array, observer: ResizeObserver| {
                    trace!("i think its working?");
                    STORE.with(|v| {
                        let mut v = v.borrow_mut();
                        let v = v.as_mut().unwrap();
                        let clients = &mut v.resize_observer_clients;
                        for client in clients.values_mut() {
                            client();
                        }
                    });
                };
                let resize_observer_closure = Closure::<dyn FnMut(Array, ResizeObserver)>::new(f);
                let resize_observer =
                    ResizeObserver::new(resize_observer_closure.as_ref().unchecked_ref()).unwrap();
                // let resize_observer = Box::pin(resize_observer);
                // let m = &mut *p.as_mut();
                let app_state = AppState {
                    resize_observer,
                    resize_observer_closure,
                    resize_observer_clients: HashMap::new(),
                };
                STORE.set(Some(app_state));
                // let mut v = Vec::new();
                // v.push(p);
                // let k = v[0];
                // STORE.m(f)

                // let a = &mut *STORE_CLOSURE_ID;
                // store_set(STORE_CLOSURE_ID, value);
            });
        }

        pub fn store_id() -> Uuid {
            Uuid::new_v4()
        }

        // pub fn store_set<T: Any + 'static>(id: Uuid, value: T) {
        //     let closure = Box::new(value) as Box<dyn Any>;
        //     STORE.with(|v| v.borrow_mut().insert(id, closure));
        // }

        // pub fn store_with<T: Any + 'static, R>(id: &Uuid, mut f: impl FnMut(Option<&T>) -> R) -> R {
        //     STORE.with(|v| {
        //         let store = v.borrow();
        //         let Some(rc) = store.get(&id) else {
        //             return f(None);
        //         };
        //         let value = rc.downcast_ref::<T>().unwrap();
        //         f(Some(value))
        //         // Some(store)
        //     })
        // }

        // // pub fn store_get(id: &Uuid) -> Ref<'_, HashMap<Uuid, Box<dyn Any>>> {
        // //     STORE.with(|v| v.borrow())
        // // }

        // pub fn store_rm(id: &Uuid) {
        //     let span = trace_span!("store_rm", "{}", format!("{}", id)).entered();
        //     STORE.with(|v| {
        //         let mut store = v.borrow_mut();
        //         let r = store.remove(id);
        //         if r.is_some() {
        //             trace!("item removed");
        //         } else {
        //             trace!("item not found");
        //         }
        //     });
        //     span.exit();
        // }
    }

    pub mod observer {
        use std::mem;

        use leptos::{ev::EventDescriptor, html::ElementType, prelude::*};
        use tracing::{trace, trace_span};
        use wasm_bindgen::prelude::*;
        use web_sys::{
            js_sys::{self, Array},
            HtmlElement, ResizeObserver,
        };

        use super::global::{self, STORE};

        pub fn new<E, F>(target: NodeRef<E>, f: F)
        where
            E: ElementType,
            E::Output: JsCast + Clone + 'static + Into<HtmlElement>,
            F: FnMut(Array, ResizeObserver) + Clone + 'static,
        {
            let store_id = global::store_id();
            // let rw_resize_observer = RwSignal::new(None::<ResizeObserver>);
            // let mut f = Some(f);
            // let f = std::sync::Arc::new(f);
            // let b = mem::take(&mut f);
            // let c = b.unwrap().clone();
            Effect::new(move || {
                let span = trace_span!("resize observer creation").entered();

                let Some(node) = target.get() else {
                    trace!("target not found");
                    return;
                };

                STORE.with_borrow_mut(|v| {
                    let v = v.as_mut().unwrap();
                    let node: HtmlElement = node.into();
                    v.resize_observer.observe(&node);
                    v.resize_observer_clients.insert(
                        store_id,
                        Box::new(|| {
                            trace!("wtf1");
                        }),
                    );
                });
                // trace!("getting target");
                // let Some(node) = target.get() else {
                //     trace!("target not found");
                //     return;
                // };
                // // trace!("target found");
                // let node: HtmlElement = node.into();

                // // let seg = global::store_with(id, f)
                // // let Some(item) = global::store_get(id) else {
                // //     return;
                // // };
                // // let item = item.downcast();

                // trace!("creating closure");
                // let closure =
                //     Closure::<dyn FnMut(Array, ResizeObserver)>::new(f.clone()).into_js_value();

                // // let closure =
                // //     Closure::<dyn FnMut(Array, ResizeObserver)>::new(mem::take(&mut f).unwrap())
                // //         .into_js_value();

                // trace!("creating observer");
                // let resize_observer =
                //     ResizeObserver::new(closure.as_ref().unchecked_ref()).unwrap();

                // resize_observer.disconnect();

                // trace!("event added");
                // resize_observer.observe(&node);
                // // rw_resize_observer.set(Some(ResizeObserverEntry));
                // // global::store_set::<ResizeObserver>(store_id, resize_observer);
                // trace!("stored in global store");
                span.exit();
                // node.add_event_listener_with_callback(
                //     &event.name(),
                //     closure.as_ref().unchecked_ref(),
                // )
                // .unwrap();
            });
            on_cleanup(move || {
                let span = trace_span!("resize observer cleanup").entered();
                let Some(node) = target.get() else {
                    trace!("target not found");
                    return;
                };
                STORE.with_borrow_mut(|v| {
                    let v = v.as_mut().unwrap();
                    let node: HtmlElement = node.into();
                    v.resize_observer.unobserve(&node);
                    v.resize_observer_clients.remove(&store_id);
                    trace!("removed");
                });
                span.exit();
            });
            // on_cleanup(move || {
            //     let span = trace_span!("resize observer").entered();
            //     trace!("running cleanup");
            //     // let a = rw_resize_observer.read().un;
            //     //let store = global::store_get::<ResizeObserver>(&store_id);
            //     trace!("getting observer");
            //     let Some(v) = v else {
            //         trace!("observer not found");
            //         return;
            //     };
            //     trace!("getting target");
            //     let Some(node) = target.get_untracked() else {
            //         trace!("target not found");
            //         return;
            //     };
            //     let node: HtmlElement = node.into();
            //     trace!("unobserving");
            //     v.unobserve(&node);
            //     global::store_rm(&store_id);
            //     span.exit();
            //     // rw_resize_observer.with(|resize_observer| {
            //     //     resize_observer.unobserve(target);
            //     // });
            // });
        }
    }

    pub mod event_listener {
        use std::mem;

        use leptos::{ev::EventDescriptor, html::ElementType, prelude::*};
        use wasm_bindgen::prelude::*;
        use web_sys::HtmlElement;

        pub fn new<E, T, F>(target: NodeRef<E>, event: T, f: F)
        where
            E: ElementType,
            E::Output: JsCast + Clone + 'static + Into<HtmlElement>,
            T: EventDescriptor + 'static,
            F: FnMut(<T as EventDescriptor>::EventType) + 'static,
        {
            let mut f = Some(f);
            let a = move || {
                let Some(node) = target.get() else {
                    return;
                };
                let node: HtmlElement = node.into();
                let closure =
                    Closure::<dyn FnMut(_)>::new(mem::take(&mut f).unwrap()).into_js_value();
                node.add_event_listener_with_callback(
                    &event.name(),
                    closure.as_ref().unchecked_ref(),
                )
                .unwrap();
            };
            Effect::new(a);
        }
    }

    pub mod dropzone {

        use std::{
            fmt::Display,
            future::Future,
            ops::{Deref, DerefMut},
            sync::{Arc, Mutex},
            time::SystemTime,
        };

        use gloo::file::{File, FileList, FileReadError};
        use leptos::{ev, html::ElementType, prelude::*, task::spawn_local};
        use tracing::{trace, trace_span};
        use wasm_bindgen::prelude::*;
        use web_sys::{
            js_sys::{self, Object, Promise, Reflect, Uint8Array},
            DragEvent, HtmlElement, ReadableStreamDefaultReader,
        };

        use super::event_listener;

        pub enum Event {
            Start,
            Enter,
            Over,
            Drop,
            Leave,
        }

        impl Display for Event {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let name = match self {
                    Event::Start => "start",
                    Event::Enter => "enter",
                    Event::Over => "over",
                    Event::Drop => "drop",
                    Event::Leave => "leave",
                };
                write!(f, "{}", name)
            }
        }

        pub trait AddDropZone {
            fn add_dropzone<F, R>(&self, callback: F)
            where
                R: Future<Output = ()> + 'static,
                F: FnMut(Event, DragEvent) -> R + 'static;
        }

        pub trait GetFiles {
            fn files(&self) -> gloo::file::FileList;
        }

        pub trait GetFileData {
            async fn data(&self) -> Result<Vec<u8>, FileReadError>;
        }

        impl<E> AddDropZone for NodeRef<E>
        where
            E: ElementType,
            E::Output: JsCast + Clone + 'static + Into<HtmlElement>,
        {
            fn add_dropzone<F, R>(&self, callback: F)
            where
                R: Future<Output = ()> + 'static,
                F: FnMut(Event, DragEvent) -> R + 'static,
            {
                new(self.clone(), callback);
            }
        }

        impl GetFileData for gloo::file::File {
            async fn data(&self) -> Result<Vec<u8>, FileReadError> {
                gloo::file::futures::read_as_bytes(self).await
            }
        }

        impl GetFiles for DragEvent {
            fn files(&self) -> gloo::file::FileList {
                let Some(files) = self.data_transfer().and_then(|v| v.files()) else {
                    trace!("shouldnt be here");
                    return gloo::file::FileList::from(web_sys::FileList::from(JsValue::null()));
                };
                trace!("len: {}", files.length());
                gloo::file::FileList::from(files)
            }
        }

        pub fn new<E, F, R>(target: NodeRef<E>, mut callback: F)
        where
            E: ElementType,
            E::Output: JsCast + Clone + 'static + Into<HtmlElement>,
            R: Future<Output = ()> + 'static,
            F: FnMut(Event, DragEvent) -> R + 'static,
        {
            let callback = Arc::new(Mutex::new(callback));

            event_listener::new(target, ev::dragstart, {
                let callback = callback.clone();

                move |e| {
                    let mut callback = callback.lock().unwrap();
                    let fut = callback(Event::Start, e);

                    spawn_local(fut);
                }
            });

            event_listener::new(target, ev::dragleave, {
                let callback = callback.clone();

                move |e| {
                    let mut callback = callback.lock().unwrap();
                    let fut = callback(Event::Leave, e);
                    spawn_local(fut);
                }
            });

            event_listener::new(target, ev::dragenter, {
                let callback = callback.clone();

                move |e| {
                    let mut callback = callback.lock().unwrap();
                    let fut = callback(Event::Enter, e);
                    spawn_local(fut);
                }
            });

            event_listener::new(target, ev::dragover, {
                let callback = callback.clone();

                move |e| {
                    e.prevent_default();

                    let mut callback = callback.lock().unwrap();
                    let fut = callback(Event::Over, e);
                    spawn_local(fut);
                }
            });

            event_listener::new(target, ev::drop, {
                let callback = callback.clone();

                move |e| {
                    e.prevent_default();
                    e.stop_propagation();

                    let mut callback = callback.lock().unwrap();
                    let fut = callback(Event::Drop, e);
                    spawn_local(fut);
                }
            });
        }
    }
}

#[component]
pub fn DragTest() -> impl IntoView {
    // let drag_ref = use_event_listener_dragover(|e| {
    //     trace!("wowza");
    // });

    view! {
        <div  class="p-10 bg-red-600">"tab2"</div>
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Img {
    pub id: u64,
    pub width: u64,
    pub height: u64,
}

impl Img {
    pub fn rand() -> Self {
        // let a = random() as u64;
        // let id = ;
        // trace!("id: {}", id);
        let id = random().to_bits();
        let width = random().to_bits() % 1000;
        let height = random().to_bits() % 1000;
        // let mut rng = rand::rng();
        // let id = rng.random::<u64>();
        // let width = rng.random_range(1_u64..1000);
        // let height = rng.random_range(1_u64..1000);

        Self { id, width, height }
    }

    pub fn rand_vec(n: usize) -> Vec<Self> {
        let mut output = Vec::new();
        for _ in 0..n {
            output.push(Img::rand());
        }
        output
    }
}

#[component]
pub fn App() -> impl IntoView {
    init_toolbox();

    let main_ref = NodeRef::new();
    let tab_2_ref = NodeRef::new();

    main_ref.add_dropzone(async move |e, d| {
        //trace!("{}", e);
        for file in d.files().iter() {
            let data = file.data().await.unwrap();
            trace!("file name: {}", file.name(),);
            upload_file(data).await.unwrap();
        }
    });

    observer::new(tab_2_ref, move |a, b| {
        trace!("oh wtf");
    });

    let tab = RwSignal::new(false);
    let switch_tab = move |e| {
        tab.update(|v| *v = !*v);
    };

    let imgs = RwSignal::new(Vec::new());
    Effect::new(move || {
        imgs.set(Img::rand_vec(200));
    });

    view! {
        <main node_ref=main_ref >
            <nav class="text-gray-200 pb-1">
                <h3 class="font-black text-xl">"ArtBounty"</h3>
            </nav>
            <div>
                // <img draggable="true" src="/assets/sword_lady.webp" />
                <button on:click=switch_tab class="font-black text-xl text-white">"switch tab"</button>
                <Show
                    when = move || { tab.get() }
                    fallback=|| view!( <div id="tab1" class="p-10 bg-green-600" >"tab1"</div> )
                >
                    {
                        view!{ <div node_ref=tab_2_ref id="tab1" class="p-10 bg-red-600" >"tab2"</div> }
                    }
                    // <DragTest />
                    // <DragTest2 />
                </Show>
                <For
                    each=move|| imgs.get()
                    key=|img| img.id
                    children=move |img: Img| {
                        let width = img.width;
                        let height = img.height;

                        view! {
                            <div
                                class="text-white grid place-items-center bg-blue-950"
                                style:width=move || format!("{}px", width)
                                style:height=move || format!("{}px", height)>{
                                    format!("{}x{}", img.width, img.height)
                                }</div>
                        }
                    }
                />

            </div>
        </main>
    }
}

#[server(
    input = Rkyv,
    output = Rkyv
)]
pub async fn upload_file(file: Vec<u8>) -> Result<(), ServerFnError> {
    let text = String::from_utf8_lossy(&file);

    println!("file uploaded: {}", text);
    Ok(())
    //Ok(std::fs::read_to_string("/home/hey/github/artcord/Dockerfile").unwrap())
}

#[server()]
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
