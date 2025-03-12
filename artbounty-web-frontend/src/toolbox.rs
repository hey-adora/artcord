pub mod prelude {
    pub use super::dropzone::{self, AddDropZone, GetFileData, GetFiles};
    pub use super::event_listener::{self, AddEventListener};
    pub use super::intersection_observer::{self};
    pub use super::random::{random_u8, random_u32, random_u32_ranged, random_u64};
    pub use super::resize_observer::{self, AddResizeObserver, GetContentBoxSize};
}

// pub mod tree {
//     use indextree::{Arena, NodeId};
//     use leptos::prelude::*;

//     #[derive(Debug, Clone, Default, PartialEq, Eq)]
//     pub struct TreeState {
//         id: usize,
//         tree: Arena<usize>,
//         current: Option<NodeId>,
//     }

//     pub fn ping() {
//         let tree_state = use_context::<StoredValue<TreeState>>().unwrap_or_else(move || {
//             provide_context(StoredValue::new(TreeState::default()));
//             expect_context::<StoredValue<TreeState>>()
//         });
//         tree_state.update_value(|tree_state| {
//             let id = tree_state.id;
//             tree_state.id += 1;
//             let Some(node_id) = tree_state.current else {
//                 return;
//             };
//             no
//         });
//         let id = ctx
//             .id
//             .try_update_value(|v| {
//                 let id = *v;
//                 *v += 1;
//                 id
//             })
//             .unwrap();
//         ctx.current.update_value(|v| {
//             ctx.tree.update_value(|tree| {});
//             match v {
//                 Some(v) => {}
//                 None => {}
//             }
//         });
//     }
// }

pub mod random {
    use web_sys::js_sys::Math::random;

    pub fn random_u8() -> u8 {
        (random().to_bits() % 255) as u8
    }

    pub fn random_u64() -> u64 {
        random().to_bits()
    }

    pub fn random_u32() -> u32 {
        random_u64() as u32
    }

    pub fn random_u32_ranged(min: u32, max: u32) -> u32 {
        (random_u32() + min) % max
    }
}

pub mod intersection_observer {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::prelude::Closure;
    use web_sys::{IntersectionObserver, IntersectionObserverEntry, js_sys::Array};

    pub fn new<F>(mut callback: F) -> IntersectionObserver
    where
        F: FnMut(Vec<IntersectionObserverEntry>, IntersectionObserver) + Clone + 'static,
    {
        let observer_closure = Closure::<dyn FnMut(Array, IntersectionObserver)>::new(
            move |entries: Array, observer: IntersectionObserver| {
                let entries: Vec<IntersectionObserverEntry> = entries
                    .to_vec()
                    .into_iter()
                    .map(|v| v.unchecked_into::<IntersectionObserverEntry>())
                    .collect();
                callback(entries, observer);
            },
        )
        .into_js_value();
        IntersectionObserver::new(observer_closure.as_ref().unchecked_ref()).unwrap()
    }
}

pub mod resize_observer {
    use std::{collections::HashMap, ops::DerefMut, str::FromStr};

    use leptos::{
        html::ElementType,
        prelude::{
            Effect, Get, GetUntracked, LocalStorage, NodeRef, RwSignal, Set, Storage, StoredValue,
            UpdateValue, With, expect_context, on_cleanup, provide_context,
        },
    };
    use send_wrapper::SendWrapper;
    use tracing::{error, trace, trace_span};
    use uuid::Uuid;
    use wasm_bindgen::prelude::*;
    use web_sys::{
        self, Element, HtmlElement, ResizeObserver, ResizeObserverEntry, ResizeObserverSize,
        js_sys::Array,
    };

    const ATTRIBUTE_FIELD_NAME: &str = "leptos_toolbox_resize_observer_id";

    pub trait AddResizeObserver {
        fn add_resize_observer<F>(&self, callback: F)
        where
            F: FnMut(ResizeObserverEntry, ResizeObserver) + Send + Sync + Clone + 'static;
    }

    pub trait GetContentBoxSize {
        fn get_content_box_size(&self) -> Vec<ResizeObserverSize>;
    }

    impl GetContentBoxSize for ResizeObserverEntry {
        fn get_content_box_size(&self) -> Vec<ResizeObserverSize> {
            self.content_box_size()
                .to_vec()
                .into_iter()
                .map(|v| v.unchecked_into::<ResizeObserverSize>())
                .collect()
        }
    }

