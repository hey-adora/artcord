use crate::app::utils::{GlobalState, ScrollSection};
use leptos::*;

#[component]
pub fn Navbar() -> impl IntoView {
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let section = global_state.section;

    view! {
        <nav  id="thenav" class=move || { format!("sticky  text-low-purple top-0 z-50 px-6 flex items-center justify-between gap-2 transition-all duration-500    {}", if section() == ScrollSection::HomeTop || section() == ScrollSection::GalleryTop { "bg-transparent"  } else { "bg-gradient-to-r from-mid-purple to-dark-purple" } ) }>
            <div class="flex items-center gap-6">
                <a href="/" class="  font-bold text-[2rem] ">{  move || format!("ArtCord") }</a>
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
