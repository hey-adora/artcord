use std::{rc::Rc, sync::Arc};

use hook::{resize_imgs, GalleryImg};
use leptos_toolbox::prelude::*;
use ordered_float::OrderedFloat;
use server_fn::codec::Rkyv;
// pub mod app;
// pub mod error_template;
// pub mod errors;
// #[cfg(feature = "ssr")]
// pub mod middleware;
use leptos::{
    html::{button, div, Div},
    prelude::*,
    tachys::html::node_ref::node_ref,
};
use leptos_router::components::*;
use leptos_router::hooks::use_params_map;
use leptos_router::path;
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
            <body class="bg-gray-950">
                <App/>
            </body>
        </html>
    }
}

pub mod leptos_toolbox {
    pub mod prelude {
        pub use super::dropzone::{self, AddDropZone, GetFileData, GetFiles};
        pub use super::event_listener::{self, AddEventListener};
        pub use super::resize_observer::{self};
        // pub use super::global;
    }

    // pub mod global {
    //     use std::{
    //         any::Any,
    //         cell::{Ref, RefCell},
    //         collections::HashMap,
    //         marker::PhantomData,
    //         ops::Deref,
    //         pin::Pin,
    //         rc::Rc,
    //         sync::LazyLock,
    //     };

    //     use leptos::prelude::Effect;
    //     use tracing::{trace, trace_span};
    //     use uuid::Uuid;
    //     use wasm_bindgen::JsCast;
    //     use wasm_bindgen::{convert::ReturnWasmAbi, prelude::Closure};
    //     use web_sys::{
    //         js_sys::Array, Element, HtmlElement, MutationObserver, ResizeObserver,
    //         ResizeObserverEntry, ResizeObserverSize,
    //     };

    //     pub struct AppState {
    //         pub resize_observer: Pin<Box<ResizeObserver>>,
    //         pub mutation_observer: Pin<Box<MutationObserver>>,
    //         // pub resize_observer_closure: Pin<Box<Closure<dyn FnMut(Array, ResizeObserver)>>>,
    //         pub resize_observer_clients:
    //             HashMap<Uuid, (HtmlElement, Box<dyn FnMut(ResizeObserverEntry)>)>,
    //         pub mutation_observer_clients: HashMap<Uuid, (HtmlElement, Box<dyn FnMut()>)>,
    //         // pub event_listener_closures: HashMap<Uuid, Box<dyn Any>>,
    //     }

    //     thread_local! {
    //         pub static STORE: RefCell<Option<AppState>> = RefCell::new(None);
    //     }

    //     pub fn init_toolbox() {
    //         Effect::new(move || {
    //             let resize_callback = |entries: Array, observer: ResizeObserver| {
    //                 let entries: Vec<ResizeObserverEntry> = entries
    //                     .to_vec()
    //                     .into_iter()
    //                     .map(|v| v.unchecked_into::<ResizeObserverEntry>())
    //                     .collect();

    //                 STORE.with(|v| {
    //                     let mut v = v.borrow_mut();
    //                     let v = v.as_mut().unwrap();
    //                     let clients = &mut v.resize_observer_clients;
    //                     for entry in entries {
    //                         let target_elm = entry.target();
    //                         for (client_elm, closure) in clients.values_mut() {
    //                             let client_elm: Element = client_elm.clone().into();
    //                             let id = client_elm.clone().to_locale_string();

    //                             if target_elm == client_elm {
    //                                 trace!("abi id: {:?}", id);
    //                                 // let rect = entry.content_rect();
    //                                 // rect.w
    //                                 // let size: Vec<ResizeObserverSize> = entry
    //                                 //     .content_box_size()
    //                                 //     .to_vec()
    //                                 //     .into_iter()
    //                                 //     .map(|v| v.unchecked_into::<ResizeObserverSize>())
    //                                 //     .collect();
    //                                 // size[0].

    //                                 closure(entry);
    //                                 break;
    //                             }
    //                         }
    //                     }
    //                 });
    //             };
    //             let mutation_callback = |entries: Array, observer: MutationObserver| {
    //                 trace!("wow wtf");
    //             };
    //             let mutation_observer_closure =
    //                 Closure::<dyn FnMut(Array, MutationObserver)>::new(mutation_callback)
    //                     .into_js_value();
    //             let resize_observer_closure =
    //                 Closure::<dyn FnMut(Array, ResizeObserver)>::new(resize_callback)
    //                     .into_js_value();
    //             let resize_observer =
    //                 ResizeObserver::new(resize_observer_closure.as_ref().unchecked_ref()).unwrap();

