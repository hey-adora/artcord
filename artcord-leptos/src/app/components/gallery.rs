use crate::app::global_state::GlobalState;




use leptos::*;
use leptos_router::use_location;





use crate::app::utils::{
    LoadingNotFound, ServerMsgImgResized,
};

// //F: Fn(ServerMsgImgResized) -> IV + 'static, IV: IntoView
// #[derive(Copy, Clone, Debug)]
// pub struct SocketBs {
//     pub msgs: RwSignal<HashMap<u128, Rc<dyn Fn() -> ()>>>,
//     pub pending: RwSignal<Vec<u8>>,
// }

// impl SocketBs {
//     pub fn new() -> Self {
//         Self {
//             msgs: RwSignal::new(HashMap::new()),
//             pending: RwSignal::new(Vec::new()),
//         }
//     }
// }

// enum MsgPkg<S, R> {
//     Send(S),
//     Recv(R),
// }

// struct Sender {

// }

// impl Sender {

// }

// pub fn create_sender<S: 'static + Clone + PartialEq + Eq + PartialOrd + Ord, E: 'static + Clone + PartialEq + Eq + PartialOrd + Ord, F: Fn(ServerMsg) -> Result<S, E> + Copy + 'static>(on_receive: F) -> Rc<dyn Fn(&ClientMsg)> {
//     let uuid = uuid::Uuid::new_v4().to_u128_le();
//     let global_state = use_context::<GlobalState>().expect("Failed to provide socket_bs state");
//     let msgs = global_state.socket_closures;
//     let is_connected = global_state.socket_connected;
//     let default_state = if is_connected.get_untracked() { SenderState::Ready } else { SenderState::Connecting };
//     let old_state = RwSignal::new(default_state.clone());
//     let state = RwSignal::new(default_state);

//     let send_msg = move |client_msg: &ClientMsg| {
//         global_state.socket_send(uuid, client_msg);
//     };

//     create_effect(move |_| {
//         log!("ATTACHING: {:?}", msgs.with_untracked(|msgs|msgs.len()));
//         msgs.update_untracked(move |msgs| {
//             let a = Rc::new(move |server_msg| {
//                 let result = on_receive(server_msg);
//                 if let Ok(result) = result {
//                     state.set(SenderState::Succesfull(result));
//                 } else {
//                     state.set(SenderState::Error(result.err().unwrap()))
//                 }
//             });
//             msgs.insert(uuid, a);
//         });
//         log!("ATTACHED: {:?}", msgs.with_untracked(|msgs|msgs.len()));
//     });

//     create_effect(move |_| {
//         let is_connected = is_connected.get();
//         if is_connected {
//             if old_state.with_untracked(move |old_state| *old_state != SenderState::Connecting) {
//                 state.set(old_state.get_untracked());
//             } else {
//                 state.set(SenderState::Ready);
//             }
//         } else {
//             old_state.set(state.get_untracked());
//             state.set(SenderState::Connecting);
//         }
//     });

//     on_cleanup(move || {
//         msgs.update(move |msgs| {
//             msgs.remove(&uuid);
//         });
//         log!("DETACHING: {:?}", msgs.with_untracked(|msgs|msgs.len()));
//     });

//     Rc::new(send_msg)
// }

// pub fn execute(id: u128, server_msg: ServerMsg) {
//     let global_state = use_context::<GlobalState>().expect("Failed to provide socket_bs state");
//     let msgs: RwSignal<HashMap<u128, Rc<dyn Fn(ServerMsg)>>> = global_state.socket_closures;

//     msgs.with_untracked(move |msgs| {
//         let Some(f) = msgs.get(&id) else {
//             log!("Fn not found for {}", id);
//             return;
//         };

//         f(server_msg);
//     });
// }

#[component]
pub fn Gallery<
    OnClick: Fn(ServerMsgImgResized) + Copy + 'static,
    OnFetch: Fn(i64, u32) + Copy + 'static,
>(
    _global_gallery_imgs: RwSignal<Vec<ServerMsgImgResized>>,
    _on_click: OnClick,
    _on_fetch: OnFetch,
    _loaded_sig: RwSignal<LoadingNotFound>,
    _connection_load_state_name: &'static str,
) -> impl IntoView {
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");

    let _nav_tran = global_state.nav_tran;
    let _location = use_location();

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

    //execute(id);

    view! {
        "hello"
    }
}
