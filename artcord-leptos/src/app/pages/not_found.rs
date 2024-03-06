use leptos::*;

#[component]
pub fn NotFound() -> impl IntoView {
    // #[cfg(feature = "ssr")]
    // {
    //     let resp = expect_context::<leptos_actix::ResponseOptions>();
    //     resp.set_status(actix_web::http::StatusCode::NOT_FOUND);
    // }

    view! {
        <h1>"Not Found"</h1>
    }
}