    //             let mutation_observer =
    //                 MutationObserver::new(mutation_observer_closure.as_ref().unchecked_ref())
    //                     .unwrap();

    //             let app_state = AppState {
    //                 resize_observer: Box::pin(resize_observer),
    //                 mutation_observer: Box::pin(mutation_observer),
    //                 // resize_observer_closure: Box::pin(resize_observer_closure),
    //                 resize_observer_clients: HashMap::new(),
    //                 mutation_observer_clients: HashMap::new(),
    //                 // event_listener_closures: HashMap::new(),
    //             };
    //             STORE.set(Some(app_state));
    //         });
    //     }

    //     pub fn store_id() -> Uuid {
    //         Uuid::new_v4()
    //     }
    // }

    // pub mod mutation_observer {
    //     pub struct MutationObserver {}
    // }

    pub mod resize_observer {
        use wasm_bindgen::prelude::*;
        use web_sys::{self, js_sys::Array, ResizeObserver, ResizeObserverEntry};

        pub fn new<F>(mut callback: F) -> ResizeObserver
        where
            F: FnMut(Vec<web_sys::ResizeObserverEntry>, web_sys::ResizeObserver) + Clone + 'static,
        {
            let resize_observer_closure = Closure::<dyn FnMut(Array, ResizeObserver)>::new(
                move |entries: Array, observer: ResizeObserver| {
                    let entries: Vec<ResizeObserverEntry> = entries
                        .to_vec()
                        .into_iter()
                        .map(|v| v.unchecked_into::<ResizeObserverEntry>())
                        .collect();
                    callback(entries, observer);
                },
            )
            .into_js_value();
            ResizeObserver::new(resize_observer_closure.as_ref().unchecked_ref()).unwrap()
        }
        //                 let entries: Vec<ResizeObserverEntry> = entries
        //                     .to_vec()
        //                     .into_iter()
        //                     .map(|v| v.unchecked_into::<ResizeObserverEntry>())
        //                     .collect();

        // let mutation_observer_closure =
        //     Closure::<dyn FnMut(Array, MutationObserver)>::new(mutation_callback)
        //         .into_js_value();
        // pub struct ResizeObserver<F>
        // where
        //     F: FnMut(Vec<web_sys::ResizeObserverEntry>, web_sys::ResizeObserver) + Clone + 'static,
        // {
        //     pub callback: F,
        // }

        // impl<F> ResizeObserver<F> where
        //     F: FnMut(Vec<web_sys::ResizeObserverEntry>, web_sys::ResizeObserver) + Clone + 'static
        // {
        //     pub fn
        // }

        // use leptos::{ev::EventDescriptor, html::ElementType, prelude::*};
        // use tracing::{trace, trace_span};
        // use wasm_bindgen::prelude::*;
        // use web_sys::{
        //     js_sys::{self, Array},
        //     HtmlElement, MutationObserverInit, ResizeObserver, ResizeObserverEntry,
        // };

        // use super::global::{self, STORE};

        // pub trait AddResizeObserver {
        //     type Elm;

        //     fn add_resize_observer<F>(&self, callback: F)
        //     where
        //         F: FnMut(ResizeObserverEntry, Self::Elm) + Clone + 'static;
        // }

        // impl<E> AddResizeObserver for NodeRef<E>
        // where
        //     E: ElementType,
        //     E::Output: JsCast + Clone + 'static + Into<HtmlElement>,
        // {
        //     type Elm = E::Output;
        //     fn add_resize_observer<F>(&self, callback: F)
        //     where
        //         F: FnMut(ResizeObserverEntry, Self::Elm) + Clone + 'static,
        //     {
        //         new(self.clone(), callback);
        //     }
        // }

        // pub fn new<E, F>(target: NodeRef<E>, f: F)
        // where
        //     E: ElementType,
        //     E::Output: JsCast + Clone + 'static + Into<HtmlElement>,
        //     F: FnMut(ResizeObserverEntry, E::Output) + Clone + 'static,
        // {
        //     let store_id = global::store_id();

