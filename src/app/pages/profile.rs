use bson::DateTime;
use leptos::logging::log;
use leptos::*;
use leptos_router::use_params_map;

use crate::app::components::navbar::Navbar;
use crate::app::components::profile_gallery::ProfileGallery;
use crate::app::utils::{GlobalState, ServerMsgImgResized};

#[component]
pub fn Profile() -> impl IntoView {
    let params = use_params_map();
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let nav_tran = global_state.nav_tran;

    create_effect(move |_| {
        nav_tran.set(true);
    });

    view! {
        <main class=move||format!("grid grid-rows-[1fr] min-h-[100dvh] transition-all duration-300 {}", if nav_tran.get() {"pt-[4rem]"} else {"pt-[0rem]"})>
            <Navbar/>
            <ProfileGallery />
        </main>
    }
}
