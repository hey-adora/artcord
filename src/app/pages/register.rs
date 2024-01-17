use crate::app::components::navbar::Navbar;
use crate::app::utils::GlobalState;
use crate::server::client_msg::ClientMsg;
use crate::server::registration_invalid::{RegistrationInvalidMsg, MINIMUM_PASSWORD_LENGTH};
use leptos::html::Input;
use leptos::logging::log;
use leptos::*;
use web_sys::SubmitEvent;

pub fn validate_registration(
    email: &str,
    password: &str,
    password_confirmation: &str,
) -> (bool, Option<String>, Option<String>) {
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
    } else if password_confirmation.len() < 1 {
        Some("Password confirm field can't be empty.".to_string())
    } else if password != password_confirmation {
        Some("Password confirm doesn't match.".to_string())
    } else {
        None
    };

    let invalid = email_error.is_some() || password_error.is_some();

    (invalid, email_error, password_error)
}

#[component]
pub fn Register() -> impl IntoView {
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");

    let input_email: NodeRef<Input> = create_node_ref();
    let input_password: NodeRef<Input> = create_node_ref();
    let input_password_confirm: NodeRef<Input> = create_node_ref();

    let input_email_error: RwSignal<Option<String>> = RwSignal::new(None);
    let input_password_error: RwSignal<Option<String>> = RwSignal::new(None);

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let Some(email) = input_email.get() else {
            return;
        };
        let Some(password) = input_password.get() else {
            return;
        };
        let Some(password_confirm) = input_password_confirm.get() else {
            return;
        };
        let email = email.value();
        let email_trimmed = email.trim();

        let password = password.value();
        let password_trimmed = password.trim();

        let password_confirm = password_confirm.value();
        let password_confirm_trimmed = password_confirm.trim();

        let (invalid, email_error, password_error) =
            validate_registration(email_trimmed, password_trimmed, password_confirm_trimmed);

        input_email_error.set(email_error);
        input_password_error.set(password_error);

        if invalid {
            return;
        }

        log!("Submit: '{}' '{}' '{}'", email, password, password_confirm);

        let msg = ClientMsg::Register { password, email };

        global_state.socket_send(msg);
    };

    let show_error_when = |signal: RwSignal<Option<String>>| -> bool {
        let Some(value) = signal.get() else {
            return false;
        };

        if value.len() < 1 {
            return false;
        }

        true
    };

    view! {
        <main class=move||format!("grid grid-rows-[1fr] min-h-[100dvh] transition-all duration-300 pt-[4rem]")>
            <Navbar/>
            <section>
                <form class="text-black flex flex-col gap-2 w-[20rem] mx-auto bg-black" on:submit=on_submit>
                    <div class="flex flex-col">
                        <label for="email" class="text-white">"Email2"</label>
                        <Show when=move || show_error_when(input_email_error) >
                            <div class="text-white">{input_email_error.get()}</div>
                        </Show>
                        <input _ref=input_email id="email" type="text"/>
                    </div>
                    <div class="flex flex-col" >
                        <label for="password" class="text-white">"Password"</label>
                        <Show when=move || show_error_when(input_password_error) >
                            <div class="text-white">{input_password_error.get()}</div>
                        </Show>
                        <input _ref=input_password id="password" type="text"/>
                    </div>
                    <div class="flex flex-col" >
                        <label for="password_confirm" class="text-white">"Password confirm"</label>
                        <input _ref=input_password_confirm id="password_confirm" type="text"/>
                    </div>
                    <input class="text-white" type="submit" value="Register" />
                </form>
            </section>
        </main>
    }
}
