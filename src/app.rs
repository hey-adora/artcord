use leptos::html::Section;
use leptos::logging::log;
use leptos::{html::Nav, *};
use leptos_meta::*;
use leptos_router::*;
use leptos_use::use_window_scroll;

//use wasm_bindgen::prelude::*;
#[derive(Clone, PartialEq, Debug)]
enum ScrollSection {
    None,
    Home,
    About,
}

// fn boom(n: i32) -> Result::<i32, &'static str> {
//     match n {
//         5 => Ok(100),
//         _ => Err("error OwO")
//     }
// }
//
// fn maien() {
//     let output = boom(4);
//     match output {
//         Ok(n) => println!("works! {}", n),
//         Err(e) => panic!("BOOM: {}", e)
//     }
// }

#[derive(Copy, Clone, Debug)]
struct GlobalState {
    home_section: RwSignal<NodeRef<Section>>,
    about_section: RwSignal<NodeRef<Section>>,
}

impl GlobalState {
    pub fn new() -> Self {
        Self {
            home_section: create_rw_signal(create_node_ref::<html::Section>()),
            about_section: create_rw_signal(create_node_ref::<html::Section>()),
        }
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    //let navi = create_node_ref::<html::Main>();
    // let y3 = js_sys::Function::new_no_args("console.log(\"test\");");
    // let a = create_effect(move |prev_value| {
    //     let node = navi.get();
    //     if let Some(node) = node {
    //         logging::log!("loaded!");
    //         node.add_event_listener_with_callback("scroll", &y3);
    //     }
    // });

    //provide_context(GlobalState::new());

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <meta name="viewport" content="width=device-width, initial-scale=1.0"/>

        <Stylesheet id="leptos" href="/pkg/leptos_start.css"/>
        // <link rel="preconnect" href="https://fonts.googleapis.com"/>
        // <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin/>
        // <link href="https://fonts.googleapis.com/css2?family=Libre+Barcode+128+Text&display=swap" rel="stylesheet"/>


        // sets the document title
        <Title text="Welcome to Leptos"/>



        // content for this welcome page
        <Router>
            <Navbar/>
            <main id="home" on:scroll=|_|{ logging::log!("SCROLLED!"); }  class=" grid grid-rows-[auto_1fr] pt-6 gap-6   text-low-purple bg-gradient-to-br from-mid-purple to-dark-purple    ">
                <Routes>
                    <Route path="" view=HomePage/>
                    <Route path="/gallery" view=GalleryPage/>
                    <Route path="/*any" view=NotFound/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn Navbar() -> impl IntoView {
    // Creates a reactive value to update the button
    let (count, set_count) = create_signal(0);
    //let (nav_bg, set_nav_bg) = create_signal(false);
    let (scroll_section, set_scroll_section) = create_signal(ScrollSection::None);

    //let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    //let home_section = global_state.home_section.get();
    //let about_section = global_state.about_section.get(); create_node_ref::<html::Section>()
    let home_section = create_node_ref::<html::Section>();
    let about_section = create_node_ref::<html::Section>();

    let navigate = leptos_router::use_navigate();

    //let (x, y) = use_window_scroll();
    //window().scroll
    //let a = use_scroll();
    let (x, y) = use_window_scroll();
    create_effect(move |_| {
        let y = y();
        let current_section = scroll_section();

        let home_section_y: i32 = get_offset(home_section) - 70;
        let about_section_y: i32 = get_offset(about_section) - 70;

        let new_section = match y {
            n if n > about_section_y as f64 => ScrollSection::About,
            n if n > home_section_y as f64 => ScrollSection::Home,
            _ => ScrollSection::None,
        };

        if new_section != current_section {
            // log!("{:?}", new_section);
            // match new_section {
            //     ScrollSection::Home => silent_navigate("home", "", "#home"),
            //     ScrollSection::About => silent_navigate("about", "", "#about"),
            //     _ => (),
            // };

            set_scroll_section(new_section);
        }

        // logging::log!("{}, {}, {}", y, home_section_y, about_section_y);

        // if y > 50f64 {
        //     if nav_bg == false {
        //         set_nav_bg(true);
        //     }
        // } else if nav_bg == true {
        //     set_nav_bg(false);
        // }
        //logging::log!("{}", y());
    });

    let btn_click = move |_| {
        log!("wowowow");
        // let a = window().history();
        // if let Ok(a) = a {
        //     log!("wowowow2");
        //     a.push_state_with_url(&JsValue::from("google"), "google2", Some("wow"))
        //         .unwrap();
        // }

        //..location().replace("https://google.com");
    };

    view! {
        <nav  id="thenav" class=move || { format!("sticky text-low-purple top-0 z-50 px-6 flex items-center justify-between gap-2  {}", if scroll_section() != ScrollSection::None { "bg-gradient-to-r from-mid-purple to-dark-purple" } else { "" } ) }>
            <div class="flex items-center gap-6">
                <h3 class="  font-bold text-[2rem] ">"ArtCord"</h3>
                <ul class="hidden sm:flex gap-2 text-[1rem] text-center">
                    <li><a class=move || { format!( " w-[3.5rem] cursor-pointer border-b-[0.30rem] transition duration-300 font-bold {} ", if scroll_section() == ScrollSection::None || scroll_section() == ScrollSection::Home { "border-low-purple font-bold" } else { "border-transparent hover:border-low-purple/40 text-low-purple/60 hover:text-low-purple " } ) } href="#home">"Home"</a></li>
                    <li><a class=move || { format!( " w-[3.5rem] cursor-pointer border-b-[0.30rem] transition duration-300 font-bold {} ", if scroll_section() == ScrollSection::About { "border-low-purple font-bold" } else { "border-transparent hover:border-low-purple/40 text-low-purple/60 hover:text-low-purple " } ) } href="#about">"About"</a></li>
                    <li><a class=move || { format!( " w-[3.5rem] cursor-pointer border-b-[0.30rem] transition duration-300 font-bold {} ", if false { "border-low-purple font-bold" } else { "border-transparent hover:border-low-purple/40 text-low-purple/60 hover:text-low-purple " } ) } href="/gallery">"Gallery"</a></li>
                </ul>
            </div>
            <button on:click=btn_click>
                <div class="hidden sm:flex gap-2 items-center text-[1rem] font-black bg-half-purple border-[0.30rem] border-low-purple rounded-3xl px-4 py-[0.15rem] hover:bg-dark-purple transition-colors duration-300 " href="">
                    <img src="/assets/discord.svg"/>
                    "Join"
                </div>
                <img class="cursor-pointer block sm:hidden " src="assets/burger.svg" alt=""/>
            </button>
        </nav>
    }
}

fn get_offset(element: NodeRef<Section>) -> i32 {
    let mut section_y: i32 = 0;
    let home_section = element.get();
    if let Some(section) = home_section {
        section_y = section.offset_top();
    }
    section_y
}

// fn silent_navigate(state: &str, unused: &str, url: &str) {
//     let a = window().history();
//     if let Ok(a) = a {
//         a.push_state_with_url(&JsValue::from(state), unused, Some(url))
//             .unwrap();
//     }
// }

#[component]
fn GalleryPage() -> impl IntoView {
    view! {
        <h1>GALLERY</h1>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // home_section: NodeRef<Section>, about_section: NodeRef<Section>

    // let navi = create_node_ref::<html::Main>();
    //
    // //let on_click = move |_| set_count.update(|count| *count += 1);
    // //let navi = document().get_element_by_id("thenav");
    // //let ff = js_sys::F
    // //let y3 = js_sys::Function::from(JsValue::from("console.log('test')"));
    // let y3 = js_sys::Function::new_no_args("console.log(\"test\");");
    // //let y: Option<::js_sys::Function> = ;
    // //unsafe { js_sys:: }
    // let a = create_effect(move |prev_value| {
    //     let node = navi.get();
    //     if let Some(node) = node {
    //         logging::log!("loaded!");
    //         node.add_event_listener_with_callback("scroll", &y3);
    //         //node.set_onscroll(Some(&y3));
    //         //logging::log!("{:?}", );
    //     }
    // });

    // let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    // let home_section = global_state.home_section.get();
    // let about_section = global_state.about_section.get();

    view! {

        // <div class="  " >

        // </div>

        <section  class="px-6 py-6 line-bg grid grid-rows-[1fr_1fr_0.3fr] md:grid-rows-[1fr] md:grid-cols-[1fr_1fr] place-items-center  overflow-hidden " style=move|| format!("min-height: calc(100vh - 100px)")>
                <div class=" bg-the-star bg-center bg-contain bg-no-repeat h-full w-full grid place-items-center  ">
                    <div class="text-center flex flex-col">
                        <h1 class="text-[4rem] font-bold">"ArtCord"</h1>
                        <h2 class="text-[2rem]">"Discord Art Server"</h2>
                        <div class="flex gap-8 mt-4 items-center justify-center">
                            <a class=" text-[1rem] cursor-pointer border-b-[0.30rem] border-low-purple font-bold whitespace-nowrap">"Read More"</a>
                            <a class="flex gap-2 items-center text-[1rem] font-black bg-half-purple border-[0.30rem] border-low-purple rounded-3xl px-4 py-[0.15rem] hover:bg-dark-purple transition-colors duration-300 " href="">
                                <img src="/assets/discord.svg"/>
                                "Join"
                            </a>
                        </div>
                    </div>
                </div>
                <div class="flex flex-col  justify-center gap-6 sm:gap-12">
                    <div class="flex justify-center relative">
                        <div class="z-10 w-[32vw] h-[55vw] lg:max-w-[15rem] lg:max-h-[25rem] max-w-[10rem] max-h-[20rem] bg-center bg-cover absolute rotate-[15deg] translate-x-[60%]" style="background-image: url('/assets/1.jpg')" ></div>
                        <div class="z-20 w-[32vw] h-[55vw] lg:max-w-[15rem] lg:max-h-[25rem] max-w-[10rem] max-h-[20rem] bg-center bg-cover" style="background-image: url('/assets/2.jpg')" ></div>
                        <div class="z-10 w-[32vw] h-[55vw] lg:max-w-[15rem] lg:max-h-[25rem] max-w-[10rem] max-h-[20rem] bg-center bg-cover absolute -rotate-[15deg] -translate-x-[60%]" style="background-image: url('/assets/3.jpg')" ></div>
                    </div>
                    <div class="flex justify-center">
                        <a class=" shadow-glowy text-[1rem] font-black bg-half-purple border-[0.30rem] border-low-purple rounded-3xl px-4 py-[0.15rem] hover:bg-dark-purple transition-colors duration-300 " href="">
                            "View Gallery"
                        </a>
                    </div>

                </div>
                <div class=" md:col-span-2 grid place-items-center mt-auto text-center font-bold ">
                    <div class="flex flex-col gap-2 justify-center">
                        "About"
                        <img class="h-[2rem]" src="/assets/triangle.svg"/>
                    </div>
                </div>
                // <div class="absolute w-[0.25rem]  h-full bg-low-purple/40"></div> grid-rows-[auto_auto_auto] grid-cols-[auto_auto]
            </section>
            <section  id="about" class=" line-bg px-6 py-6 flex flex-col md:grid md:grid-rows-[1fr_1fr_1fr_auto] md:grid-cols-[1fr_1fr] gap-0" style=move|| format!("min-height: calc(100vh - 50px)")>
                <div>
                    <h4 class="text-[3rem] font-bold" >"About Us"</h4>
                    <p class="text-[1.5rem]" >"We're a community of artists who love to create, share, and learn. We're open to all types of art, from traditional to digital, and we're always looking for new members!"</p>
                </div>
                <img class="mx-auto hidden md:block" src="assets/circle.svg" alt=""/>
                <img class="mx-auto hidden md:block" src="assets/rectangle.svg" alt=""/>
                <div>
                    <h4 class="text-[3rem] font-bold" >"You Can"</h4>
                    <p class="text-[1.5rem]">
                        "Share your art with other artists." <br/>
                        "Get feedback on your art." <br/>
                        "Find inspiration from other artists." <br/>
                        "Collaborate with other artists on projects."
                    </p>
                </div>
                <div>
                    <h4 class="text-[3rem] font-bold" >"We Have"</h4>
                    <p class="text-[1.5rem] flex flex-col gap-4">
                        <div class="flex gap-4">
                            <span>"Challenges"</span>
                            <span>"Art Arena"</span>
                        </div>
                        <div class="flex gap-4">
                            <span>"Reaction Roles"</span>
                            <span>"24x7 Support"</span>
                        </div>
                    </p>
                </div>
                <img class="mx-auto hidden md:block" src="assets/triangle2.svg" alt=""/>
                <div class=" mt-auto text-[3rem] col-span-2 text-center font-barcode">"Copyrighted  2023"</div>
            </section>
        // <img class="" src="/assets/bg.svg" alt=""/>
    }
}

/// 404 - Not Found
#[component]
fn NotFound() -> impl IntoView {
    // set an HTTP status code 404
    // this is feature gated because it can only be done during
    // initial server-side rendering
    // if you navigate to the 404 page subsequently, the status
    // code will not be set because there is not a new HTTP request
    // to the server
    #[cfg(feature = "ssr")]
    {
        // this can be done inline because it's synchronous
        // if it were async, we'd use a server function
        let resp = expect_context::<leptos_actix::ResponseOptions>();
        resp.set_status(actix_web::http::StatusCode::NOT_FOUND);
    }

    view! {
        <h1>"Not Found"</h1>
    }
}
