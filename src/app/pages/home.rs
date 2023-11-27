use crate::app::components::navbar::Navbar;
use leptos::*;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <Navbar/>
        <section class=" px-6 py-6 bg-sword-lady bg-[right_85%_bottom_0] md:bg-center bg-cover bg-no-repeat grid grid-rows-[auto_auto_1fr] grid-cols-[1fr]  min-h-[100svh]" >
            <div class="h-[4rem]"></div>
            <div class="flex flex-col gap-[2rem] md:gap-[4rem]  max-w-min">
                <div class="text-left flex flex-col justify-start">
                    <h2 class="text-[2rem] font-bold whitespace-nowrap ">"Discord Art Server"</h2>
                    <p class="text-[1.3rem]">"Where creativity knows no bounds and artistic expression finds its true home! "</p>
                    <div class="flex gap-8 mt-4 items-center ">
                        <a target="_blank" href="https://discord.gg/habmw7Ehga" class="flex gap-2 items-center text-[1rem] font-black bg-half-purple border-[0.30rem] border-low-purple rounded-3xl px-4 py-[0.15rem] hover:bg-dark-purple transition-colors duration-300 " >
                            <img src="/assets/discord.svg"/>
                            "Join"
                        </a>
                        <a href="#about" class=" text-[1rem] cursor-pointer border-b-[0.30rem] border-low-purple font-bold whitespace-nowrap">"Read More"</a>
                    </div>
                </div>
                <div class="text-left flex flex-col">
                    <h3 class="text-[2rem] font-bold">"Art Gallery"</h3>
                    <p class="text-[1.3rem]">"With thousands of unique art posted by the community"</p>
                    <div class="flex gap-8 mt-4 items-center ">
                        <a target="_blank" href="/gallery" class="flex gap-2 items-center text-[1rem] font-black bg-half-purple border-[0.30rem] border-low-purple rounded-3xl px-4 py-[0.15rem] hover:bg-dark-purple transition-colors duration-300 " >
                            "Galley"
                        </a>
                    </div>
                </div>
            </div>
            <div class="  grid place-items-center mt-auto text-center font-bold ">
                <a href="#about" class="flex flex-col gap-2 justify-center">
                    "About"
                    <img class="h-[2rem]" src="/assets/triangle.svg"/>
                </a>
            </div>
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
    }
}
