pub mod gallery {
    use leptos::{html::Div, prelude::*};
    use std::fmt::Debug;
    use tracing::trace;
    use web_sys::js_sys::Math::random;

    pub const NEW_IMG_HEIGHT: u32 = 250;

    #[component]
    pub fn Gallery(imgs: impl Fn() -> Vec<(usize, Img)> + Send + Sync + 'static) -> impl IntoView {
        let gallery_ref = NodeRef::<Div>::new();

        view! {
            <div
                id="gallery"
                node_ref=gallery_ref
                class="relative overflow-y-scroll overflow-x-hidden"
            >
                <div class="bg-red-600 h-[100px] w-[500px] left-0 top-0 absolute">// style:width=move || format!("{}px", gallery_wdith.get())
                </div>
                <For
                    each=imgs
                    key=|img| img.1.id
                    children=move |(i, img)| {
                        view! { <GalleryImg index=i img /> }
                    }
                />
            </div>
        }
    }

    #[component]
    pub fn GalleryImg(
        img: Img,
        #[prop(optional)] index: usize,
        #[prop(optional)] node_ref: Option<NodeRef<Div>>,
    ) -> impl IntoView {
        let gallery_img_ref = NodeRef::<Div>::new();

        gallery_img_ref.on_load(move |e| {
            trace!("did i load or what? o.O");
        });

        Effect::new(move || {
            if index != 0 {
                return;
            }
            trace!("omg, i think im the first one");
            let Some(gallery_img_ref) = gallery_img_ref.get() else {
                return;
            };
            gallery_img_ref.scroll_into_view();
            // if let Some(node_ref) = node_ref {

            //     node_ref.track();
            //     trace!("tracking!");
            // }
        });

        let width = img.width;
        let height = img.height;
        let view_width = img.view_width;
        let view_height = img.view_height;
        let left = img.view_pos_x;
        let top = img.view_pos_y;
        let r = (random().to_bits() % 255) as u8;
        let g = (random().to_bits() % 255) as u8;
        let b = (random().to_bits() % 255) as u8;

        let fn_background = move || format!("rgb({}, {}, {})", r, g, b);
        let fn_left = move || format!("{}px", left.get());
        let fn_top = move || format!("{}px", top.get() + 100.0);
        let fn_width = move || format!("{}px", view_width.get());
        let fn_height = move || format!("{}px", view_height.get());

        view! {
            <div
                node_ref=gallery_img_ref
                // node_ref=first_ref
                class="text-white grid place-items-center bg-blue-950 absolute border border-red-600 overflow-hidden"
                style:background-color=fn_background
                style:left=fn_left
                style:top=fn_top
                style:width=fn_width
                style:height=fn_height
            >
                {format!("{}x{}", width, height)}
            </div>
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Img {
        pub id: u64,
        pub width: u32,
        pub height: u32,
        pub view_width: RwSignal<f32>,
        pub view_height: RwSignal<f32>,
        pub view_pos_x: RwSignal<f32>,
        pub view_pos_y: RwSignal<f32>,
    }

    impl GalleryImg for Img {
        fn get_size(&self) -> (u32, u32) {
            (self.width, self.height)
        }

        fn set_pos(&mut self, left: f32, top: f32, new_width: f32, new_height: f32) {
            self.view_width.set(new_width);
            self.view_height.set(new_height);
            self.view_pos_x.set(left);
            self.view_pos_y.set(top);
        }
    }

    impl Img {
        pub fn rand() -> Self {
            // let a = random() as u64;
            // let id = ;
            // trace!("id: {}", id);
            let id = random().to_bits();
            let width = (random().to_bits() % 1000) as u32;
            let height = (random().to_bits() % 1000) as u32;
            // let mut rng = rand::rng();
            // let id = rng.random::<u64>();
            // let width = rng.random_range(1_u64..1000);
            // let height = rng.random_range(1_u64..1000);

            Self {
                id,
                width,
                height,
                view_width: RwSignal::new(0.0),
                view_height: RwSignal::new(0.0),
                view_pos_x: RwSignal::new(0.0),
                view_pos_y: RwSignal::new(0.0),
            }
        }

        pub fn rand_vec(n: usize) -> Vec<Self> {
            let mut output = Vec::new();
            for _ in 0..n {
                output.push(Img::rand());
            }
            output
        }
    }

    pub trait GalleryImg {
        fn get_size(&self) -> (u32, u32);
        // fn get_pos(&self) -> (f32, f32);
        fn set_pos(&mut self, left: f32, top: f32, new_width: f32, new_height: f32);
    }

    pub fn resize_img<T: GalleryImg + Debug>(
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
            let (width, height) = imgs[i].get_size();
            let new_width = optimal_height * (width as f32 / height as f32);
            let new_height = optimal_height;
            imgs[i].set_pos(left, *top, new_width, new_height);
            left += new_width;
        }
        *top += optimal_height;
    }

    pub fn resize_img2<T: GalleryImg + Debug>(
        top: &mut f32,
        max_width: u32,
        new_row_start: usize,
        new_row_end: usize,
        imgs: &mut [T],
    ) {
        let mut optimal_count =
            (max_width as i32 / NEW_IMG_HEIGHT as i32) - (new_row_end - new_row_start) as i32;
        if optimal_count < 0 {
            optimal_count = 0;
        }
        let mut total_ratio: f32 = optimal_count as f32;
        if max_width < NEW_IMG_HEIGHT * 3 {
            total_ratio = 0.0;
        }

        for i in new_row_start..(new_row_end + 1) {
            let (width, height) = imgs[i].get_size();
            total_ratio += width as f32 / height as f32;
        }
        let optimal_height: f32 = max_width as f32 / total_ratio;
        let mut left: f32 = 0.0;

        for i in new_row_start..(new_row_end + 1) {
            let (width, height) = imgs[i].get_size();
            let new_width = optimal_height * (width as f32 / height as f32);
            let new_height = optimal_height;
            imgs[i].set_pos(left, *top, new_width, new_height);
            left += new_width;
        }

        *top += optimal_height;
    }

    pub fn resize_imgs<T: GalleryImg + Debug>(
        new_height: u32,
        max_width: u32,
        imgs: &mut [T],
    ) -> () {
        // debug!("utils: resizing started: count: {}", imgs.len());
        let loop_start = 0;
        let loop_end = imgs.len();
        let mut new_row_start: usize = 0;
        let mut new_row_end: usize = if loop_end > 0 { loop_end - 1 } else { 0 };
        let mut current_row_filled_width: u32 = 0;
        let mut top: f32 = 0.0;

        for index in loop_start..loop_end {
            let org_img = &mut imgs[index];
            let (width, height) = org_img.get_size();
            let ratio: f32 = width as f32 / height as f32;
            let height_diff: u32 = if height < new_height {
                0
            } else {
                height - new_height
            };
            let new_width: u32 = width - (height_diff as f32 * ratio) as u32;
            if (current_row_filled_width + new_width) <= max_width {
                current_row_filled_width += new_width;
                new_row_end = index;
                if index == loop_end - 1 {
                    resize_img2(&mut top, max_width, new_row_start, new_row_end, imgs);
                }
            } else {
                if index != 0 {
                    resize_img(&mut top, max_width, new_row_start, new_row_end, imgs);
                }
                new_row_start = index;
                new_row_end = index;
                current_row_filled_width = new_width;
                if index == loop_end - 1 {
                    resize_img2(&mut top, max_width, new_row_start, new_row_end, imgs);
                }
            }
        }

        // debug!("utils: resizing ended: count: {}", imgs.len());
    }

    pub fn calc_fit_count(width: u32, height: u32) -> u32 {
        (width * height) / (NEW_IMG_HEIGHT * NEW_IMG_HEIGHT)
    }
}
