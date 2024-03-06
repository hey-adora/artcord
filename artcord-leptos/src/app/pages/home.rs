use crate::app::components::navbar::shrink_nav;
use crate::app::components::navbar::Navbar;

use artcord_leptos_web_sockets::Runtime;
use leptos::html::Main;
use leptos::logging::log;
use leptos::*;
use web_sys::Event;
use crate::app::global_state::GlobalState;

#[component]
pub fn HomePage() -> impl IntoView {
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let scroll_el = create_node_ref::<Main>();
    let nav_tran = global_state.nav_tran;

    let on_scroll = move |_: Event| {
        let Some(scroll_el) = scroll_el.get() else {
            return;
        };
        let y = scroll_el.scroll_top();
        shrink_nav(nav_tran, y as u32);
    };

    view! {
        <main  on:scroll=on_scroll _ref=scroll_el class="flex flex-col ">
           
            <Navbar/>
            <section id="home" class=" px-6 py-6 2xl:px-[6rem] desktop:px-[16rem]  grid grid-rows-[auto_auto_1fr] grid-cols-[1fr]  min-h-[100svh] " >
                <div class="h-[4rem] md:h-[6rem]"></div>
                // <button on:click=test_click>"CLICK ME"</button>
                <div class="flex flex-col gap-[2rem] md:gap-[4rem]  max-w-min ">
                    <div class="text-left flex flex-col justify-start">
                        <h2 class="text-[2rem] font-bold whitespace-nowrap ">"Discord Art Server2"</h2>
                        <p class="text-[1.3rem]">"Where creativity knows no bounds and artistic expression finds its true home! "</p>
                        <div class="flex gap-8 mt-4 items-center ">
                            <a target="_blank" href="https://discord.gg/habmw7Ehga" class="flex gap-2 items-center text-[1rem] h-12 font-black bg-gradient-to-br from-first-one to-second-one hover:to-dark-purple border-[0.30rem] border-low-purple rounded-3xl px-4 py-[0.15rem]  transition-colors duration-300 " >
                                <img class="h-8" src="/assets/discord.svg"/>
                                "Join"
                            </a>
                            <a href="#about" class=" text-[1rem] cursor-pointer border-b-[0.30rem] border-low-purple font-bold whitespace-nowrap">"Read More"</a>
                        </div>
                    </div>
                    <div class="text-left flex flex-col">
                        <h3 class="text-[2rem] font-bold">"Art Gallery"</h3>
                        <p class="text-[1.3rem]">"With thousands of unique art posted by the community"</p>
                        <div class="flex gap-8 mt-4 items-center ">
                            <a href="/gallery" class="bg-gradient-to-br from-first-one to-second-one hover:to-dark-purple flex h-12 gap-2 items-center text-[1rem] font-black bg-half-purple border-[0.30rem] border-low-purple rounded-3xl px-4 py-[0.15rem] hover:bg-dark-purple transition-colors duration-300 " >
                                "Gallery"
                            </a>
                        </div>
                    </div>
                </div>
                <div class="  grid place-items-center mt-auto text-center font-bold text-white  ">
                    <a href="#about" class="flex flex-col gap-2 justify-center">
                        "About"
                        <img class="h-[2rem]" src="/assets/triangle.svg"/>
                    </a>
                </div>
                <div>
                    "Background by AYYWA"
                </div>
            </section>
            <section  id="about" class="backdrop-blur-[50px] bg-gradient-to-r from-dark-night2/25 to-light-flower/10 px-6 py-6 2xl:px-[6rem] desktop:px-[16rem] flex flex-col md:grid md:grid-rows-[auto_1fr_1fr_1fr_auto] md:grid-cols-[1fr_1fr] gap-8 md:gap-x-24 lg:gap-x-[6rem]  text-[1.3rem] min-h-[100svh]" >
                <div class="col-span-2 h-[4rem] "></div>
                <p class=""><img class="w-[2rem] h-[2rem] inline px-1" src="/assets/about_unleash.svg"/><strong>"Unleash"</strong>" Your Artistic Spirit: Whether you're a seasoned artist, an aspiring creator, or someone who simply appreciates the beauty of art, ArtCord welcomes everyone. Explore a diverse range of styles, mediums, fandoms and techniques that spark inspiration"</p>
                <p class=""><img class="w-[2rem] h-[2rem] inline px-1" src="/assets/about_connect.svg"/><strong>"Connect"</strong>" with Like-minded people: Forge connections with fellow artists and art-enjoyers from around the globe. Share your work, exchange tips, and engage in conversations. We're proud to have many friendly and welcoming members that support each other!" </p>
                <p class=""><img class="w-[2rem] h-[2rem] inline px-1" src="/assets/about_challenges.svg"/><strong>"Challenges"</strong> " and Events: Elevate your skills and challenge your artistic boundaries with our ecreative challenges and events. From themed prompts to collaborative projects, ArtCord provides a platform to push your creativity to new heights. Sometimes we also hold Giveaways with prizes!" </p>
                <p class=""><img class="w-[2rem] h-[2rem] inline px-1" src="/assets/about_showcase.svg"/><strong>"Showcase"</strong> " Your Masterpieces: Your art deserves to be seen! Showcase your masterpieces in dedicated channels, where the spotlight is on you. Receive constructive feedback, encouragement, and applause from an appreciative audience. Your art can also be seen on our website!" </p>
                <p class=""><img class="w-[2rem] h-[2rem] inline px-1" src="/assets/about_learn.svg"/><strong>"Learn and Share"</strong> " Knowledge: Whether you're a beginner seeking guidance or an expert willing to share your wisdom, ArtCord is welcoming you! Learn new techniques, discover resources, and contribute to a collective pool of artistic wisdom."</p>
                <p class=""><img class="w-[2rem] h-[2rem] inline px-1" src="/assets/about_join.svg"/><strong>"Join the ArtCord"</strong>" Family: We believe that art has the power to connect people across borders and languages. Join ArtCord and become a part of a global family where creativity flows endlessly and art is appreciated."</p>
                <a target="_blank" href="https://discord.gg/habmw7Ehga" class=" transition-shadow duration-300 hover:shadow-wow col-span-2 bg-sword-ico bg-contain bg-center w-[10rem] h-[10rem] bg-no-repeat grid place-items-center text-center mx-auto cursor-pointer  font-black text-[3rem]">
                    "JOIN"
                </a>
            </section>
        </main>
    }
}
