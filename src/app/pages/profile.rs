use bson::DateTime;
use leptos::*;

use crate::app::components::gallery::{Gallery, SelectedImg};
use crate::app::components::navbar::Navbar;
use crate::app::utils::{GlobalState, ServerMsgImgResized};
use crate::server::ClientMsg;

#[component]
pub fn Profile() -> impl IntoView {
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let imgs = global_state.gallery_imgs;
    let nav_tran = global_state.nav_tran;
    let selected_img: RwSignal<Option<SelectedImg>> = create_rw_signal(None);

    let select_click_img = move |img: ServerMsgImgResized| {
        selected_img.set(Some(SelectedImg {
            org_url: img.display_high.clone(),
            author_name: img.user.name.clone(),
            author_pfp: format!("/assets/gallery/pfp_{}.webp", img.user.id.clone()),
            width: img.width,
            height: img.height,
        }))
    };

    let on_fetch = move |from: DateTime, amount: u32| {
        let msg = ClientMsg::GalleryInit { amount, from };
        global_state.socket_send(msg);
    };

    view! {
         {
            move || {
                match selected_img.get() {
                    Some(img) => Some(view! {
                        <div on:click=move |_| { selected_img.set(None); } class=" absolute grid grid-rows-[1fr] left-0 top-0 w-screen h-[100dvh] place-items-center bg-gradient-to-br from-mid-purple/50 to-dark-purple/50 z-[150] ">
                            <div  >
                                <div class="flex justify-between items-center rounded-t-lg bg-dark-purple pl-2">
                                       <div class="flex gap-2">
                                            <div>"By "</div>
                                            <img class="border border-low-purple rounded-full bg-mid-purple h-[25px] " src=img.author_pfp/>
                                            <div>{img.author_name}</div>
                                       </div>
                                     <img class="cursor-pointer border-2 border-low-purple rounded-full bg-mid-purple w-[30px] h-[30px] p-1 m-2" src="assets/x.svg"/>
                                </div>
                                <img class="bg-mid-purple object-contain " alt="loading..." style=move|| format!("max-height: calc(100dvh - 70px); max-width: 100vw; height: min({1}px, calc(100vw * ( {1} / {0} ))); aspect-ratio: {0} / {1};", img.width, img.height) on:click=move |e| { e.stop_propagation();  } src=img.org_url/>
                            </div>
                        </div> }),
                None => None
                }
            }
        }

        <main class=move||format!("grid grid-rows-[1fr] min-h-[100dvh] transition-all duration-300 {}", if nav_tran() {"pt-[4rem]"} else {"pt-[0rem]"})>
            <Navbar/>
            <Gallery global_gallery_imgs=imgs on_click=select_click_img on_fetch=on_fetch />
        </main>
    }
}
