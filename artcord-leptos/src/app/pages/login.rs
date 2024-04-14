use crate::app::components::navbar::Navbar;
use crate::app::global_state::GlobalState;
use crate::app::pages::register::{auth_input_show_error, AuthLoadingState};
//use crate::app::utils::signal_switch::signal_switch;
use artcord_leptos_web_sockets::WsResourcSendResult;
use artcord_state::message::prod_client_msg::ClientMsg;
use artcord_state::misc::registration_invalid::{RegistrationInvalidMsg, MINIMUM_PASSWORD_LENGTH};
use leptos::html::Input;
use leptos::leptos_dom::log;
use leptos::*;
use leptos_router::{use_navigate, NavigateOptions};
use tracing::debug;
use tracing::error;
use tracing::warn;
use web_sys::SubmitEvent;

pub fn validate_login(email: &str, password: &str) -> (bool, Option<String>, Option<String>) {
    let email_error = if email.len() < 1 {
        Some("Email field can't be empty.".to_string())
    } else {
        None
    };

    let password_error = if password.len() < 1 {
        Some("Password field can't be empty.".to_string())
    } else if password.len() < MINIMUM_PASSWORD_LENGTH {
        Some(format!(
            "Minimum password length is {}.",
            MINIMUM_PASSWORD_LENGTH
        ))
    } else {
        None
    };

    let invalid = email_error.is_some() || password_error.is_some();

    (invalid, email_error, password_error)
}

