use wasm_bindgen::prelude::wasm_bindgen;
use leptos::logging::log;
use web_sys::MessageEvent;

#[wasm_bindgen]
pub fn worker_process_data(i: i32)-> i32 {
    log!("processing: {i}");
    i + 1
}

#[wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();

    tracing::trace!("loading stuff...");

    let script = r#"

            onmessage = (e) => {
                console.log("worker received", e.data);
                e.data();
            }

        "#;

    let script = js_sys::Array::from_iter([js_sys::Uint8Array::from(script.as_bytes())]);
    let script: &wasm_bindgen::JsValue = script.as_ref();
    let mut blob_property_bag = web_sys::BlobPropertyBag::new();
    blob_property_bag.type_("module");
    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(
            script,
            &blob_property_bag,
        ).unwrap();
    let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();

    let worker = web_sys::Worker::new(&url).unwrap();

    let on_recv = move |e: wasm_bindgen::JsValue| {
        let e: MessageEvent = e.into();
        let data: i32 = e.data().as_f64().unwrap() as i32;
        log!("received: {}", data); 
    };
    let on_recv: wasm_bindgen::closure::Closure<dyn Fn(wasm_bindgen::JsValue)> = wasm_bindgen::closure::Closure::new(on_recv);
    let on_recv = on_recv.into_js_value();
    let on_recv = js_sys::Function::from(on_recv);

    worker.set_onmessage(Some(&on_recv));

    let payload_fn = move || {
        log!("IM RUNNING IN WORKER");
    };
    let payload_fn: wasm_bindgen::closure::Closure<dyn Fn()> = wasm_bindgen::closure::Closure::new(payload_fn);
    let payload_fn = payload_fn.into_js_value();
    //let payload_fn = js_sys::Function::from(payload_fn);
    
    worker.post_message(&payload_fn).unwrap();
}



// use wasm_bindgen::prelude::wasm_bindgen;
// use leptos::logging::log;
// use web_sys::MessageEvent;

// #[wasm_bindgen]
// pub fn worker_process_data(i: i32)-> i32 {
//     log!("processing: {i}");
//     i + 1
// }

// #[wasm_bindgen]
// pub fn hydrate() {
//     console_error_panic_hook::set_once();

//     tracing::trace!("loading stuff...");

//     let script = r#"
//             console.log("1");

//             let fn_worker_process_data = null;
//             let queue = [];

//             import("http://localhost:8080/pkg/leptos_start5.js").then((mod) => {
//                 console.log("3");
//                 mod.default("http://localhost:8080/pkg/leptos_start5_bg.wasm").then(() => {
//                     console.log("4");
//                     fn_worker_process_data = mod.worker_process_data;

//                     for (let payload of queue) {
//                         let result = mod.worker_process_data(payload);
//                         postMessage(result);
//                     }

//                 });
//             });

//             console.log("2");

//             onmessage = (e) => {
//                 console.log("worker received", e.data);
//                 if (fn_worker_process_data) {
//                     fn_worker_process_data(e.data);
//                 } else {
//                     queue.push(e.data);
//                 }
//             }

//         "#;

//     let script = js_sys::Array::from_iter([js_sys::Uint8Array::from(script.as_bytes())]);
//     let script: &wasm_bindgen::JsValue = script.as_ref();
//     let mut blob_property_bag = web_sys::BlobPropertyBag::new();
//     blob_property_bag.type_("module");
//     let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(
//             script,
//             &blob_property_bag,
//         ).unwrap();
//     let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();

//     let worker = web_sys::Worker::new(&url).unwrap();

//     let on_recv = move |e: wasm_bindgen::JsValue| {
//         let e: MessageEvent = e.into();
//         let data: i32 = e.data().as_f64().unwrap() as i32;
//         log!("received: {}", data); 
//     };
//     let on_recv: wasm_bindgen::closure::Closure<dyn Fn(wasm_bindgen::JsValue)> = wasm_bindgen::closure::Closure::new(on_recv);
//     let on_recv = on_recv.into_js_value();
//     let on_recv = js_sys::Function::from(on_recv);

//     worker.set_onmessage(Some(&on_recv));
    
//     let payload: usize = 9000;
//     let payload = wasm_bindgen::JsValue::from(payload);
//     worker.post_message(&payload).unwrap();
// }















