        //     Effect::new(move || {
        //         let span = trace_span!("resize_observer", "{}", format!("{store_id}")).entered();

        //         let Some(node) = target.get() else {
        //             trace!("target not found");
        //             return;
        //         };
        //         if STORE.with(|v| {
        //             v.borrow()
        //                 .as_ref()
        //                 .unwrap()
        //                 .resize_observer_clients
        //                 .contains_key(&store_id)
        //         }) {
        //             trace!("updating");
        //         } else {
        //             trace!("creating");
        //         }
        //         STORE.with_borrow_mut(|v| {
        //             let v = v.as_mut().unwrap();
        //             let html_node: HtmlElement = node.clone().into();
        //             html_node
        //                 .set_attribute("leptos_toolbox_id", &store_id.to_string())
        //                 .unwrap();
        //             let options = MutationObserverInit::new();
        //             options.set_child_list(true);
        //             options.set_subtree(true);
        //             v.mutation_observer
        //                 .observe_with_options(&html_node, &options)
        //                 .unwrap();
        //             v.resize_observer.observe(&html_node);
        //             v.resize_observer_clients.insert(
        //                 store_id,
        //                 (
        //                     html_node,
        //                     Box::new({
        //                         let mut f = f.clone();
        //                         move |entry| {
        //                             f(entry, node.clone());
        //                         }
        //                     }),
        //                 ),
        //             );
        //         });
        //         span.exit();
        //     });

        //     on_cleanup(move || {
        //         let span = trace_span!("resize_observer", "{}", format!("{store_id}")).entered();
        //         let Some(node) = target.get() else {
        //             trace!("target not found");
        //             return;
        //         };
        //         STORE.with_borrow_mut(|v| {
        //             let Some(v) = v.as_mut() else {
        //                 trace!("app state is not set for cleanup");
        //                 return;
        //             };
        //             trace!("removing");
        //             let node: HtmlElement = node.into();
        //             v.resize_observer.unobserve(&node);
        //             v.resize_observer_clients.remove(&store_id);
        //         });
        //         span.exit();
        //     });
        // }
    }

    pub mod event_listener {
        use std::fmt::Debug;

        use leptos::{ev::EventDescriptor, html::ElementType, prelude::*};
        use tracing::{trace, trace_span};
        use wasm_bindgen::prelude::*;
        use web_sys::HtmlElement;

        pub trait AddEventListener {
            fn add_event_listener<T, F>(&self, event: T, callback: F)
            where
                T: EventDescriptor + Debug + 'static,
                F: FnMut(<T as EventDescriptor>::EventType) + Clone + 'static;
        }

        impl<E> AddEventListener for NodeRef<E>
        where
            E: ElementType,
            E::Output: JsCast + Clone + 'static + Into<HtmlElement>,
        {
            fn add_event_listener<T, F>(&self, event: T, callback: F)
            where
                T: EventDescriptor + Debug + 'static,
                F: FnMut(<T as EventDescriptor>::EventType) + Clone + 'static,
            {
                new(self.clone(), event, callback);
            }
        }

        pub fn new<E, T, F>(target: NodeRef<E>, event: T, f: F)
        where
            E: ElementType,
            E::Output: JsCast + Clone + 'static + Into<HtmlElement>,
            T: EventDescriptor + Debug + 'static,
            F: FnMut(<T as EventDescriptor>::EventType) + Clone + 'static,
        {
            Effect::new(move || {
                let span = trace_span!("event_listener").entered();
                let Some(node) = target.get() else {
                    trace!("target not found");
                    return;
                };

                let node: HtmlElement = node.into();

                let closure = Closure::<dyn FnMut(_)>::new(f.clone()).into_js_value();

                node.add_event_listener_with_callback(
                    &event.name(),
                    closure.as_ref().unchecked_ref(),
                )
                .unwrap();

                span.exit();
            });
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
    pub width: u32,
    pub height: u32,
    pub view_width: RwSignal<f32>,
    pub view_height: RwSignal<f32>,
    pub view_pos_x: RwSignal<f32>,
    pub view_pos_y: RwSignal<f32>,
}

impl GalleryImg for Img {
    fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn set_pos(&mut self, left: f32, top: f32, new_width: f32, new_height: f32) {
        self.view_width.set(new_width);
        self.view_height.set(new_height);
        self.view_pos_x.set(left);
        self.view_pos_y.set(top);
    }
}

impl Img {
    pub fn rand() -> Self {
        // let a = random() as u64;
        // let id = ;
        // trace!("id: {}", id);
        let id = random().to_bits();
        let width = (random().to_bits() % 1000) as u32;
        let height = (random().to_bits() % 1000) as u32;
        // let mut rng = rand::rng();
        // let id = rng.random::<u64>();
        // let width = rng.random_range(1_u64..1000);
        // let height = rng.random_range(1_u64..1000);

        Self {
            id,
            width,
            height,
            view_width: RwSignal::new(0.0),
            view_height: RwSignal::new(0.0),
            view_pos_x: RwSignal::new(0.0),
            view_pos_y: RwSignal::new(0.0),
        }
    }

