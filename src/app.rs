use leptos::html::{Body, Section};
use leptos::logging::log;
use leptos::{html::Nav, *};
use leptos_meta::*;
use leptos_router::*;
use leptos_use::use_window_scroll;

#[derive(Clone, PartialEq, Debug)]
enum ScrollSection {
    None,
    Home,
    About,
}

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
    provide_meta_context();
    provide_context(GlobalState::new());

    view! {
        <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
        <Stylesheet id="leptos" href="/pkg/leptos_start.css"/>
        <Title text="Welcome to Leptos"/>
        <Body class="text-low-purple bg-gradient-to-br from-mid-purple to-dark-purple" />
        <Router>
            <Navbar/>
            <main id="home" on:scroll=|_|{ logging::log!("SCROLLED!"); }  class=" grid grid-rows-[auto_1fr] pt-6 gap-6       ">
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
    let (count, set_count) = create_signal(0);
    let (scroll_section, set_scroll_section) = create_signal(ScrollSection::None);

    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let home_section = global_state.home_section.get();
    let about_section = global_state.about_section.get();

    let navigate = leptos_router::use_navigate();

    create_effect(move |_| {
        let (x, y) = use_window_scroll();
        let y = y();
        let current_section = scroll_section();

        let home_section_y: i32 = get_offset(home_section) - 70;
        let about_section_y: i32 = get_offset(about_section) - 70;

        let new_section = match y {
            n if n > about_section_y as f64 => ScrollSection::About,
            n if n > home_section_y as f64 => ScrollSection::Home,
            _ => ScrollSection::None,
        };

        log!("{}", y);

        if new_section != current_section {
            set_scroll_section(new_section);
        }
    });

    let btn_click = move |_| {
        log!("wowowow");
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

#[component]
fn GalleryPage() -> impl IntoView {
    view! {
        <h1>GALLERY</h1>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let home_section = global_state.home_section.get();
    let about_section = global_state.about_section.get();

    view! {
        <section _ref=home_section class="px-6 py-6 line-bg grid grid-rows-[1fr_1fr_0.3fr] md:grid-rows-[1fr] md:grid-cols-[1fr_1fr] place-items-center  overflow-hidden " style=move|| format!("min-height: calc(100vh - 100px)")>
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
