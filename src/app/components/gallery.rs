use crate::app::components::navbar::{shrink_nav, Navbar};
use bson::DateTime;
use chrono::Utc;
use leptos::ev::resize;
use leptos::html::Section;
use leptos::logging::log;
use leptos::*;
use leptos_use::{use_event_listener, use_window};
use rand::Rng;
use web_sys::Event;

use crate::app::utils::{GlobalState, ServerMsgImgResized};
use crate::server::{ClientMsg, ServerMsg, SERVER_MSG_IMGS_NAME};

pub const NEW_IMG_HEIGHT: u32 = 250;

#[derive(Clone)]
struct SelectedImg {
    pub org_url: String,
    pub author_name: String,
    pub author_pfp: String,
    pub width: u32,
    pub height: u32,
}

pub trait GalleryImg {
    fn get_size(&self) -> (u32, u32);
    fn set_pos(&mut self, left: f32, top: f32, new_width: f32, new_height: f32);
    fn mark_as_modified(&mut self, id: u128);
}

fn resize_img<T: GalleryImg>(
    top: &mut f32,
    max_width: u32,
    new_row_start: usize,
    new_row_end: usize,
    imgs: &mut [T],
) {
    let mut total_ratio: f32 = 0f32;

    for i in new_row_start..(new_row_end + 1) {
        let (width, height) = imgs[i].get_size();
        total_ratio += width as f32 / height as f32;
    }
    let optimal_height: f32 = max_width as f32 / total_ratio;
    let mut left: f32 = 0.0;

    for i in new_row_start..(new_row_end + 1) {
        // let line = String::new();
        let (width, height) = imgs[i].get_size();
        let new_width = optimal_height * (width as f32 / height as f32);
        let new_height = optimal_height;
        imgs[i].set_pos(left, *top, new_width, new_height);
        // imgs[i].new_width = optimal_height * (imgs[i].width as f64 / imgs[i].height as f64);
        // imgs[i].new_height = optimal_height;
        // imgs[i].left = left;
        // imgs[i].top = *top;
        left += new_width;
    }

    // let mut total: f64 = 0.0;
    // for i in new_row_start..(new_row_end + 1) {
    //     total += imgs[i].new_width;
    // }
    // log!("line: {}", total);

    *top += optimal_height;
}

pub fn resize_imgs<T: GalleryImg>(new_height: u32, max_width: u32, imgs: &mut [T]) -> () {
    let loop_start = 0;
    let loop_end = imgs.len();
    let mut new_row_start: usize = 0;
    let mut new_row_end: usize = if loop_end > 0 { loop_end - 1 } else { 0 };
    let mut current_row_filled_width: u32 = 0;
    // let new_height: u32 = NEW_IMG_HEIGHT;
    let mut top: f32 = 0.0;

    let mut rand = rand::thread_rng();

    for index in loop_start..loop_end {
        let org_img = &mut imgs[index];
        let (width, height) = org_img.get_size();
        // let width: u32 = org_img.width;
        // let height: u32 = org_img.height;
        let ratio: f32 = width as f32 / height as f32;
        let height_diff: u32 = if height < new_height {
            0
        } else {
            height - new_height
        };
        let new_width: u32 = width - (height_diff as f32 * ratio) as u32;
        let id = rand.gen::<u128>();
        org_img.mark_as_modified(id);

        if (current_row_filled_width + new_width) <= max_width {
            current_row_filled_width += new_width;
            new_row_end = index;
            if index == loop_end - 1 {
                resize_img(&mut top, max_width, new_row_start, new_row_end, imgs);
            }
        } else {
            resize_img(&mut top, max_width, new_row_start, new_row_end, imgs);

            new_row_start = index;
            new_row_end = index;
            current_row_filled_width = new_width;
            if index == loop_end - 1 {
                resize_img(&mut top, max_width, new_row_start, new_row_end, imgs);
            }
        }
    }
}

fn calc_fit_count(width: u32, height: u32) -> u32 {
    (width * height) / (NEW_IMG_HEIGHT * NEW_IMG_HEIGHT)
}

fn create_client_test_imgs() -> Vec<ServerMsgImgResized> {
    let mut new_imgs: Vec<ServerMsgImgResized> = Vec::new();
    for _ in 0..25 {
        new_imgs.push(ServerMsgImgResized::default());
    }
    new_imgs
}

