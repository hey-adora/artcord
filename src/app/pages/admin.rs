use leptos::*;
use leptos_router::use_params_map;

use crate::app::components::navbar::Navbar;
use crate::app::components::profile_gallery::ProfileGallery;
use crate::app::global_state::GlobalState;

#[component]
pub fn Admin() -> impl IntoView {
    let params = use_params_map();
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let nav_tran = global_state.nav_tran;

    create_effect(move |_| {
        nav_tran.set(true);
    });

    view! {
        <main class=move||format!("grid grid-rows-[1fr] min-h-[100dvh] transition-all duration-300 {}", if nav_tran() {"pt-[4rem]"} else {"pt-[0rem]"})>
            <Navbar/>
            <div class="flex gap-4 bg-white ">
                <div class="flex flex-col gap-4 bg-black px-6 py-4">
                    <div class="font-bold">"DASHBOARD"</div>
                    <div class="flex flex-col gap-2 ">
                        <div>"Activity"</div>
                        <div>"Banned IP's"</div>
                        <div>"Users"</div>
                    </div>
                </div>
                <div class="w-full text-black py-4 gap-4 flex flex-col  ">
                    <div class="font-bold">"Activity"</div>
                    <div>"Activity"</div>
                </div>
            </div>
        </main>
    }
}