#[component]
pub fn Login() -> impl IntoView {
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let ws = global_state.ws;
    let loading_state = global_state.pages.login.loading_state;

    //signal_switch(false, &loading_state, AuthLoadingState::Connecting);
    //let suspended_loading_state = RwSignal::new(loading_state.get_untracked());

    let input_email: NodeRef<Input> = create_node_ref();
    let input_password: NodeRef<Input> = create_node_ref();

    let input_general_error: RwSignal<Option<String>> = RwSignal::new(None);
    let input_email_error: RwSignal<Option<String>> = RwSignal::new(None);
    let input_password_error: RwSignal<Option<String>> = RwSignal::new(None);

    let ws_login = ws.builder().portal().build();

    // ws_login.send_or_skip(ClientMsg::Logout, |msg| {
    //     debug!("hellllooo");
    // });

    // #[cfg(target_arch = "wasm32")]
    // ws.on_ws_state(move |is_connected| {
    //    // debug!("AAAAAAAAAAAAAAAAAAAAAAAAAAA: {}", is_connected);
    //     debug!("login: running on_ws_state: {}", is_connected);
    //     if is_connected {
    //       //  debug!("login: ready");
    //         loading_state.set(AuthLoadingState::Ready);
    //     } else {
    //       //  debug!("login: connecting");
    //         loading_state.set(AuthLoadingState::Connecting);
    //     }
    // });

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let Some(email) = input_email.get() else {
            return;
        };
        let Some(password) = input_password.get() else {
            return;
        };

        let email = email.value();
        let email_trimmed = email.trim();

        let password = password.value();
        let password_trimmed = password.trim();

        let (invalid, email_error, password_error) =
            validate_login(email_trimmed, password_trimmed);

        input_email_error.set(email_error);
        input_password_error.set(password_error);

        if invalid {
            return;
        }

        log!("Submit: '{}' '{}'", email, password);

        let msg = ClientMsg::Login { password, email };

        match ws_login.send_and_recv(msg, move |msg| {
            debug!("login: RECEIVED: {:?}", msg);
            loading_state.set(AuthLoadingState::Completed);
        }) {
            Ok(result) => {
                match result {
                    WsResourcSendResult::Sent | WsResourcSendResult::Queued => {
                        loading_state.set(AuthLoadingState::Processing);
                    }
                    WsResourcSendResult::Skipped => {
                        warn!("login: ws unexpected result: tried to login twice");
                        //loading_state.set(AuthLoadingState::Processing);
                    }
                }
            }
            Err(err) => {
                error!("login: ws error: {}", err);
                loading_state.set(AuthLoadingState::Failed(
                    RegistrationInvalidMsg::new().general(format!("login: failed: {}", err)),
                ));
            }
        }
        //global_state.socket_send(&msg); a
    };

    // create_effect(move |_| {
    //     let connected = global_state.socket_connected.get();
    //     let current_loading_state = loading_state.get_untracked();

    //     if !connected && current_loading_state != AuthLoadingState::Connecting {
    //         suspended_loading_state.set(current_loading_state);
    //         loading_state.set(AuthLoadingState::Connecting);
    //     } else if connected && current_loading_state == AuthLoadingState::Connecting {
    //         loading_state.set(suspended_loading_state.get_untracked());
    //     }
    // });

    // create_effect(move |_| {
    //     let navigation = use_navigate();

    //     if global_state.auth_is_logged_in() {
    //         navigation("/", NavigateOptions::default());
    //     }
    // });

    view! {
        <main class=move||format!("grid grid-rows-[1fr] place-items-center min-h-[100dvh] transition-all duration-300 pt-[4rem]")>
            <Navbar/>
            // <Show when=move|| global_state.socket_connected.get() fallback=move || { "Connecting..." }>
            //     <Show when=move||loaded_sig.with(|state| *state == LoadingNotFound::NotLoaded || *state == LoadingNotFound::Loading) >
            //     <div>"LOADING..."</div>
            //     </Show>
            //     <Show when=move||loaded_sig.with(|state| *state == LoadingNotFound::NotFound) >
            //     <div>"No Images Found."</div>
            //     </Show>
            // </Show>
            <Show when=move || ws.connected.get()  fallback = move|| view! {
                <section class="text-center text-black flex flex-col justify-center max-w-[20rem] w-full min-h-[20rem] bg-white rounded-3xl p-5 flex"  >
                "Connecting..."
                </section>
            }>
                <section class="text-center text-black flex flex-col justify-center max-w-[20rem] w-full min-h-[20rem] bg-white rounded-3xl p-5" style:display=move || if loading_state.get() == AuthLoadingState::Completed { "flex" } else {"none"} >
                "Login Completed\n."
                </section>
                <section class="text-center text-black flex flex-col justify-center max-w-[20rem] w-full min-h-[20rem] bg-white rounded-3xl p-5" style:display=move || if loading_state.get() == AuthLoadingState::Processing { "flex" } else {"none"} >
                        "Processing..."
                </section>
            </Show>



             <section class=" flex flex-col justify-center max-w-[20rem] w-full min-h-[20rem] bg-white rounded-3xl p-5" style:display=move || if ws.connected.get() && match loading_state.get() { AuthLoadingState::Ready =>true, AuthLoadingState::Failed(_) => true, _ => false } { "flex" } else {"none"} >
                        <form class="text-black flex flex-col gap-5 " on:submit=on_submit>
                            <Show when=move || auth_input_show_error(input_general_error) >
                                    <div class="text-red-600 text-center">{input_general_error.get()}</div>
                            </Show>
                            <div class="flex flex-col">
                                <label for="email" class="">"Email"</label>
                                <Show when=move || auth_input_show_error(input_email_error) >
                                    <div class="text-red-600">{input_email_error.get()}</div>
                                </Show>
                                <input class="border-black border-b-2 border-solid" _ref=input_email id="email" type="text"/>
                            </div>
                            <div class="flex flex-col" >
                                <label for="password" class="">"Password"</label>
                                <Show when=move || auth_input_show_error(input_password_error) >
                                    <div class="text-red-600">{input_password_error.get()}</div>
                                </Show>
                                <input class="border-black border-b-2 border-solid" _ref=input_password id="password" type="text"/>
                            </div>
                            <input class="border-black border-2 border-solid rounded hover:text-white hover:bg-black transition-colors duration-300" type="submit" value="Login" />
                        </form>
                </section>

        </main>
    }
}
