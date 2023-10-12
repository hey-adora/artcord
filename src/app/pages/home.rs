use leptos::*;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <section class="px-6 py-6 line-bg grid grid-rows-[1fr_1fr_0.3fr] md:grid-rows-[1fr] md:grid-cols-[1fr_1fr] place-items-center  overflow-hidden " style=move|| format!("min-height: calc(100vh - 100px)")>
                <div class=" bg-the-star bg-center bg-contain bg-no-repeat h-full w-full grid place-items-center  ">
                    <div class="text-center flex flex-col">
                        <h1 class="text-[4rem] font-bold">"ArtCord"</h1>
                        <h2 class="text-[2rem]">"Discord Art Server"</h2>
                        <div class="flex gap-8 mt-4 items-center justify-center">
                            <a href="#about" class=" text-[1rem] cursor-pointer border-b-[0.30rem] border-low-purple font-bold whitespace-nowrap">"Read More"</a>
                            <a target="_blank" href="https://discord.gg/habmw7Ehga" class="flex gap-2 items-center text-[1rem] font-black bg-half-purple border-[0.30rem] border-low-purple rounded-3xl px-4 py-[0.15rem] hover:bg-dark-purple transition-colors duration-300 " >
                                <img src="/assets/discord.svg"/>
                                "Join"
                            </a>
                        </div>
                    </div>
                </div>
                <div class="flex flex-col  justify-center gap-6 sm:gap-12">
                    <div class="flex justify-center relative">
                        <div class="z-10 font-bold text-center flex flex-col border-2 border-low-purple absolute rotate-[15deg] translate-x-[60%] bg-dark2-purple ">
                            <div>"@moyanice"</div>
                            <div class="  w-[32vw] h-[55vw] lg:max-w-[15rem] lg:max-h-[25rem] max-w-[10rem] max-h-[20rem] bg-center bg-cover " style="background-image: url('/assets/1.jpg')" ></div>
                        </div>
                        <div class="z-20 font-bold text-center flex flex-col border-2 border-low-purple bg-dark2-purple">
                            <div>"@valnikryatuveli"</div>
                            <div class=" w-[32vw] h-[55vw] lg:max-w-[15rem] lg:max-h-[25rem] max-w-[10rem] max-h-[20rem] bg-center bg-cover" style="background-image: url('/assets/2.jpg')" ></div>
                        </div>
                        <div class="z-10 font-bold text-center flex flex-col border-2 border-low-purple absolute -rotate-[15deg] -translate-x-[60%] bg-dark2-purple">
                            <div>"@stalkstray"</div>
                            <div class="z-10 w-[32vw] h-[55vw] lg:max-w-[15rem] lg:max-h-[25rem] max-w-[10rem] max-h-[20rem] bg-center bg-cover " style="background-image: url('/assets/3.jpg')" ></div>
                        </div>
                    </div>
                    <div class="flex justify-center">
                        <a href="/gallery" class=" shadow-glowy text-[1rem] font-black bg-half-purple border-[0.30rem] border-low-purple rounded-3xl px-4 py-[0.15rem] hover:bg-dark-purple transition-colors duration-300 " >
                            "View Gallery"
                        </a>
                    </div>

                </div>
                <div class=" md:col-span-2 grid place-items-center mt-auto text-center font-bold ">
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
