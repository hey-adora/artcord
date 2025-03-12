use components::gallery::Img;
use indextree::Arena;
use indextree::NodeId;
use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;
use page::home;
use reactive_stores::Store;
use tracing::trace;

use crate::toolbox::prelude::*;

pub mod components;
pub mod page;

#[derive(Clone, Default, Debug)]
pub struct GlobalState {
    imgs: RwSignal<Vec<Img>>,
    id: RwSignal<usize>,
    tree: RwSignal<Arena<usize>>,
    current: RwSignal<Option<NodeId>>,
}

// impl Default for GlobalState {
//     fn default() -> Self {
//         Self {
//             imgs: RwSignal::new(Vec::new()),
//         }
//     }
// }

#[component]
pub fn App() -> impl IntoView {
    provide_context(GlobalState::default());

    resize_observer::init_global_state();
    //intersection_observer::init_global_state();

    // Effect::new(move || {
    //     use indextree::Arena;

    //     // Create a new arena
    //     let arena = &mut Arena::new();

    //     // Add some new nodes to the arena
    //     let a = arena.new_node(1);
    //     let b = arena.new_node(2);
    //     let c = arena.new_node(3);
    //     a.append(b, arena);
    //     b.append(c, arena);

    //     // Append a to b
    //     // assert!(a.append(b, arena).is_ok());
    //     // assert_eq!(b.ancestors(arena).into_iter().count(), 2);
    //     trace!("{:#?}", arena);
    // });

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