#[component]
pub fn Gallery<F: Fn() -> IV, IV: IntoView>(
    global_gallery_imgs: RwSignal<Vec<ServerMsgImgResized>>,
    render_prop: F,
) -> impl IntoView {
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let gallery_section = create_node_ref::<Section>();
    let selected_img: RwSignal<Option<SelectedImg>> = create_rw_signal(None);
    let nav_tran = global_state.nav_tran;

    // create_effect(move |_| {
    //     global_gallery_imgs.update_untracked(move |imgs| {
    //         log!("once on load");
    //         let section = gallery_section.get_untracked();
    //         if let Some(section) = section {
    //             let width = section.client_width() as u32;
    //
    //             resize_imgs(NEW_IMG_HEIGHT, width, imgs);
    //         };
    //     });
    // });

    create_effect(move |_| {
        let _ = use_event_listener(use_window(), resize, move |_| {
            // log!("TRYING TO RESIZE");
            global_gallery_imgs.update(|imgs| {
                let section = gallery_section.get_untracked();
                if let Some(section) = section {
                    let width = section.client_width() as u32;

                    // log!("RESIZING!!!!!!!!!!!!");
                    resize_imgs(NEW_IMG_HEIGHT, width, imgs);
                };
            });
        });
    });

    create_effect(move |_| {
        let connected = global_state.socket_connected.get();
        let loaded = global_state.gallery_loaded.get();
        if loaded || !connected {
            // log!("ITS NOT READY LOADED");
            return;
        }

        let Some(section) = gallery_section.get() else {
            return;
        };
        // log!("THIS SHOULDN'T HAPPEN {} {}", loaded, connected);

        let client_height = section.client_height();
        let client_width = section.client_width();

        let msg = ClientMsg::GalleryInit {
            amount: calc_fit_count(client_width as u32, client_height as u32) * 2,
            from: DateTime::from_millis(Utc::now().timestamp_millis()),
        };

        global_state.socket_send(msg);
        global_state.gallery_loaded.set(true);
    });

    let section_scroll = move |_: Event| {
        if !global_state.socket_state_is_ready(SERVER_MSG_IMGS_NAME) {
            return;
        }

        let Some(last) = global_gallery_imgs.with_untracked(|imgs| match imgs.last() {
            Some(l) => Some(l.created_at),
            None => None,
        }) else {
            return;
        };

        let Some(section) = gallery_section.get_untracked() else {
            return;
        };

        let scroll_top = section.scroll_top();
        let client_height = section.client_height();
        let scroll_height = section.scroll_height();
        let client_width = section.client_width();

        shrink_nav(nav_tran, scroll_top as u32);

        let left = scroll_height - (client_height + scroll_top);

        if left < client_height {
            let msg = ClientMsg::GalleryInit {
                amount: calc_fit_count(client_width as u32, client_height as u32) * 2,
                from: last,
            };
            global_state.socket_send(msg);
            global_state.socket_state_used(SERVER_MSG_IMGS_NAME);
        }
    };

    let select_click_img = move |img: &ServerMsgImgResized| {
        selected_img.set(Some(SelectedImg {
            org_url: img.display_high.clone(),
            author_name: img.user.name.clone(),
            author_pfp: format!("/assets/gallery/pfp_{}.webp", img.user.id.clone()),
            width: img.width,
            height: img.height,
        }))
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

        <section id="gallery_section" on:scroll=section_scroll _ref=gallery_section class="relative content-start overflow-x-hidden overflow-y-scroll h-full" >
            <For each=global_gallery_imgs key=|state| state._id let:img >
                {

                        // let (w, h) = img.get_size();
                        // let height = ;
                        // let with = format!("{}px", w);
                        let height = img.new_height;
                        let width = img.new_width;
                        let top = img.top;
                        let left = img.left;
                        let bg_img = format!("url('{}')", &img.display_preview);

                        view! {
                            <div
                                class="absolute bg-center bg-contain transition-all bg-no-repeat flex-shrink-0 font-bold grid place-items-center border hover:shadow-glowy hover:z-[99]  duration-300 bg-mid-purple  border-low-purple"
                                style:height=move || format!("{}px", height.get())
                                style:width=move || format!("{}px", width.get())
                                style:top=move || format!("{}px", top.get())
                                style:left=move || format!("{}px", left.get())
                                style:background-image=bg_img
                                on:click= move |_| select_click_img(&img)
                            >
                                // <div class="relative flex opacity-0 hover:opacity-100 transition duration-300 w-full h-full flex-col text-center justify-center gap-2  "  >
                                //     <div class="absolute bg-dark-purple bottom-0 left-0 translate-y-full w-full">{&img.user.name}</div>
                                // </div>
                            </div>
                        }
                }
            </For>
        </section>
    }
}