// pub mod app;
// use app::App;
// use wasm_bindgen::prelude::wasm_bindgen;
// use wasm_bindgen::JsCast;


// struct WASMTracingLayer {
//     pub spans: std::sync::Arc<std::sync::Mutex<indexmap::IndexMap<tracing::span::Id, String>>>,
//     pub config: WASMTracingConfig,
// }

// struct WASMTracingConfig {
//     pub target: bool,
//     pub line: bool,
// }

// struct Executer {
//     ready_queue: std::sync::mpsc::Receiver<std::sync::Arc<Task>>
// }

// struct Spawner {
//     task_sender: std::sync::mpsc::SyncSender<std::sync::Arc<Task>>
// }

// struct Task {
//     future: std::sync::Mutex<Option<futures::future::BoxFuture<'static, ()>>>,
//     task_sender: std::sync::mpsc::SyncSender<std::sync::Arc<Task>>
// }

// impl Spawner {
//     pub fn spawn(&self, future: impl std::future::Future<Output = ()> + 'static + Send) {
//         let future = std::boxed::Box::pin(future);
//         let task = std::sync::Arc::new(Task {
//             future: std::sync::Mutex::new(Some(future)),
//             task_sender: self.task_sender.clone(),
//         });

//         self.task_sender.send(task).expect("too many tasks");
//     }
// }

// impl Executer {
//     pub fn run(&self) {
//         while let Ok(task) = self.ready_queue.recv() {
//             let mut future_slot = task.future.lock().unwrap();
//             if let Some(mut future) = future_slot.take() {
//                 let waker = futures::task::waker_ref(&task);
//                 let context = &mut std::task::Context::from_waker(&waker);
//                 if future.as_mut().poll(context).is_pending() {
//                     *future_slot = Some(future);
//                 }
//             }
//         }
//     }
// }

// impl futures::task::ArcWake for Task {
//     fn wake_by_ref(arc_self: &std::sync::Arc<Self>) {
//         let cloned = arc_self.clone();
//         arc_self.task_sender.send(cloned).expect("too many tasks queued");
//     }
// }

// fn new_executor_and_spawner() -> (Executer, Spawner) {
//     const MAX_QUEUED_TASKS: usize = 10_000;
//     let (task_sender, ready_queue) = std::sync::mpsc::sync_channel(MAX_QUEUED_TASKS);
//     (Executer { ready_queue }, Spawner { task_sender })
// }

// static HELLO: std::sync::OnceLock<std::pin::Pin<Box<i32>>> = std::sync::OnceLock::new();

// lazy_static::lazy_static! {
//     static ref ARRAY: std::sync::Mutex<Vec<u8>> = std::sync::Mutex::new(vec![]);
// }

// // #[wasm_bindgen]
// // //#[wasm_bindgen(variadic)]
// // //&wasm_bindgen::JsValue
// // pub fn hello(pp: usize) {
// //     //f: &js_sys::Function
// //     console_error_panic_hook::set_once();

// //     tracing::subscriber::set_global_default(
// //         tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt::with(
// //             tracing_subscriber::Registry::default(),
// //             WASMTracingLayer::new(WASMTracingConfig {
// //                 line: false,
// //                 target: false,
// //             }),
// //         ),
// //     )
// //     .unwrap();

// //     let p5 = pp as *mut [u8; 4];
// //     unsafe {
// //         tracing::trace!("im in danger :3 {pp}: {:?}", *p5);
// //     }

// //     let b: &mut Vec<u8> = &mut ARRAY.lock().unwrap();
// //     tracing::trace!("aaaaaaa {:?}", b);

// //     //tracing::trace!("im in danger :3 {}", p);

// //     //let this = wasm_bindgen::JsValue::null();

// //     //f.call0(&this).unwrap();
// //     //let v = HELLO.get().unwrap();
// //     //let v = &HELLO.get();
// //     //tracing::trace!("im in danger :3 {:p}", v);
// // }
// use leptos::logging::log;

// #[wasm_bindgen]
// pub fn hydrate() {
//     console_error_panic_hook::set_once();
 
