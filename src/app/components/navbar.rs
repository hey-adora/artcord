use leptos::logging::log;
use leptos::*;
use leptos_router::use_location;
use leptos_use::{use_interval_fn, use_window_scroll};
use web_sys::MouseEvent;

use crate::app::utils::{get_window_path, GlobalState, LoadingNotFound, ScrollSection};

pub fn shrink_nav(nav_tran: RwSignal<bool>, y: u32) {
    if y > 100 {
        if nav_tran.with(|&s| s) {
            //log!("FALSE: {}", y());
            nav_tran.set(false);
        }
    } else {
        if nav_tran.with(|&s| !s) {
            //log!("TRUE: {}", y());
            nav_tran.set(true);
        }
    }
}

#[component]
pub fn Navbar() -> impl IntoView {
    //let a = use_context::<TestContext>().expect("Failed to provide test context.");
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let section = global_state.section;
    let nav_tran = global_state.nav_tran;

    let on_nav_click = move |_: MouseEvent| {
        global_state
            .nav_open
            .update(|open: &mut bool| *open = !*open);
    };
    let l = use_location();

    create_effect(move |_| {
        let section: ScrollSection = match format!("{}{}", l.pathname.get(), l.hash.get()).as_str()
        {
            "/gallery" => ScrollSection::Gallery,
            "/#about" => ScrollSection::About,
            s if s.contains("/user/") => ScrollSection::UserProfile,
            _ => ScrollSection::Home,
        };
        if section != global_state.section.get() {
            global_state.section.set(section);
        }
    });

    let title = move || {
        let mut output = String::from("ArtCord");
        // log!("{:?} {:?}", global_state.section.get(), global_state.page_profile.gallery_loaded.get());
        if global_state.section.get() == ScrollSection::UserProfile
            && global_state.page_profile.gallery_loaded.get() == LoadingNotFound::Loaded
        {
            if let Some(user) = global_state.page_profile.user.get() {
                let pfp_url = format!("/assets/gallery/pfp_{}.webp", user.id.clone());
                //  log!("wow1");
                return view! {
                    <div class="flex gap-4">
                        <img class="border border-low-purple rounded-full bg-mid-purple h-[45px] " src=pfp_url/>
                        <p class="text-ellipsis overflow-hidden"> {user.name} </p>
                    </div>
                };
            }
        }
        // log!("wow2");
        view! {
            <div>
                <p class="text-ellipsis overflow-hidden">{output}</p>
           </div>
        }
    };

    view! {
        <nav  id="thenav" class=move || { format!("fixed backdrop-blur text-low-purple w-full top-0 z-[100] px-6 2xl:px-[6rem] desktop:px-[16rem]  flex   gap-2  duration-500  bg-gradient-to-r from-dark-night2/75 to-light-flower/10 supports-backdrop-blur:from-dark-night2/95 supports-backdrop-blur:to-light-flower/95 {} {}", if nav_tran.get() == true && global_state.nav_open.get() != true { " py-2 "  } else { "" }, if global_state.nav_open.get() == true { "w-[100vw] h-[100vh]" } else { "items-center justify-between transition-all" } ) }>
            <div class=move || format!("flex gap-6 items-center {}", if global_state.nav_open.get() == true { " flex-col w-full " } else { " " })>
                {
                    move || {
                        if global_state.nav_open.get() == true {
                            view! {
                                <div class="w-full flex justify-between font-bold text-[2rem]" >
                                    <div>{ title() }</div>
                                    <button on:click=on_nav_click>X</button>
                                </div>
                            }
                        } else {
                            view! {
                                <div>
                                    <a href="/" class="  font-bold text-[2rem] ">{  title()  }</a>
                                </div>
                            }
                        }
                    }
                }
                <ul class=move || format!(" gap-2  text-center {}", if global_state.nav_open.get() == true { " flex-col text-[2rem] flex flex-col h-full" } else { "hidden sm:flex text-[1rem] " })>
                    <li><a on:click=move |_| { global_state.nav_open.set(false); } href="/#home" class=move || { format!( " w-[3.5rem] cursor-pointer border-b-[0.30rem] transition duration-300 font-bold {} ", if section.get() == ScrollSection::Home  { "border-low-purple font-bold" } else { "border-transparent hover:border-low-purple/40 text-low-purple/60 hover:text-low-purple " } ) } >"Home"</a></li>
                    <li><a on:click=move |_| { global_state.nav_open.set(false); } href="/#about" class=move || { format!( " w-[3.5rem] cursor-pointer border-b-[0.30rem] transition duration-300 font-bold {} ", if section.get() == ScrollSection::About { "border-low-purple font-bold" } else { "border-transparent hover:border-low-purple/40 text-low-purple/60 hover:text-low-purple " } ) } >"About"</a></li>
                    <li><a on:click=move |_| { global_state.nav_open.set(false); } href="/gallery" class=move || { format!( " w-[3.5rem] cursor-pointer border-b-[0.30rem] transition duration-300 font-bold {} ", if section.get() == ScrollSection::Gallery || section.get() == ScrollSection::UserProfile { "border-low-purple font-bold" } else { "border-transparent hover:border-low-purple/40 text-low-purple/60 hover:text-low-purple " } ) } >"Gallery"</a></li>
                </ul>
            </div>
            {
                move || {
                    if global_state.nav_open.get() == false {
                       Some(
                        view! {
                            <div class=move || format!("{}", if global_state.nav_open.get() == true { " hidden " } else { " " }) >
                                // { move || global_state.nav_open.get() }
                                <a target="_blank" href="https://discord.gg/artfriends" class="hidden h-12 sm:flex gap-2 items-center text-[1rem] font-black bg-gradient-to-br from-first-one to-second-one hover:to-dark-purple border-[0.30rem] border-low-purple rounded-3xl px-4 py-[0.15rem] transition-colors duration-300 " >
                                    <img class="h-8" src="/assets/discord.svg"/>
                                    "Join"
                                </a>
                                <button class="block sm:hidden h-[48px]" on:click=on_nav_click >
                                    <img class="    " src="/assets/burger.svg" alt=""/>
                                </button>
                            </div>
                           }
                       )
                    } else {
                        None
                    }
                }
            }

        </nav>
    }
}