    // let size: Vec<ResizeObserverSize> = entry
    //     .content_box_size()
    //     .to_vec()
    //     .into_iter()
    //     .map(|v| v.unchecked_into::<ResizeObserverSize>())
    //     .collect();

    impl<E> AddResizeObserver for NodeRef<E>
    where
        E: ElementType,
        E::Output: JsCast + Clone + 'static + Into<HtmlElement>,
    {
        fn add_resize_observer<F>(&self, callback: F)
        where
            F: FnMut(ResizeObserverEntry, ResizeObserver) + Send + Sync + Clone + 'static,
        {
            new(self.clone(), callback);
        }
    }

    #[derive(Default, Clone)]
    pub struct GlobalState {
        pub observer: RwSignal<Option<SendWrapper<ResizeObserver>>>,
        pub callbacks: StoredValue<
            HashMap<
                Uuid,
                Box<dyn FnMut(ResizeObserverEntry, ResizeObserver) + Send + Sync + 'static>,
            >,
        >,
    }

    pub fn init_global_state() {
        provide_context(GlobalState::default());

        Effect::new(move || {
            let ctx = expect_context::<GlobalState>();

            let observer = new_raw(move |entries, observer| {
                ctx.callbacks.update_value(|callbacks| {
                    for entry in entries {
                        let target = entry.target();
                        let Some(id) = get_observer_id(&target) else {
                            continue;
                        };

                        let Some(callback) = callbacks.get_mut(&id) else {
                            continue;
                        };
                        callback(entry, observer.clone());
                    }
                });
            });

            ctx.observer.set(Some(SendWrapper::new(observer)));
        });
    }

    fn get_observer_id(target: &Element) -> Option<Uuid> {
        let Some(id) = target.get_attribute(ATTRIBUTE_FIELD_NAME) else {
            error!(
                "{} was not set {:?}",
                ATTRIBUTE_FIELD_NAME,
                target.to_string().as_string()
            );
            return None;
        };
        let id = match Uuid::from_str(&id) {
            Ok(id) => id,
            Err(err) => {
                error!(
                    "{} is invalid {:?}",
                    ATTRIBUTE_FIELD_NAME,
                    target.to_string().as_string()
                );
                return None;
            }
        };

        Some(id)
    }

    fn set_observer_id(target: &Element, id: Uuid) {
        target
            .set_attribute(ATTRIBUTE_FIELD_NAME, &id.to_string())
            .unwrap();
    }

    pub fn new<E, F>(target: NodeRef<E>, mut callback: F)
    where
        E: ElementType,
        E::Output: JsCast + Clone + 'static + Into<HtmlElement>,
        F: FnMut(ResizeObserverEntry, web_sys::ResizeObserver) + Clone + Send + Sync + 'static,
    {
        let ctx = expect_context::<GlobalState>();
        let id = Uuid::new_v4();

        Effect::new(move || {
            let span = trace_span!("resize observer").entered();

            let (Some(target), Some(observer)) = (target.get(), ctx.observer.get()) else {
                return;
            };

            let target: HtmlElement = target.into();

            set_observer_id(&target, id);

            ctx.callbacks.update_value(|v| {
                v.insert(id, Box::new(callback.clone()));
                trace!("created {}", &id);
            });

            observer.observe(&target);

            span.exit();
        });

        on_cleanup(move || {
            let span = trace_span!("resize observer").entered();

            let (Some(target), Some(observer)) =
                (target.get_untracked(), ctx.observer.get_untracked())
            else {
                return;
            };

            let target: HtmlElement = target.into();

            let Some(id) = get_observer_id(&target) else {
                return;
            };

            observer.unobserve(&target);

            ctx.callbacks.update_value(|callbacks| {
                callbacks.remove(&id);
                trace!("removed {}", &id);
            });

            span.exit();
        });
    }

    pub fn new_raw<F>(mut callback: F) -> ResizeObserver
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

            node.add_event_listener_with_callback(&event.name(), closure.as_ref().unchecked_ref())
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
        DragEvent, HtmlElement, ReadableStreamDefaultReader,
        js_sys::{self, Object, Promise, Reflect, Uint8Array},
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