//     // tracing_subscriber::fmt()
//     //     .with_writer(
//     //         // To avoide trace events in the browser from showing their
//     //         // JS backtrace, which is very annoying, in my opinion
//     //         tracing_subscriber_wasm::MakeConsoleWriter::default(), //.map_trace_level_to(tracing::Level::TRACE),
//     //     )
//     //     .with_ansi(false)
//     //     .with_max_level(tracing::Level::TRACE)
//     //     .with_env_filter(
//     //         <tracing_subscriber::EnvFilter as std::str::FromStr>::from_str(
//     //             "artcord_leptos=trace,artcord_leptos_web_sockets=trace",
//     //         )
//     //         .unwrap(),
//     //     )
//     //     .with_file(true)
//     //     .with_line_number(true)
//     //     .without_time()
//     //     .with_thread_ids(true)
//     //     .with_thread_names(true)
//     //     // For some reason, if we don't do this in the browser, we get
//     //     // a runtime error.
//     //     .init();

//     tracing::subscriber::set_global_default(
//         tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt::with(
//             tracing_subscriber::Registry::default(),
//             WASMTracingLayer::new(WASMTracingConfig {
//                 line: true,
//                 target: false,
//             }),
//         ),
//     )
//     .unwrap();


//     tracing::trace!("hello");




//     let script = r#"
//             console.log("1");
//             onmessage = (e) => {
//                 console.log(420, e.data);
//                 postMessage(0);
//             }

//         "#;

//     let script = js_sys::Array::from_iter([js_sys::Uint8Array::from(script.as_bytes())]);
//     let script: &wasm_bindgen::JsValue = script.as_ref();
//     let mut blob_property_bag = web_sys::BlobPropertyBag::new();
//     blob_property_bag.type_("module");
//     let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(
//             script,
//             &blob_property_bag,
//         ).unwrap();
//     let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();

//     let worker = web_sys::Worker::new(&url).unwrap();

//     let foo: usize = 9000;
//     let foo = wasm_bindgen::JsValue::from(foo);

//     let bar = move |e: wasm_bindgen::JsValue| {
//         log!("amazing");
//     };
//     let bar: wasm_bindgen::closure::Closure<dyn Fn(wasm_bindgen::JsValue)> = wasm_bindgen::closure::Closure::new(bar);
//     //let bar = bar.as_ref();
//     let bar = bar.into_js_value();
//     let bar = js_sys::Function::from(bar);

//     worker.post_message(&foo).unwrap();
//     worker.set_onmessage(Some(&bar));









//     //=============

//     // let (executor, spawner) = new_executor_and_spawner();

//     // // spawner.spawn(async {
//     // //     tracing::trace!("task 1");
//     // // });

//     // let mut blob_property_bag = web_sys::BlobPropertyBag::new();
//     // //blob_property_bag.type_("application/javascript");
//     // blob_property_bag.type_("module");

//     // let script = r#"
//     //     console.log("1");
//     //     let a = importScripts("http://localhost:3000/pkg/leptos_start5.js");
//     //     console.log("2");
//     //     //throw "wtf";
//     //     console.log("3");

//     //     var f;
//     //     var b;
//     //     onmessage = (e) => {
//     //         console.log(420, e.data);
//     //         if (f) {
//     //             f(e.data);
//     //         } else {
//     //             b = e.data; 
//     //         }
            
//     //         // a.hello()
//     //     }

//     //     async function init_my_stuff() {
//     //         let a = await wasm_bindgen("http://localhost:3000/pkg/leptos_start5_bg.wasm");
//     //         // console.log(69, a.hello);
//     //         return a;
//     //     }
//     //     init_my_stuff().then(a => {
//     //         if (b) {
//     //             a.hello(b);
//     //         } else {
//     //          f = a.hello;
//     //         }
            
//     //     });
//     //     // let a = await init_my_stuff();
//     //     // //console.log(666, a.hello);


       

//     //     // let a = await wasm_bindgen("http://localhost:3000/pkg/leptos_start5_bg.wasm");
//     //     // console.log("3");

//     //     //console.log("no", importScripts, a);
//     //     // let a = await wasm_bindgen("http://localhost:3000/pkg/leptos_start5_bg.wasm");
//     //     // a.hello(777);

//     //     // var a;
      


        
//     //     //

//     //     //let a = await wasm_bindgen("/pkg/leptos_start5_bg.wasm");
//     //     //console.log(a.hello());

