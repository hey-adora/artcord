use std::{rc::Rc, sync::Arc};

use leptos_toolbox::use_event_listener;
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

    use leptos::{html::ElementType, prelude::*};
    use reactive_stores::Store;
    use tracing::trace;
    use uuid::Uuid;
    use wasm_bindgen::{convert::FromWasmAbi, prelude::*};
    use web_sys::{
        js_sys::{Function, Object},
        AddEventListenerOptions, HtmlElement,
    };

    thread_local! {
        static WEB_SYS_STORE: RefCell<HashMap<uuid::Uuid, Rc<Box<dyn Any>>>> = RefCell::new(HashMap::default());
    }

    fn store_fn_set<T: FromWasmAbi + 'static, F: FnMut(T) + 'static>(id: Uuid, f: F){
        // let closure = Rc::new(Closure::<dyn FnMut(_)>::new(
        //     move |event: web_sys::DragEvent| {
        //         trace!("nova");
        //     },
        // ));
        
        let closure = Rc::new(Box::new(Closure::<dyn FnMut(_)>::new(f)) as Box<dyn Any>);
        WEB_SYS_STORE.with(|v| v.borrow_mut().insert(id, closure));
    }

    fn store_fn_with<T: FromWasmAbi + 'static, F: FnMut(&Function)>(id: &Uuid, mut f: F) {
        WEB_SYS_STORE.with(|v| {
            let store = v.borrow();
            let rc = store.get(&id).unwrap();
            let closure = rc.downcast_ref::<Closure<dyn FnMut(T)>>().unwrap();
            let closure = closure.as_ref().as_ref().unchecked_ref::<Function>();
            f(closure);
        });
    }

    fn store_rm(id: &Uuid) {
        WEB_SYS_STORE.with(|v| {
            let store = v.borrow();
            let rc = store.get(id).unwrap();
            let count = Rc::weak_count(rc);
            trace!("COUNT {count}");
        });
    }

    pub fn use_event_listener<E, T, F>(event: &'static str, f: F) -> NodeRef<E>
    where
        E: ElementType,
        E::Output: JsCast + Clone + 'static + Into<HtmlElement>,
        T: FromWasmAbi + 'static,
        F: FnMut(T),
    {
        let node = NodeRef::<E>::new();
        let id = Uuid::new_v4();
        

        Effect::new( move || {
            trace!("EFFECT: {id}");
            let Some(node) = node.get() else {
                return;
            };
            // let Some(node) = node.get() else {
            //     return;
            // };
            let node: HtmlElement = node.into();
            store_fn_set(id, |event: T| {
                trace!("nova");
            });
            store_fn_with::<T, _>(&id,  |closure| {
                node.add_event_listener_with_callback(event, closure).unwrap();
            });

           
    
            
            //node.into.add("dragover", &closure).unwrap();

            //let a = Rc::new(closure.into_js_value());
            //let g = closure.as_ref().unchecked_ref::<Function>();
            //
            // let c = a.as_ref();

            //closure.forget();
        });

        on_cleanup(move || {
            trace!("CLEANUP");
            //store_rm(&id);
        });
        

        node
    }

    // fn get_context() {
    //     // let web_sys_store = use_context::<Local<WebSysStore>>().unwrap_or_else(|| {
    //     //     provide_context(WebSysStore::default());
    //     //     expect_context::<WebSysStore>()
    //     // });
    // }
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

    //let drop_zone_el = NodeRef::<Main>::new();
    let drag_ref = use_event_listener("dragover", |e: web_sys::DragEvent| {
        trace!("wowza");
    });

    // let on_drop = |event| {
    //     // called when files are dropped on zone

    //     trace!("droppp");
    // };

    let y = |e: DragEvent| {
        trace!("droppp");
    };

    let y2 = || {
        trace!("drop hover over");
    };

    // let UseDropZoneReturn {
    //     is_over_drop_zone, ..
    // } = use_drop_zone_with_options(drop_zone_el, UseDropZoneOptions::default().on_drop(on_drop));

    // /let r = NodeRef::new();

    // Effect::new(move || {
    //     let Some(node): Option<HtmlDivElement> = r.get() else {
    //         return;
    //     };

    //     let closure =Closure::<dyn FnMut(_)>::new(
    //         move |event: web_sys::DragEvent| {
    //             trace!("nova");
    //         },
    //     );
    //     //let a = Rc::new(closure.into_js_value());
    //     let g = closure.as_ref();
    //     let h= g.unchecked_ref::<Function>();
    //     //
    //     // let c = a.as_ref();

    //     node.add_event_listener_with_callback("dragover", &closure)
    //         .unwrap();
    //     //closure.forget();
    // });

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

                    <div node_ref=drag_ref  class="p-10 bg-red-600">"tab2"</div>
                </Show>
            </div>
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
    use wasm_bindgen::prelude::*;

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
                    line: true,
                    target: true,
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
        fn on_event(
            &self,
            event: &tracing::Event<'_>,
            ctx: tracing_subscriber::layer::Context<'_, S>,
        ) {
            let mut spans_combined = String::new();
            {
                let mut span_names: Vec<&str> = Vec::new();
                let mut current_span = ctx.current_span().id().and_then(|id| ctx.span(id));

                while let Some(span) = current_span {
                    let name = span.metadata().name();
                    span_names.push(&name);
                    current_span = span.parent();
                }

                if !span_names.is_empty() {
                    spans_combined.push_str(" ");
                    spans_combined += &span_names.join(" ");
                }
            }

            let mut value = String::new();
            {
                let writer = tracing_subscriber::fmt::format::Writer::new(&mut value);
                let mut visitor = tracing_subscriber::fmt::format::PrettyVisitor::new(writer, true);
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
