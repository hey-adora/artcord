pub mod home {
    use crate::toolbox::prelude::*;
    use leptos::prelude::*;
    use reactive_stores::Store;
    use tracing::trace;
    use web_sys::{HtmlDivElement, HtmlElement};

    use crate::app::{
        GlobalState,
        components::gallery::{Gallery, Img, resize_imgs},
    };

    #[component]
    pub fn Page() -> impl IntoView {
        let main_ref = NodeRef::new();
        let global_state = expect_context::<GlobalState>();
        let imgs = global_state.imgs;

        Effect::new(move || {
            let new_imgs = Img::rand_vec(100);
            imgs.set(new_imgs);
        });

        main_ref.on_file_drop(async |event, data| {
            for file in data.get_files() {
                let stream = file.get_file_stream()?;
                let mut data = Vec::<u8>::new();
                while let Some(chunk) = stream.get_stream_chunk().await? {
                    chunk.push_to_vec(&mut data);
                }
                let data_str = String::from_utf8_lossy(&data);
                trace!("file: {}", data_str);
            }

            Ok(())
            // for file in data.files().iter() {
            //     let data = file.data().await;
            //     let data = data.map(|data| String::from_utf8_lossy(&data).to_string());
            //     trace!("file! {:?}", data);
            // }
        });

        // let get_imgs = move || {
        //     let Some(gallery_elm): Option<HtmlElement> = main_ref.get() else {
        //         trace!("refresh target not found");
        //         return Vec::new();
        //     };
        //     trace!("refreshing...");
        //     let width = gallery_elm.client_width() as u32;

        //     let mut imgs = imgs.get();
        //     resize_imgs(200, width, &mut imgs);
        //     imgs.into_iter().enumerate().collect::<Vec<(usize, Img)>>()
        // };

        view! {
            <main node_ref=main_ref class="grid grid-rows-[auto_1fr] h-screen">
                <nav class="text-gray-200 pb-1">
                    <a href="/" class="font-black text-xl">
                        "ArtBounty"
                    </a>
                    <a href="/two">"two"</a>
                </nav>
                <Gallery imgs=imgs />
            </main>
        }
    }
}
