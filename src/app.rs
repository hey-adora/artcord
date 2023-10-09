use js_sys::Math::log;
use leptos::html::{AnyElement, Body, ElementDescriptor, Section, ToHtmlElement};
use leptos::leptos_dom::HydrationKey;
use leptos::logging::log;
use leptos::{html::Nav, *};
use leptos_meta::*;
use leptos_router::*;
use leptos_use::use_window_scroll;
use std::ops::Deref;
use wasm_bindgen::JsValue;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ScrollSection {
    HomeTop,
    Home,
    About,
    GalleryTop,
    Gallery,
}

#[derive(Copy, Clone, Debug)]
struct GlobalState {
    section: RwSignal<ScrollSection>,
}

impl GlobalState {
    pub fn new() -> Self {
        Self {
            section: create_rw_signal(ScrollSection::HomeTop),
        }
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    provide_context(GlobalState::new());

    view! {
        <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
        <Stylesheet id="leptos" href="/pkg/leptos_start.css"/>
        <Title text="Welcome to Leptos"/>
        <Body class="text-low-purple pt-4 bg-gradient-to-br from-mid-purple to-dark-purple" />

        <Router>
            <Navbar/>
            <main id="home" on:scroll=|_|{ logging::log!("SCROLLED!"); }  class=" grid grid-rows-[auto_1fr] pt-4 gap-6       ">
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
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let section = global_state.section;

    view! {
        <nav  id="thenav" class=move || { format!("sticky  text-low-purple top-0 z-50 px-6 flex items-center justify-between gap-2 transition-all duration-500    {}", if section() == ScrollSection::HomeTop || section() == ScrollSection::GalleryTop { "bg-transparent"  } else { "bg-gradient-to-r from-mid-purple to-dark-purple" } ) }>
            <div class="flex items-center gap-6">
                <a href="/" class="  font-bold text-[2rem] ">{  move || format!("ArtCord {:?}", section()) }</a>
                <ul class="hidden sm:flex gap-2 text-[1rem] text-center">
                    <li><a href="/#home" class=move || { format!( " w-[3.5rem] cursor-pointer border-b-[0.30rem] transition duration-300 font-bold {} ", if section() == ScrollSection::HomeTop || section() == ScrollSection::Home  { "border-low-purple font-bold" } else { "border-transparent hover:border-low-purple/40 text-low-purple/60 hover:text-low-purple " } ) } >"Home"</a></li>
                    <li><a href="/#about" class=move || { format!( " w-[3.5rem] cursor-pointer border-b-[0.30rem] transition duration-300 font-bold {} ", if section() == ScrollSection::About { "border-low-purple font-bold" } else { "border-transparent hover:border-low-purple/40 text-low-purple/60 hover:text-low-purple " } ) } >"About"</a></li>
                    <li><a href="/gallery" class=move || { format!( " w-[3.5rem] cursor-pointer border-b-[0.30rem] transition duration-300 font-bold {} ", if section() == ScrollSection::Gallery || section() == ScrollSection::GalleryTop { "border-low-purple font-bold" } else { "border-transparent hover:border-low-purple/40 text-low-purple/60 hover:text-low-purple " } ) } >"Gallery"</a></li>
                </ul>
            </div>
            <a target="_blank" href="https://discord.gg/habmw7Ehga">
                <div class="hidden  sm:flex gap-2 items-center text-[1rem] font-black bg-half-purple border-[0.30rem] border-low-purple rounded-3xl px-4 py-[0.15rem] hover:bg-dark-purple transition-colors duration-300 " >
                    <img src="/assets/discord.svg"/>
                    "Join"
                </div>
                <img class="  cursor-pointer block sm:hidden " src="assets/burger.svg" alt=""/>
            </a>
        </nav>
    }
}

fn get_window_path() -> String {
    let location = window().location();
    let path = location.pathname();
    let hash = location.hash();
    if let (Ok(path), Ok(hash)) = (path, hash) {
        format!("{}{}", path, hash)
    } else {
        String::from("/")
    }
}

pub trait OffsetTop {
    fn y(&self) -> i32;
}

impl OffsetTop for leptos::HtmlElement<Section> {
    fn y(&self) -> i32 {
        self.offset_top()
    }
}

fn get_element_y<T: ElementDescriptor + Clone>(element: NodeRef<T>) -> i32
where
    leptos::HtmlElement<T>: OffsetTop,
{
    let test = element.get();
    let a = test.unwrap().y();

    let mut section_y: i32 = 0;
    if let Some(section) = element.get() {
        let a = section.y();
        section_y = a;
    }
    section_y
}

fn silent_navigate(state: &str, unused: &str, url: &str) {
    let a = window().history();
    if let Ok(a) = a {
        a.push_state_with_url(&JsValue::from(state), unused, Some(url))
            .unwrap();
    }
}

#[component]
fn GalleryPage() -> impl IntoView {
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let section = global_state.section;

    let gallery_section = create_node_ref::<Section>();
    let scroll_items = [ScrollDetect::new(
        ScrollSection::Gallery,
        gallery_section,
        0,
        "/gallery",
    )];

    create_effect(move |_| {
        ScrollDetect::calc_section(section, ScrollSection::GalleryTop, &scroll_items);
        // if section.get() != ScrollSection::Gallery {
        //     section.set(ScrollSection::Gallery);
        // }
    });

    view! {
        <section _ref=gallery_section style=move|| format!("min-height: calc(100vh - 100px)")>
            <h1>GALLERY</h1>
        </section>
    }
}

pub struct ScrollDetect<'a, T: ElementDescriptor + 'static> {
    pub node_ref: NodeRef<T>,
    pub offset: i32,
    pub path: &'a str,
    pub id: ScrollSection,
}

impl<'a, T: ElementDescriptor> ScrollDetect<'a, T> {
    pub fn new(id: ScrollSection, node_ref: NodeRef<T>, offset: i32, path: &'a str) -> Self {
        Self {
            node_ref,
            offset,
            path,
            id,
        }
    }
}

impl<'a, T: ElementDescriptor + Clone> ScrollDetect<'a, T> {
    pub fn calc_section(
        section: RwSignal<ScrollSection>,
        default: ScrollSection,
        scroll_items: &[ScrollDetect<'a, T>],
    ) -> ()
    where
        leptos::HtmlElement<T>: OffsetTop,
    {
        let current_section: ScrollSection = section.get();

        for scroll_item in scroll_items {
            let (x, y) = use_window_scroll();
            let element_y = get_element_y(scroll_item.node_ref) - scroll_item.offset;
            //log!("{:?} : {} <= {}", scroll_item.id, element_y, y());
            if element_y as f64 <= y() {
                //log!("{:?} == {:?}", scroll_item.id, current_section);
                if scroll_item.id != current_section {
                    //log!("SET FROM {:?} TO {:?}", current_section, scroll_item.id);
                    section.set(scroll_item.id);
                    silent_navigate(scroll_item.path, "", scroll_item.path);
                }
                return ();
            }
        }

        if current_section != default {
            //log!("BOOM {:?} == {:?}", ScrollSection::None, current_section);
            section.set(default);
        }
    }
}

#[component]
fn HomePage() -> impl IntoView {
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let section = global_state.section;

    let home_section = create_node_ref::<Section>();
    let about_section = create_node_ref::<Section>();

    let scroll_items = [
        ScrollDetect::new(ScrollSection::About, about_section, 70, "/#about"),
        ScrollDetect::new(ScrollSection::Home, home_section, 70, "/#home"),
    ];

    create_effect(move |_| {
        ScrollDetect::calc_section(section, ScrollSection::HomeTop, &scroll_items);
    });

    view! {
        <section _ref=home_section class="px-6 py-6 line-bg grid grid-rows-[1fr_1fr_0.3fr] md:grid-rows-[1fr] md:grid-cols-[1fr_1fr] place-items-center  overflow-hidden " style=move|| format!("min-height: calc(100vh - 100px)")>
                <div class=" bg-the-star bg-center bg-contain bg-no-repeat h-full w-full grid place-items-center  ">
                    <div class="text-center flex flex-col">
                        <h1 class="text-[4rem] font-bold">"ArtCord"</h1>
                        <h2 class="text-[2rem]">"Discord Art Server"</h2>
                        <div class="flex gap-8 mt-4 items-center justify-center">
                            <a class=" text-[1rem] cursor-pointer border-b-[0.30rem] border-low-purple font-bold whitespace-nowrap">"Read More"</a>
                            <a target="_blank" href="https://discord.gg/habmw7Ehga" class="flex gap-2 items-center text-[1rem] font-black bg-half-purple border-[0.30rem] border-low-purple rounded-3xl px-4 py-[0.15rem] hover:bg-dark-purple transition-colors duration-300 " >
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
                        <a class=" shadow-glowy text-[1rem] font-black bg-half-purple border-[0.30rem] border-low-purple rounded-3xl px-4 py-[0.15rem] hover:bg-dark-purple transition-colors duration-300 " >
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
            </section>
            <section _ref=about_section id="about" class=" line-bg px-6 py-6 flex flex-col md:grid md:grid-rows-[1fr_1fr_1fr_auto] md:grid-cols-[1fr_1fr] gap-0" style=move|| format!("min-height: calc(100vh - 50px)")>
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
    }
}

#[component]
fn NotFound() -> impl IntoView {
    #[cfg(feature = "ssr")]
    {
        let resp = expect_context::<leptos_actix::ResponseOptions>();
        resp.set_status(actix_web::http::StatusCode::NOT_FOUND);
    }

    view! {
        <h1>"Not Found"</h1>
    }
}