//     //     // import("/pkg/leptos_start5.js").then((mod) => {
//     //     //   mod.default("/pkg/leptos_start5_bg.wasm").then(() => {
//     //     //     mod.hello();
//     //     //     console.log("what?");
//     //     //   });
//     //     // });
//     // "#;
//     // // let script = r#"
//     // //     console.log("no");
//     // // "#;
//     // let script = js_sys::Array::from_iter([js_sys::Uint8Array::from(script.as_bytes())]);
//     // let script: &wasm_bindgen::JsValue = script.as_ref();

    
    
    
//     // let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(
//     //     script,
//     //     &blob_property_bag,
//     // ).unwrap();

//     // let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();

//     // {
//     //     ARRAY.lock().unwrap().push(1);
//     // }
//     // HELLO.set(std::boxed::Box::pin(2222)).unwrap();
//     // let h = HELLO.get();
//     // let p: *const std::option::Option<&std::pin::Pin<Box<i32>>> = &h;
//     // let p2 = std::ptr::addr_of!(h);
//     // let p3 = p2 as *const usize;
//     // let p4 = p3 as usize;
//     // //let pp = std::ptr::addr_of!(HELLO) as usize;
//     // let pp = 1 as usize;
//     // //let pp = p as usize;

//     // let p5 = pp as *mut [u8; 4];
//     // //let p5 = pp as *const std::pin::Pin<Box<i32>>;
//     // unsafe {
//     //     let a = &mut *p5;
//     //     a[0] = 1;
//     //     tracing::trace!("{pp}: {:?}", a);
//     // }
//     // let worker = web_sys::Worker::new(&url).unwrap();
//     // // let f = move || {
//     // //     tracing::trace!("from f");
//     // // };
//     // let f = move |e: wasm_bindgen::JsValue| {
//     //     log1("kkkkkkkkk".to_string());
//     // };
    
//     // let f: wasm_bindgen::closure::Closure<dyn Fn(wasm_bindgen::JsValue)> = wasm_bindgen::closure::Closure::new(f);

//     // // let fu = wasm_bindgen::JsValue::from_str("");
    


//     // let f2 = f.as_ref().unchecked_ref();
//     // //let fu = js_sys::Function::from(f2);

//     // //let ff = f.into_js_value();
//     // //let f = std::rc::Rc::new(f);
//     // //let ff = std::boxed::Box::leak(f);
//     // //let p = std::sync::Arc::new(555);
    
//     // let o = wasm_bindgen::JsValue::from(pp);
//     // worker.post_message(&o).unwrap();

//     // worker.set_onmessageerror(Some(f2));
//     // //worker.set_onmessage(value);
//     // //use std::thread;

//     // // thread::spawn(move || {
//     // //     log1("theres no way this works".to_string());
//     // // });

//     // //executor.run();

//     // // let w = web_sys::window().unwrap();
//     // // let d = w.document().unwrap();
//     // // let e = d.get_element_by_id("69").unwrap().dyn_ref::<web_sys::HtmlElement>().unwrap();
//     // // e.set_onclick(value);

//     // tracing::trace!("wow: {url}");
//     // f.forget();
//     //leptos::mount_to_body(App)
// }

// impl WASMTracingLayer {
//     pub fn new(config: WASMTracingConfig,) -> Self {
//         Self {
//             spans: std::sync::Arc::new(std::sync::Mutex::new(indexmap::IndexMap::new())),
//             config,
//         }
//     }
// }

// impl<S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>>
//     tracing_subscriber::Layer<S> for WASMTracingLayer
// {
//     fn on_event(&self, event: &tracing::Event<'_>, ctx: tracing_subscriber::layer::Context<'_, S>) {
//         let mut spans_combined = String::new();
//         {
//             let spans = &mut *self.spans.lock().unwrap();
//             let len = spans.len();
//             if len > 0 {
//                 spans_combined.push_str(" ");
//             }
//             for (i, span) in spans.values().enumerate() {
//                 spans_combined.push_str(span);
//                 if i + 1 != len {
//                     spans_combined.push_str(", ")
//                 }
//             }
//             //output.push_str(": ");
//         }
//         let mut value = String::new();
//         let writer = tracing_subscriber::fmt::format::Writer::new(&mut value);
//         let mut visitor = tracing_subscriber::fmt::format::PrettyVisitor::new(writer, true);
//         event.record(&mut visitor);

