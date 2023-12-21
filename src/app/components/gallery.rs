use crate::app::components::navbar::{shrink_nav, Navbar};
use bson::DateTime;
use chrono::Utc;
use leptos::ev::resize;
use leptos::html::{ElementDescriptor, Section};
use leptos::logging::log;
use leptos::*;
use leptos_use::{use_event_listener, use_window};
use rand::Rng;
use web_sys::Event;

use crate::app::utils::{GlobalState, ServerMsgImgResized};
use crate::server::{ClientMsg, ServerMsg, SERVER_MSG_IMGS_NAME};

pub const NEW_IMG_HEIGHT: u32 = 250;

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

//F: Fn(ServerMsgImgResized) -> IV + 'static, IV: IntoView
#[component]
pub fn Gallery<
    OnClick: Fn(ServerMsgImgResized) + Copy + 'static,
    OnFetch: Fn(DateTime, u32) + Copy + 'static,
>(
    global_gallery_imgs: RwSignal<Vec<ServerMsgImgResized>>,
    on_click: OnClick,
    on_fetch: OnFetch,
) -> impl IntoView {
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let gallery_section = create_node_ref::<Section>();
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

        let section = gallery_section;

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
            on_fetch(
                last,
                calc_fit_count(client_width as u32, client_height as u32) * 2,
            );
            // let msg = ClientMsg::GalleryInit {
            //     amount: calc_fit_count(client_width as u32, client_height as u32) * 2,
            //     from: last,
            // };
            // global_state.socket_send(msg);
            global_state.socket_state_used(SERVER_MSG_IMGS_NAME);
        }
    };

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

        on_fetch(
            DateTime::from_millis(Utc::now().timestamp_millis()),
            calc_fit_count(client_width as u32, client_height as u32) * 2,
        );

        // let msg = ClientMsg::GalleryInit {
        //     amount: calc_fit_count(client_width as u32, client_height as u32) * 2,
        //     from: DateTime::from_millis(Utc::now().timestamp_millis()),
        // };

        //global_state.socket_send(msg);
        global_state.gallery_loaded.set(true);
    });

    view! {
        <section id="gallery_section" on:scroll=section_scroll _ref=gallery_section class="relative content-start overflow-x-hidden overflow-y-scroll h-full" >
            <For each=move || global_gallery_imgs.get().into_iter().enumerate()  key=|state| state.1._id let:data > {
                    let img = data.1;
                    let i = data.0;
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
                                on:click=  move |_| on_click(global_gallery_imgs.with_untracked(|imgs|imgs[i].clone()))
                            >
                            </div>
                        }
                    }

            </For>
        </section>
    }
}
