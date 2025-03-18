pub mod prelude {
    pub use super::dropzone::{self, AddDropZone};
    pub use super::event_listener::{self, AddEventListener};
    pub use super::file::{self, GetFileStream, GetFiles, GetStreamChunk, PushChunkToVec};
    pub use super::intersection_observer::{self, AddIntersectionObserver};
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

// pub mod closure {
//     use wasm_bindgen::{JsCast, JsValue, prelude::Closure};
//     use web_sys::{IntersectionObserver, IntersectionObserverEntry, js_sys::Array};

//     pub fn new() -> JsValue {
//         Closure::<dyn FnMut(Array, IntersectionObserver)>::new(
//             move |entries: Array, observer: IntersectionObserver| {
//                 let entries: Vec<IntersectionObserverEntry> = entries
//                     .to_vec()
//                     .into_iter()
//                     .map(|v| v.unchecked_into::<IntersectionObserverEntry>())
//                     .collect();
//                 callback(entries, observer);
//             },
//         )
//         .into_js_value()
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

pub mod uuid {
    use std::str::FromStr;

    use tracing::{error, trace, trace_span};
    use uuid::Uuid;
    use web_sys::Element;

    pub fn get_id(target: &Element, field_name: &str) -> Option<Uuid> {
        // trace!(
        //     "what does the fox say?: {:?}",
        //     target.to_string().as_string()
        // );
        let Some(id) = target.get_attribute(field_name) else {
            error!(
                "{} was not set {:?}",
                field_name,
                target.to_string().as_string()
            );
            return None;
        };
        let id = match Uuid::from_str(&id) {
            Ok(id) => id,
            Err(err) => {
                error!(
                    "{} is invalid {:?}",
                    field_name,
                    target.to_string().as_string()
                );
                return None;
            }
        };

        Some(id)
    }

    pub fn set_id(target: &Element, field_name: &str, id: Uuid) {
        target.set_attribute(field_name, &id.to_string()).unwrap();
    }
}

pub mod intersection_observer {
    use std::collections::HashMap;
    use std::hash::{DefaultHasher, Hash, Hasher};
    use std::ops::Deref;

    use leptos::html;
    use leptos::{html::ElementType, prelude::*};
    use ordered_float::OrderedFloat;
    use send_wrapper::SendWrapper;
    use sha2::Digest;
    use tracing::{error, trace, trace_span, warn};
    use uuid::Uuid;
    use wasm_bindgen::prelude::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{
        Element, HtmlElement, IntersectionObserver, IntersectionObserverEntry,
        IntersectionObserverInit, js_sys::Array,
    };

    use super::uuid::{get_id, set_id};

    const ID_FIELD_NAME: &str = "data-leptos_toolbox_intersection_observer_id";
    const ID_FIELD_ROOT_NAME: &str = "data-leptos_toolbox_intersection_observer_root_id";

    pub trait AddIntersectionObserver {
        fn observe_intersection_with_options<F, R>(&self, callback: F, options: Options<R>)
        where
            R: ElementType,
            R::Output: JsCast + Clone + 'static + Into<HtmlElement>,
            F: FnMut(IntersectionObserverEntry, IntersectionObserver)
                + Send
                + Sync
                + Clone
                + 'static;
    }

    impl<E> AddIntersectionObserver for NodeRef<E>
    where
        E: ElementType,
        E::Output: JsCast + Clone + 'static + Into<HtmlElement>,
    {
        fn observe_intersection_with_options<F, R>(&self, callback: F, options: Options<R>)
        where
            R: ElementType,
            R::Output: JsCast + Clone + 'static + Into<HtmlElement>,
            F: FnMut(IntersectionObserverEntry, IntersectionObserver)
                + Send
                + Sync
                + Clone
                + 'static,
        {
            new(self.clone(), callback, options);
        }
    }

    // impl<E, F, R> AddIntersectionObserver<F, R> for NodeRef<E>
    // where
    //     E: ElementType,
    //     E::Output: JsCast + Clone + 'static + Into<HtmlElement>,
    //     R: ElementType,
    //     R::Output: JsCast + Clone + 'static + Into<HtmlElement>,
    //     F: FnMut(IntersectionObserverEntry, IntersectionObserver) + Send + Sync + Clone + 'static,
    // {
    //     fn add_intersection_observer(&self, callback: F, options: Options<R>) {
    //         new(self.clone(), callback, options);
    //     }
    // }

    #[derive(Default, Clone)]
    pub struct GlobalState {
        pub observer: StoredValue<HashMap<u64, SendWrapper<IntersectionObserver>>>,
        pub callbacks: StoredValue<
            HashMap<
                Uuid,
                Box<
                    dyn FnMut(IntersectionObserverEntry, IntersectionObserver)
                        + Send
                        + Sync
                        + 'static,
                >,
            >,
        >,
    }

    // #[derive(Default, Clone)]
    // pub struct Options<E>
    // where
    //     E: ElementType,
    //     E::Output: JsCast + Clone + 'static + Into<HtmlElement>,
    // {
    //     root: Option<NodeRef<E>>,
    //     root_margin: Option<String>,
    //     threshold: Option<OrderedFloat<f64>>,
    // }

    // //#[derive(Default, Clone)]
    // pub struct Options {
    //     root: Box<dyn Into<HtmlElement>>,
    //     root_margin: Option<String>,
    //     threshold: Option<OrderedFloat<f64>>,
    // }

    #[derive(Clone)]
    pub struct Options<E = leptos::html::Div>
    where
        E: ElementType,
        E::Output: JsCast + Clone + 'static + Into<HtmlElement>,
    {
        root: Option<NodeRef<E>>,
        root_margin: Option<String>,
        threshold: Option<OrderedFloat<f64>>,
    }

    impl<E> Default for Options<E>
    where
        E: ElementType,
        E::Output: JsCast + Clone + 'static + Into<HtmlElement>,
    {
        fn default() -> Self {
            Self {
                root: None,
                root_margin: None,
                threshold: None,
            }
        }
    }

    impl<E> Options<E>
    where
        E: ElementType,
        E::Output: JsCast + Clone + 'static + Into<HtmlElement>,
    {
        pub fn set_root(mut self, root: NodeRef<E>) -> Self {
            self.root = Some(root);
            self
        }

        pub fn set_root_margin(mut self, root_margin: String) -> Self {
            self.root_margin = Some(root_margin);
            self
        }

        pub fn set_threshold(mut self, threshold: f64) -> Self {
            self.threshold = Some(OrderedFloat(threshold));
            self
        }
    }

    // impl <E> Options<E> where
    //     E: ElementType,
    //     E::Output: JsCast + Clone + 'static + Into<HtmlElement>, {
    //     pub fn into_
    // }

    // impl <E> Into<IntersectionObserverInit> for Options<E> where  E: ElementType,
    // E::Output: JsCast + Clone + 'static + Into<HtmlElement> {
    //     fn into(self) -> IntersectionObserverInit {
    //         let init = IntersectionObserverInit::new();
    //         if let Some(root) =
    //         init
    //     }
    // }

    impl<E> Hash for Options<E>
    where
        E: ElementType,
        E::Output: JsCast + Clone + 'static + Into<HtmlElement>,
    {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.root
                .as_ref()
                .and_then(|v| {
                    let root: HtmlElement = v.get().unwrap().into();
                    root.get_attribute(ID_FIELD_ROOT_NAME)
                })
                .hash(state);
            self.root_margin.hash(state);
            self.threshold.hash(state);
        }
    }

    // pub fn init_global_state() {
    //     provide_context(GlobalState::default());

    //     Effect::new(move || {
    //         let ctx = expect_context::<GlobalState>();

    //         let observer = new_raw(move |entries, observer| {
    //             ctx.callbacks.update_value(|callbacks| {
    //                 for entry in entries {
    //                     let target = entry.target();
    //                     let Some(id) = get_id(&target, ID_FIELD_NAME) else {
    //                         continue;
    //                     };

    //                     let Some(callback) = callbacks.get_mut(&id) else {
    //                         continue;
    //                     };
    //                     callback(entry, observer.clone());
    //                 }
    //             });
    //         });

    //         ctx.observer.set(Some(SendWrapper::new(observer)));
    //     });
    // }

    // pub fn new<E, R, F>(target: NodeRef<E>, mut callback: F, options: Options<R>)
    // where
    //     E: ElementType,
    //     E::Output: JsCast + Clone + 'static + Into<HtmlElement>,
    //     R: ElementType,
    //     R::Output: JsCast + Clone + 'static + Into<HtmlElement>,
    //     F: FnMut(IntersectionObserverEntry, IntersectionObserver) + Clone + Send + Sync + 'static,
    // {

    pub fn new<E, R, F>(target: NodeRef<E>, mut callback: F, options: Options<R>)
    where
        E: ElementType,
        E::Output: JsCast + Clone + 'static + Into<HtmlElement>,
        R: ElementType,
        R::Output: JsCast + Clone + 'static + Into<HtmlElement>,
        F: FnMut(IntersectionObserverEntry, IntersectionObserver) + Clone + Send + Sync + 'static,
    {
        let ctx = match use_context::<GlobalState>() {
            Some(v) => v,
            None => {
                provide_context(GlobalState::default());
                let ctx = expect_context::<GlobalState>();

                // Effect::new(move || {
                //     let mut hasher = DefaultHasher::new();
                //     options.hash(&mut hasher);
                //     let hash = hasher.finish();
                //     trace!("hash of options: {}", hash);

                //     let root = if let Some(root) = &options.root {
                //         if let Some(root) = root.get() {
                //             let root: HtmlElement = root.into();
                //             Some(root)
                //         } else {
                //             return;
                //         }
                //     } else {
                //         None
                //     };

                //     let observer_settings = IntersectionObserverInit::new();

                //     if let Some(root) = root {
                //         observer_settings.set_root(Some(&root));
                //     }

                //     if let Some(margin) = &options.root_margin {
                //         observer_settings.set_root_margin(margin);
                //     }

                //     if let Some(threshold) = options.threshold {
                //         observer_settings.set_threshold(&JsValue::from_f64(*threshold));
                //     }

                //     let observer = new_with_options_raw(
                //         move |entries, observer| {
                //             ctx.callbacks.update_value(|callbacks| {
                //                 for entry in entries {
                //                     let target = entry.target();
                //                     let Some(id) = get_id(&target, ID_FIELD_NAME) else {
                //                         continue;
                //                     };

                //                     let Some(callback) = callbacks.get_mut(&id) else {
                //                         continue;
                //                     };
                //                     callback(entry, observer.clone());
                //                 }
                //             });
                //         },
                //         &observer_settings,
                //     );

                //     ctx.observer.set(Some(SendWrapper::new(observer)));
                // });

                ctx
            }
        };
        let id = Uuid::new_v4();
        let options_hash = StoredValue::new(None::<u64>);

        Effect::new(move || {
            let span = trace_span!("intersection observer", "{}", &id).entered();

            let Some(target) = target.get() else {
                return;
            };

            let root: Option<HtmlElement> = match &options.root {
                Some(v) => match v.get() {
                    Some(v) => {
                        let root: HtmlElement = v.into();

                        Some(root)
                    }
                    None => {
                        trace!("root is not ready");
                        return;
                    }
                },
                None => None,
            };

            trace!("root options parsed");

            let mut hasher = DefaultHasher::new();
            options.hash(&mut hasher);
            let hash = hasher.finish();
            options_hash.set_value(Some(hash));
            trace!("hash of options: {}", hash);

            let target: HtmlElement = target.into();

            set_id(&target, ID_FIELD_NAME, id);

            ctx.callbacks.update_value(|v| {
                v.insert(id, Box::new(callback.clone()));
                trace!("created callback");
            });

            ctx.observer.update_value(|observers| {
                trace!("getting observer...");
                match observers.get_mut(&hash) {
                    Some(observer) => {
                        observer.observe(&target);
                        trace!("observer already exists");
                    }
                    None => {
                        trace!("no observer found");

                        let observer_settings = IntersectionObserverInit::new();

                        if let Some(root) = root {
                            trace!("root option set");
                            observer_settings.set_root(Some(&root));
                        }

                        if let Some(margin) = &options.root_margin {
                            trace!("margin option set");
                            observer_settings.set_root_margin(margin);
                        }

                        if let Some(threshold) = options.threshold {
                            trace!("threshold option set");
                            observer_settings.set_threshold(&JsValue::from_f64(*threshold));
                        }

                        trace!("creating raw observer");
                        let observer = new_with_options_raw(
                            move |entries, observer| {
                                ctx.callbacks.update_value(|callbacks| {
                                    for entry in entries {
                                        let target = entry.target();
                                        let Some(id) = get_id(&target, ID_FIELD_NAME) else {
                                            continue;
                                        };

                                        let Some(callback) = callbacks.get_mut(&id) else {
                                            continue;
                                        };
                                        callback(entry, observer.clone());
                                    }
                                });
                            },
                            &observer_settings,
                        );

                        trace!("inserting raw observer...");
                        // trace!("getting observer entry");
                        let observer = observers.entry(hash).or_insert(SendWrapper::new(observer));
                        observer.observe(&target);
                        trace!("observer created");

                        // ctx.observer.update_value(|observers| {

                        // });
                    }
                };
            });

            // ctx.observer.update_value(|observers| {
            //     let observer = observers.entry(id).or_insert_with(|| {
            //         let observer_settings = IntersectionObserverInit::new();

            //         if let Some(root) = root {
            //             observer_settings.set_root(Some(&root));
            //         }

            //         if let Some(margin) = &options.root_margin {
            //             observer_settings.set_root_margin(margin);
            //         }

            //         if let Some(threshold) = options.threshold {
            //             observer_settings.set_threshold(&JsValue::from_f64(*threshold));
            //         }

            //         let observer = new_with_options_raw(
            //             move |entries, observer| {
            //                 ctx.callbacks.update_value(|callbacks| {
            //                     for entry in entries {
            //                         let target = entry.target();
            //                         let Some(id) = get_id(&target, ID_FIELD_NAME) else {
            //                             continue;
            //                         };

            //                         let Some(callback) = callbacks.get_mut(&id) else {
            //                             continue;
            //                         };
            //                         callback(entry, observer.clone());
            //                     }
            //                 });
            //             },
            //             &observer_settings,
            //         );
            //         RwSignal::new(None)
            //     });
            // });

            // let root = if let Some(root) = &options.root {
            //     if let Some(root) = root.get() {
            //         let root: HtmlElement = root.into();
            //         Some(root)
            //     } else {
            //         return;
            //     }
            // } else {
            //     None
            // };

            // observer.observe(&target);

            span.exit();
        });

        on_cleanup(move || {
            let span = trace_span!("intersection observer").entered();
            //     let mut hasher = DefaultHasher::new();
            //     options.hash(&mut hasher);
            //     let hash = hasher.finish();
            //     trace!("hash of options: {}", hash);

            //     let root = if let Some(root) = &options.root {
            //         if let Some(root) = root.get() {
            //             let root: HtmlElement = root.into();
            //             Some(root)
            //         } else {
            //             return;
            //         }
            //     } else {
            //         None
            //     };

            let Some(target) = target.get_untracked() else {
                return;
            };

            let Some(options_hash) = options_hash.get_value() else {
                return;
            };

            let target: HtmlElement = target.into();

            let Some(id) = get_id(&target, ID_FIELD_NAME) else {
                return;
            };

            ctx.observer
                .with_value(|observers| match observers.get(&options_hash) {
                    Some(observer) => {
                        observer.unobserve(&target);
                    }
                    None => {
                        warn!("observer not found with hash {} for {}", options_hash, id);
                    }
                });

            ctx.callbacks.update_value(|callbacks| {
                callbacks.remove(&id);
                trace!("removed {}", &id);
            });

            span.exit();
        });
    }

    pub fn new_closure(
        mut callback: impl FnMut(Vec<IntersectionObserverEntry>, IntersectionObserver) + 'static,
    ) -> JsValue {
        Closure::<dyn FnMut(Array, IntersectionObserver)>::new(
            move |entries: Array, observer: IntersectionObserver| {
                let entries: Vec<IntersectionObserverEntry> = entries
                    .to_vec()
                    .into_iter()
                    .map(|v| v.unchecked_into::<IntersectionObserverEntry>())
                    .collect();
                callback(entries, observer);
            },
        )
        .into_js_value()
    }

    pub fn new_raw<F>(callback: F) -> IntersectionObserver
    where
        F: FnMut(Vec<IntersectionObserverEntry>, IntersectionObserver) + Clone + 'static,
    {
        IntersectionObserver::new(new_closure(callback).as_ref().unchecked_ref()).unwrap()
    }

    pub fn new_with_options_raw<F>(
        callback: F,
        options: &IntersectionObserverInit,
    ) -> IntersectionObserver
    where
        F: FnMut(Vec<IntersectionObserverEntry>, IntersectionObserver) + Clone + 'static,
    {
        IntersectionObserver::new_with_options(
            new_closure(callback).as_ref().unchecked_ref(),
            options,
        )
        .unwrap()
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

    use super::uuid::{get_id, set_id};

    const ID_FIELD_NAME: &str = "data-leptos_toolbox_resize_observer_id";

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
                        let Some(id) = get_id(&target, ID_FIELD_NAME) else {
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

            set_id(&target, ID_FIELD_NAME, id);

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

            let Some(id) = get_id(&target, ID_FIELD_NAME) else {
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

pub mod file {
    use thiserror::Error;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{
        DragEvent, File, FileList, ReadableStreamDefaultReader,
        js_sys::{Object, Reflect, Uint8Array},
    };

    #[derive(Error, Debug)]
    pub enum ErrorGetFileStream {
        #[error("failed to cast as \"ReadableStreamDefaultReader\" \"{0}\"")]
        Cast(String),
    }

    #[derive(Error, Debug)]
    pub enum ErrorGetStreamChunk {
        #[error("failed to get chunk \"{0}\"")]
        GetChunk(String),

        #[error("failed to cast chunk to object \"{0}\"")]
        CastToObject(String),

        #[error("failed to cast chunk to Uint8Array \"{0}\"")]
        CastToArray(String),

        #[error("failed to read 'done' field from chunk object \"{0}\"")]
        ReadingFieldDone(String),

        #[error("failed to read 'value' field from chunk object \"{0}\"")]
        ReadingFieldValue(String),
    }

    pub trait PushChunkToVec {
        fn push_to_vec(&self, buffer: &mut Vec<u8>);
    }

    pub trait GetFiles {
        fn get_files(&self) -> Vec<File>;
    }

    pub trait GetFileStream {
        fn get_file_stream(&self) -> Result<ReadableStreamDefaultReader, ErrorGetFileStream>;
    }

    pub trait GetStreamChunk {
        async fn get_stream_chunk(&self) -> Result<Option<Uint8Array>, ErrorGetStreamChunk>;
    }

    impl PushChunkToVec for Uint8Array {
        fn push_to_vec(&self, buffer: &mut Vec<u8>) {
            let chunk = self;
            let data_len = buffer.len();
            buffer.resize(data_len + chunk.length() as usize, 0);
            chunk.copy_to(&mut buffer[data_len..]);
        }
    }

    impl GetStreamChunk for ReadableStreamDefaultReader {
        async fn get_stream_chunk(&self) -> Result<Option<Uint8Array>, ErrorGetStreamChunk> {
            get_stream_chunk(self).await
        }
    }

    impl GetFileStream for File {
        fn get_file_stream(&self) -> Result<ReadableStreamDefaultReader, ErrorGetFileStream> {
            get_file_stream(self)
        }
    }

    impl GetFiles for DragEvent {
        fn get_files(&self) -> Vec<File> {
            get_files(self)
        }
    }

    pub fn get_files(drag_event: &DragEvent) -> Vec<File> {
        let Some(files) = drag_event.data_transfer().and_then(|v| v.files()) else {
            return Vec::new();
        };
        let files = (0..files.length())
            .filter_map(|i| files.get(i))
            .collect::<Vec<File>>();
        files
    }

    pub fn get_file_stream(file: &File) -> Result<ReadableStreamDefaultReader, ErrorGetFileStream> {
        let stream = file.stream();
        let reader = stream
            .get_reader()
            .dyn_into::<ReadableStreamDefaultReader>()
            .map_err(|e| {
                ErrorGetFileStream::Cast(
                    e.as_string()
                        .unwrap_or_else(|| String::from("uwknown error")),
                )
            })?;
        Ok(reader)
    }

    pub async fn get_stream_chunk(
        reader: &ReadableStreamDefaultReader,
    ) -> Result<Option<Uint8Array>, ErrorGetStreamChunk> {
        let promise = reader.read();
        let chunk = JsFuture::from(promise)
            .await
            .map_err(|e| {
                ErrorGetStreamChunk::GetChunk(
                    e.as_string()
                        .unwrap_or_else(|| String::from("uwknown error")),
                )
            })?
            .dyn_into::<Object>()
            .map_err(|e| {
                ErrorGetStreamChunk::CastToObject(
                    e.as_string()
                        .unwrap_or_else(|| String::from("uwknown error")),
                )
            })?;
        let done = Reflect::get(&chunk, &"done".into()).map_err(|e| {
            ErrorGetStreamChunk::ReadingFieldDone(
                e.as_string()
                    .unwrap_or_else(|| String::from("uwknown error")),
            )
        })?;
        if done.is_truthy() {
            return Ok(None);
        }
        let chunk = Reflect::get(&chunk, &"value".into())
            .map_err(|e| {
                ErrorGetStreamChunk::ReadingFieldValue(
                    e.as_string()
                        .unwrap_or_else(|| String::from("uwknown error")),
                )
            })?
            .dyn_into::<Uint8Array>()
            .map_err(|e| {
                ErrorGetStreamChunk::CastToArray(
                    e.as_string()
                        .unwrap_or_else(|| String::from("uwknown error")),
                )
            })?;

        Ok(Some(chunk))
    }
}

pub mod dropzone {

    use std::{
        cell::RefCell,
        fmt::Display,
        future::Future,
        ops::{Deref, DerefMut},
        rc::Rc,
        sync::{Arc, Mutex},
        time::SystemTime,
    };

    // use gloo::file::{File, FileList, FileReadError};
    use leptos::{ev, html::ElementType, prelude::*, task::spawn_local};
    use tracing::{debug, error, trace, trace_span};
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{
        DragEvent, HtmlElement, ReadableStreamDefaultReader,
        js_sys::{self, Object, Promise, Reflect, Uint8Array},
    };

    use super::event_listener;
    use super::file::{GetFileStream, GetFiles, GetStreamChunk};

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
    // #[track_caller]
    // pub fn ws_loc_key() -> u128 {
    //     xxhash_rust::xxh3::xxh3_128(std::panic::Location::caller().to_string().as_bytes())
    // }

    pub trait AddDropZone {
        fn on_file_drop<F, R>(&self, callback: F)
        where
            R: Future<Output = anyhow::Result<()>> + 'static,
            F: FnMut(Event, DragEvent) -> R + 'static;
    }

    // pub trait GetFiles {
    //     fn files(&self) -> gloo::file::FileList;
    // }

    // pub trait GetFileData {
    //     async fn data(&self) -> Result<Vec<u8>, FileReadError>;
    // }

    impl<E> AddDropZone for NodeRef<E>
    where
        E: ElementType,
        E::Output: JsCast + Clone + 'static + Into<HtmlElement>,
    {
        #[track_caller]
        fn on_file_drop<F, R>(&self, callback: F)
        where
            R: Future<Output = anyhow::Result<()>> + 'static,
            F: FnMut(Event, DragEvent) -> R + 'static,
        {
            new(self.clone(), callback);
        }
    }

    // impl GetFileData for gloo::file::File {
    //     async fn data(&self) -> Result<Vec<u8>, FileReadError> {
    //         // let r = web_sys::FileReader::new().unwrap();
    //         // r.se
    //         gloo::file::futures::read_as_bytes(self).await
    //     }
    // }

    // impl GetFiles for DragEvent {
    //     fn files(&self) -> gloo::file::FileList {
    //         let Some(files) = self.data_transfer().and_then(|v| v.files()) else {
    //             trace!("shouldnt be here");
    //             return gloo::file::FileList::from(web_sys::FileList::from(JsValue::null()));
    //         };
    //         {
    //             let files = files.clone();
    //             let event = self.clone();
    //             trace!("spawning...");
    //             spawn_local(async move {
    //                 for files in event.get_files() {}
    //                 // debug!("im in spawn!");
    //                 // let files = (0..files.length())
    //                 //     .map(|i| files.get(i))
    //                 //     .collect::<Vec<Option<web_sys::File>>>();
    //                 // // let reader = web_sys::FileReader::new().unwrap();
    //                 // // // reader.result();

    //                 // // let closure = Closure::<dyn FnMut()>::new({
    //                 // //     let reader = reader.clone();
    //                 // //     move || {
    //                 // //         let result = reader.result().unwrap();
    //                 // //         let result = js_sys::Uint8Array::new(&result).to_vec();
    //                 // //         let result_str = String::from_utf8_lossy(&result);
    //                 // //         trace!("ohohohoho: {}", result_str);
    //                 // //     }
    //                 // // })
    //                 // // .into_js_value();

    //                 // // reader.set_onloadend(Some(closure.as_ref().unchecked_ref()));

    //                 // for file in files {
    //                 //     let file = file.unwrap();
    //                 //     let stream = file.stream();
    //                 //     let reader = stream
    //                 //         .get_reader()
    //                 //         .dyn_into::<ReadableStreamDefaultReader>()
    //                 //         .unwrap();
    //                 //     let mut data = Vec::<u8>::new();

    //                 //     loop {
    //                 //         let promise = reader.read();
    //                 //         let chunk = JsFuture::from(promise)
    //                 //             .await
    //                 //             .unwrap()
    //                 //             .dyn_into::<Object>()
    //                 //             .unwrap();
    //                 //         let done = js_sys::Reflect::get(&chunk, &"done".into()).unwrap();
    //                 //         if done.is_truthy() {
    //                 //             debug!("its done");
    //                 //             break;
    //                 //         }
    //                 //         let chunk = Reflect::get(&chunk, &"value".into())
    //                 //             .unwrap()
    //                 //             .dyn_into::<Uint8Array>()
    //                 //             .unwrap();
    //                 //         let data_len = data.len();
    //                 //         debug!("chunk len: {}", chunk.length());
    //                 //         data.resize(data_len + chunk.length() as usize, 0);
    //                 //         chunk.copy_to(&mut data[data_len..]);
    //                 //         // data.resize(data_len + chunk.length() as usize, 255);
    //                 //         // chunk.copy_to(&mut data);
    //                 //     }
    //                 //     let s = String::from_utf8_lossy(&data);
    //                 //     debug!("full data: {:?}", s);
    //                 //     // reader.read_as_array_buffer(&file).unwrap();
    //                 // }
    //             });
    //         }
    //         trace!("len: {}", files.length());
    //         gloo::file::FileList::from(files)
    //     }
    // }

    #[track_caller]
    pub fn new<E, F, R>(target: NodeRef<E>, mut callback: F)
    where
        E: ElementType,
        E::Output: JsCast + Clone + 'static + Into<HtmlElement>,
        R: Future<Output = anyhow::Result<()>> + 'static,
        F: FnMut(Event, DragEvent) -> R + 'static,
    {
        let callback_location = *std::panic::Location::caller();
        let callback = Rc::new(RefCell::new(callback));

        event_listener::new(target, ev::dragstart, {
            let callback = callback.clone();
            move |e| {
                let callback = callback.clone();
                let fut = async move {
                    let mut callback = callback.borrow_mut();
                    let result = callback(Event::Start, e).await;

                    trace!("tracking 3: {}", std::panic::Location::caller().to_string());
                    if let Err(err) = result {
                        error!("dropzone error at: {}: {}", callback_location, err);
                    }
                };
                spawn_local(fut);
            }
        });

        event_listener::new(target, ev::dragleave, {
            let callback = callback.clone();

            move |e| {
                let callback = callback.clone();
                let fut = async move {
                    let mut callback = callback.borrow_mut();
                    let result = callback(Event::Leave, e).await;
                    if let Err(err) = result {
                        error!("dropzone error at: {}: {}", callback_location, err);
                    }
                };
                spawn_local(fut);
            }
        });

        event_listener::new(target, ev::dragenter, {
            let callback = callback.clone();

            move |e| {
                let callback = callback.clone();
                let fut = async move {
                    let mut callback = callback.borrow_mut();
                    let result = callback(Event::Enter, e).await;
                    if let Err(err) = result {
                        error!("dropzone error at: {}: {}", callback_location, err);
                    }
                };
                spawn_local(fut);
            }
        });

        event_listener::new(target, ev::dragover, {
            let callback = callback.clone();

            move |e| {
                e.prevent_default();

                let callback = callback.clone();
                let fut = async move {
                    let mut callback = callback.borrow_mut();
                    let result = callback(Event::Over, e).await;
                    if let Err(err) = result {
                        error!("dropzone error at: {}: {}", callback_location, err);
                    }
                };
                spawn_local(fut);
            }
        });

        event_listener::new(target, ev::drop, {
            let callback = callback.clone();

            move |e| {
                e.prevent_default();
                e.stop_propagation();

                let callback = callback.clone();
                let fut = async move {
                    let mut callback = callback.borrow_mut();
                    let result = callback(Event::Drop, e).await;
                    if let Err(err) = result {
                        error!("dropzone error at: {}: {}", callback_location, err);
                    }
                };
                spawn_local(fut);
            }
        });
    }
}
