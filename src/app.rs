use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <meta name="viewport" content="width=device-width, initial-scale=1.0"/>

        <Stylesheet id="leptos" href="/pkg/leptos_start.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes>
                    <Route path="" view=HomePage/>
                    <Route path="/*any" view=NotFound/>
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let (count, set_count) = create_signal(0);
    //let on_click = move |_| set_count.update(|count| *count += 1);

    view! {
       <main class=" grid grid-rows-[auto_1fr] gap-6 pt-6 text-low-purple bg-gradient-to-br from-mid-purple to-dark-purple   min-h-screen  ">
        <nav class="px-6 flex items-center justify-between gap-2">
            <div class="flex items-baseline gap-6">
                <h3 class="  font-bold text-[2rem] ">"ArtCord"</h3>
                <ul class="hidden sm:flex gap-2 text-[1rem] text-center">
                    <li class=" w-[3.5rem] cursor-pointer border-b-[0.30rem] border-low-purple font-bold ">"Home"</li>
                    <li class=" w-[3.5rem] cursor-pointer border-b-[0.30rem] border-transparent hover:border-low-purple/40 hover:font-bold transition duration-300 " >"About"</li>
                    <li class=" w-[3.5rem] cursor-pointer border-b-[0.30rem] border-transparent hover:border-low-purple/40 hover:font-bold transition duration-300 ">"Gallery"</li>
                </ul>
            </div>
            <div>
                <a class="hidden sm:flex gap-2 items-center text-[1rem] font-black bg-half-purple border-[0.30rem] border-low-purple rounded-3xl px-4 py-[0.15rem] hover:bg-dark-purple transition-colors duration-300 " href=""> 
                    <img src="/assets/discord.svg"/> 
                    "Join"
                </a>
                <img class="cursor-pointer block sm:hidden " src="assets/burger.svg" alt=""/>
            </div>
        </nav>
        <section class="px-6 py-6 line-bg  grid grid-rows-[1fr_1fr_0.3fr] md:grid-rows-[1fr] md:grid-cols-[1fr_1fr] place-items-center  overflow-hidden " style=move|| format!("max-height: calc(100vh - 100px)")>
            <div class="text-center flex flex-col  ">
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
            <div class="flex flex-col  justify-center gap-6">
                <div class="flex justify-center relative">
                    <div class="z-10 w-[32vw] h-[55vw] lg:max-w-[15rem] lg:max-h-[25rem] max-w-[10rem] max-h-[20rem] bg-center bg-cover absolute rotate-[15deg] translate-x-[60%]" style="background-image: url('/assets/1.jpg')" ></div>
                    <div class="z-20 w-[32vw] h-[55vw] lg:max-w-[15rem] lg:max-h-[25rem] max-w-[10rem] max-h-[20rem] bg-center bg-cover" style="background-image: url('/assets/2.jpg')" ></div>
                    <div class="z-10 w-[32vw] h-[55vw] lg:max-w-[15rem] lg:max-h-[25rem] max-w-[10rem] max-h-[20rem] bg-center bg-cover absolute -rotate-[15deg] -translate-x-[60%]" style="background-image: url('/assets/3.jpg')" ></div>
                </div>
                <div class="flex justify-center">
                    <a class=" text-[1rem] font-black bg-half-purple border-[0.30rem] border-low-purple rounded-3xl px-4 py-[0.15rem] hover:bg-dark-purple transition-colors duration-300 " href=""> 
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
            // <div class="absolute w-[0.25rem]  h-full bg-low-purple/40"></div>
        </section>
        // <img class="" src="/assets/bg.svg" alt=""/>
       </main>
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
