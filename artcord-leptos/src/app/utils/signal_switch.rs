// use std::{any::Any, collections::HashMap, rc::Rc};

// use leptos::{provide_context, use_context, RwSignal, StoredValue};
// use tracing::debug;

// #[derive(Copy, Clone, Debug)]
// pub struct SignalSwitchState {
//     pub prev_signals: StoredValue<HashMap<u128, Box<dyn Any>>>
// }

// impl SignalSwitchState {
//     pub fn new() -> Self {
//         Self {
//             prev_signals: StoredValue::new(HashMap::new())
//         }
//     }
// }

// pub fn signal_switch_init() {
//     provide_context(SignalSwitchState::new());
// }

// pub fn signal_switch<T>(condition: bool, current_signal: &RwSignal<T>, switch_to: T) {
//     let global_state = use_context::<SignalSwitchState>().expect("Failed to provide switch signal state");
//     let ptr: *const RwSignal<T> = current_signal;
//     let ptr = ptr as usize;
//     debug!("PTR: {}", ptr);
//     //let points_at = unsafe { *ptr };
//     // global_state.prev_signals.update_value(|prev_signals| {
//     //     let current_signal = Box::new(current_signal);
        
//     //   //  prev_signals.insert(0, current_signal);
//     // });
//    // let 
// }