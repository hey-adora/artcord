use components::gallery::Img;
use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;
use page::home;
use reactive_stores::Store;
use tracing::trace;

pub mod components;
pub mod page;

#[derive(Clone, Debug, Store)]
pub struct GlobalState {
    imgs: Vec<Img>,
}

impl Default for GlobalState {
    fn default() -> Self {
        let imgs = Vec::new();
        Self { imgs }
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_context(Store::new(GlobalState::default()));

    view! {
        <Router>
            <Routes fallback=|| "not found">
                <Route path=path!("") view=home::Page />
                <Route
                    path=path!("two")
                    view=move || {
                        view! {
                            <nav class="text-gray-200 pb-1">
                                <a href="/" class="font-black text-xl">
                                    "ArtBounty"
                                </a>
                                <a href="/two">"two"</a>
                            </nav>
                        }
                    }
                />
            </Routes>
        </Router>
    }
}