    pub fn rand_vec(n: usize) -> Vec<Self> {
        let mut output = Vec::new();
        for _ in 0..n {
            output.push(Img::rand());
        }
        output
    }
}

// struct X<T>;
// // struct B<T>;

// impl<A, B> From<X<A>> for X<B>
// where
//     B: From<A>,
// {
//     fn from(value: X<A>) -> Self {
//         X
//     }
// }

#[component]
pub fn GalleryImg(
    img: Img,
    #[prop(optional)] index: usize,
    #[prop(optional)] node_ref: Option<NodeRef<Div>>,
) -> impl IntoView {
    let node_ref2 = NodeRef::<Div>::new();

    node_ref2.on_load(move |e| {
        trace!("did i load or what? o.O");
    });

    Effect::new(move || {
        if index != 0 {
            return;
        }
        trace!("omg, i think im the first one");
        let Some(node_ref) = node_ref2.get() else {
            return;
        };
        node_ref.scroll_into_view();
        // if let Some(node_ref) = node_ref {

        //     node_ref.track();
        //     trace!("tracking!");
        // }
    });

    let width = img.width;
    let height = img.height;
    let view_width = img.view_width;
    let view_height = img.view_height;
    let left = img.view_pos_x;
    let top = img.view_pos_y;
    let r = (random().to_bits() % 255) as u8;
    let g = (random().to_bits() % 255) as u8;
    let b = (random().to_bits() % 255) as u8;

    let fn_background = move || format!("rgb({}, {}, {})", r, g, b);
    let fn_left = move || format!("{}px", left.get());
    let fn_top = move || format!("{}px", top.get() + 100.0);
    let fn_width = move || format!("{}px", view_width.get());
    let fn_height = move || format!("{}px", view_height.get());

    view! {
        <div
            node_ref=node_ref2
            // node_ref=first_ref
            class="text-white grid place-items-center bg-blue-950 absolute border border-red-600 overflow-hidden"
            style:background-color=fn_background
            style:left=fn_left
            style:top=fn_top
            style:width=fn_width
            style:height=fn_height>{
                format!("{}x{}", width, height)
            }
        </div>
    }
}

pub fn App() -> impl IntoView {
    // init_toolbox();

    let main_ref = NodeRef::new();
    let gallery_ref = NodeRef::new();
    let first_ref = NodeRef::<Div>::new();
    // let first_ref2 = StoredValue::<Option<HtmlDivElement>>::new_local(None);
    let imgs = RwSignal::<Vec<Img>>::new(Vec::new());
    let gallery_wdith = RwSignal::<f64>::new(0.0);

    // Effect::new(move || {
    //     let Some(first_elm): Option<HtmlDivElement> = first_ref.get() else {
    //         trace!("first is not found (effect)");
    //         return;
    //     };
    //     let v = first_elm.text_content().unwrap();
    //     trace!("first one?: {}", v);
    //     first_elm.scroll_into_view();
    // });
    // let tab_2_ref = NodeRef::new();
    // let tab_3_ref = NodeRef::new();

    main_ref.add_dropzone(async move |e, d| {
        //trace!("{}", e);
        for file in d.files().iter() {
            let data = file.data().await.unwrap();
            trace!("file name: {}", file.name(),);
            upload_file(data).await.unwrap();
        }
    });

    Effect::new(move || {
        trace!("logs broke?");
        let Some(gallery_elm): Option<HtmlDivElement> = gallery_ref.get() else {
            return;
        };
        let observer = resize_observer::new(move |entries, observer| {
            let Some(gallery_entry) = entries.first() else {
                return;
            };
            let rect = gallery_entry.content_rect();
            let w = rect.width();
            gallery_wdith.set(w);
            imgs.update_untracked(move |imgs| {
                resize_imgs(200, w as u32, imgs);
            });
        });
        observer.observe(&gallery_elm);
        // gallery_elm.scroll_to();
    });

    Effect::new(move || {
        trace!("your code broke");
        imgs.set(Img::rand_vec(100));
    });

    // gallery_ref.add_resize_observer(move |e, t| {
    //     let rect = e.content_rect();
    //     let w = rect.width() as u32;
    //     // trace!("w: {}", w);
    //     imgs.update_untracked(move |imgs| {
    //         resize_imgs(200, w, imgs);
    //         // for img in imgs.iter_mut() {
    //         //     let id = random().to_bits();
    //         //     img.id = id;
    //         // }
    //     });
    // });

    // let a ;

    // tab_2_ref.add_resize_observer(move |e, t| {
    //     trace!("oh wtf from tab 2");
    // });

    // tab_3_ref.add_resize_observer(move |e, t| {
    //     trace!("oh wtf from tab 3");
    // });

    let tab = RwSignal::new(false);
    let switch_tab = move |e| {
        let Some(gallery_elm): Option<HtmlDivElement> = gallery_ref.get_untracked() else {
            trace!("refresh target not found");
            return;
        };
        trace!("refreshing...");
        let width = gallery_elm.client_width() as u32;
        let new_imgs = Img::rand_vec(100);
        imgs.set(new_imgs);
        imgs.update_untracked(move |imgs| {
            resize_imgs(200, width, imgs);
        });
        // tab.update(|v| *v = !*v);
    };

    let scroll_btn = move |e| {
        // let Some(first_elm): Option<HtmlDivElement> = first_ref.get_untracked() else {
        //     trace!("first is not found!");
        //     return;
        // };
        // let v = first_elm.text_content().unwrap();
        // trace!("first one?: {}", v);
        // first_elm.scroll_into_view();
    };

    let get_imgs = move || {
        let mut imgs = imgs
            .get()
            .into_iter()
            .enumerate()
            .collect::<Vec<(usize, Img)>>();
        // resize_imgs(200, 1500, &mut imgs);
        imgs
    };

    // let k1 = div().node_ref(first_ref);
    // let k2 = div().add_any_attr(node_ref(first_ref));

    // let img_iter = move |(i, img): (usize, Img)| {
    //     let width = img.width;
    //     let height = img.height;
    //     let view_width = img.view_width;
    //     let view_height = img.view_height;
    //     let left = img.view_pos_x;
    //     let top = img.view_pos_y;
    //     let r = (random().to_bits() % 255) as u8;
    //     let g = (random().to_bits() % 255) as u8;
    //     let b = (random().to_bits() % 255) as u8;

    //     let fn_background = move || format!("rgb({}, {}, {})", r, g, b);
    //     let fn_left = move || format!("{}px", left.get());
    //     let fn_top = move || format!("{}px", top.get() + 100.0);
    //     let fn_width = move || format!("{}px", view_width.get());
    //     let fn_height = move || format!("{}px", view_height.get());

    //     // let v = view! {
    //     // };
    //     if i == 0 {
    //      view! {
    //         <div
    //             // node_ref=first_ref
    //             class="text-white grid place-items-center bg-blue-950 absolute border border-red-600 overflow-hidden"
    //             style:background-color=fn_background
    //             style:left=fn_left
    //             style:top=fn_top
    //             style:width=fn_width
    //             style:height=fn_height>{
    //                 format!("{}x{}", width, height)
    //             }
    //         </div>
    //      }
    //     } else {
    //         view! {
    //             <div
    //                 class="text-white grid place-items-center bg-blue-950 absolute border border-red-600 overflow-hidden"
    //                 style:background-color=fn_background
    //                 style:left=fn_left
    //                 style:top=fn_top
    //                 style:width=fn_width
    //                 style:height=fn_height>{
    //                     format!("{}x{}", width, height)
    //                 }
    //             </div>
    //         }
    //     }
    // if i == 0 {
    //     view! {
    //         <>
    //             <div
    //                 // node_ref=first_ref
    //                 class="text-white grid place-items-center bg-blue-950 absolute border border-red-600 overflow-hidden"
    //                 style:background-color=move || format!("rgb({}, {}, {})", r, g, b)
    //                 style:left=move || format!("{}px", left.get())
    //                 style:top=move || format!("{}px", top.get() + 100.0)
    //                 style:width=move || format!("{}px", view_width.get())
    //                 style:height=move || format!("{}px", view_height.get())>{
    //                     // format!("x:{}y:{}\n{}x{}", left, top, width, height)
    //                     format!("{}x{}", width, height)
    //                 }</div>
    //         </>
    //     }
    // } else {
    //     view! {
    //         <>
    //             <div
    //                 class="text-white grid place-items-center bg-blue-950 absolute border border-red-600 overflow-hidden"
    //                 style:background-color=move || format!("rgb({}, {}, {})", r, g, b)
    //                 style:left=move || format!("{}px", left.get())
    //                 style:top=move || format!("{}px", top.get() + 100.0)
    //                 style:width=move || format!("{}px", view_width.get())
    //                 style:height=move || format!("{}px", view_height.get())>{
    //                     // format!("x:{}y:{}\n{}x{}", left, top, width, height)
    //                     format!("{}x{}", width, height)
    //                 }</div>
    //         </>
    //     }
    // }
    // };
    // }

    view! {
        <Router>
            <Routes fallback=|| "not found">
                <Route path=path!("") view=move || view!{
                    <main node_ref=main_ref class="grid grid-rows-[auto_auto_auto_1fr] h-screen" >
                        <nav class="text-gray-200 pb-1">
                            <a href="/" class="font-black text-xl">"ArtBounty"</a>
                            <a href="/two" >"two"</a>
                        </nav>
                        <button on:click=switch_tab class="font-black text-xl text-white">"refresh"</button>
                        <button on:click=scroll_btn.clone() class="font-black text-xl text-white">"scroll"</button>
                        // <div class="h-full">
                        // <div node_ref=tab_3_ref id="tab3" class="p-10 bg-purple-600" >"tab3"</div>
                        // <img draggable="true" src="/assets/sword_lady.webp" />
                        // <Show
                        //     when = move || { tab.get() }
                        //     fallback=|| view!( <div id="tab1" class="p-10 bg-green-600" >"tab1"</div> )
                        // >
                        //     {
                        //         view!{
                        //             <div node_ref=tab_2_ref id="tab2" class="p-10 bg-red-600" >"tab2"</div>
                        //         }
                        //     }
                        //     // <DragTest />
                        //     // <DragTest2 />
                        // </Show>
                        <div id="gallery" node_ref=gallery_ref class="relative overflow-y-scroll overflow-x-hidden">
                            <div
                                class="bg-red-600 h-[100px] left-0 top-0 absolute"
                                style:width=move || format!("{}px", gallery_wdith.get())
                                // style:width=move || {
                                //     let Some(gallery_ref) : Option<HtmlDivElement> = gallery_ref.get() else {
                                //         trace!("zero? why o.O");
                                //         return String::from("50px");
                                //     };
                                //     let width = gallery_ref.client_width();
                                //     trace!("seting width or something, to: {}", width);
                                //     format!("{}px", width)
                                // }
                                >
                            </div>
                            <For
                                each=get_imgs
                                key=|img| img.1.id
                                children=move |(i, img)| {
                                    view! {
                                        <GalleryImg index=i img  />
                                    }
                                }
                            />
                        </div>
                        // </div>
                    </main>
                }/>
                <Route path=path!("two") view=move || view!{
                    <nav class="text-gray-200 pb-1">
                        <a href="/" class="font-black text-xl">"ArtBounty"</a>
                        <a href="/two" >"two"</a>
                    </nav>
                }/>
            </Routes>
        </Router>
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
        // fn get_pos(&self) -> (f32, f32);
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
        // debug!("utils: resizing started: count: {}", imgs.len());
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

        // debug!("utils: resizing ended: count: {}", imgs.len());
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
