pub mod prelude {
    pub use super::dropzone::{self, AddDropZone, GetFileData, GetFiles};
    pub use super::event_listener::{self, AddEventListener};
    pub use super::resize_observer::{self};
}

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