//         let meta = event.metadata();
//         let level = meta.level();
//         let target = if self.config.target { format!(" {}", meta.target()) } else { "".to_string() };
//         let origin = if self.config.line { meta
//             .file()
//             .and_then(|file| meta.line().map(|ln| format!(" {}:{}", file, ln)))
//             .unwrap_or_default() } else { String::new() };
//         //let thread_suffix = thread_display_suffix();

//         log5(
//             format!("%c{level}%c{spans_combined}%c{target}{origin}%c: {value}"),
//             match *level {
//                 tracing::Level::TRACE => "color: dodgerblue; background: #444",
//                 tracing::Level::DEBUG => "color: lawngreen; background: #444",
//                 tracing::Level::INFO => "color: whitesmoke; background: #444",
//                 tracing::Level::WARN => "color: orange; background: #444",
//                 tracing::Level::ERROR => "color: red; background: #444",
//             },
//             "color: inherit; font-weight: bold",
//             "color: gray; font-style: italic",
//             "color: inherit",
//         );
//     }

//     fn on_new_span(
//         &self,
//         attrs: &tracing::span::Attributes<'_>,
//         id: &tracing::span::Id,
//         ctx: tracing_subscriber::layer::Context<'_, S>,
//     ) {
//         let meta = attrs.metadata();
//         let name = meta.name();
//         let target = meta.target();
//         if !target.contains("artcord") {
//             return;
//         }

//         let mut body = String::new();
//         let writer = tracing_subscriber::fmt::format::Writer::new(&mut body);
//         let mut visitor = tracing_subscriber::fmt::format::PrettyVisitor::new(writer, true);
//         attrs.record(&mut visitor);

//         let has_name = !name.is_empty() && name != "{}";
//         let has_body = !body.is_empty() && body != "{}";

//         let output = match (has_name, has_body) {
//             (true, false) => name.to_string(),
//             (true, true) => {
//                 format!("{} = {}", name, body)
//             }
//             (false, false) => String::from("{}"),
//             (false, true) => body.to_string(),
//         };

//         let spans = &mut *self.spans.lock().unwrap();
//         spans.insert(id.clone(), output);
//     }

//     fn on_exit(&self, id: &tracing::span::Id, _ctx: tracing_subscriber::layer::Context<'_, S>) {
//         let spans = &mut *self.spans.lock().unwrap();
//         spans.swap_remove(id);
//     }

//     fn on_close(&self, id: tracing::span::Id, _ctx: tracing_subscriber::layer::Context<'_, S>) {
//         let spans = &mut *self.spans.lock().unwrap();
//         spans.swap_remove(&id);
//     }
// }

// #[wasm_bindgen]
// extern "C" {
//     #[wasm_bindgen(js_namespace = performance)]
//     fn mark(a: &str);
//     #[wasm_bindgen(catch, js_namespace = performance)]
//     fn measure(name: String, startMark: String) -> Result<(), wasm_bindgen::JsValue>;
//     #[wasm_bindgen(js_namespace = console, js_name = log)]
//     fn log1(message: String);
//     #[wasm_bindgen(js_namespace = console, js_name = log)]
//     fn log2(message1: &str, message2: &str);
//     #[wasm_bindgen(js_namespace = console, js_name = log)]
//     fn log3(message1: &str, message2: &str, message3: &str);
//     #[wasm_bindgen(js_namespace = console, js_name = log)]
//     fn log4(message1: String, message2: &str, message3: &str, message4: &str);
//     #[wasm_bindgen(js_namespace = console, js_name = log)]
//     fn log5(message1: String, message2: &str, message3: &str, message4: &str, message5: &str);
// }

// #[cfg(not(feature = "mark-with-rayon-thread-index"))]
// #[inline]
// fn thread_display_suffix() -> &'static str {
//     ""
// }
// #[cfg(feature = "mark-with-rayon-thread-index")]
// fn thread_display_suffix() -> String {
//     let mut message = " #".to_string();
//     match rayon::current_thread_index() {
//         Some(idx) => message.push_str(&format!("{}", idx)),
//         None => message.push_str("main"),
//     }
//     message
// }
