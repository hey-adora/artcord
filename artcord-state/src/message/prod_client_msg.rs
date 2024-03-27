//use crate::database::rkw::date_time::DT;
// use crate::message::server_msg::{
//     SERVER_MSG_IMGS_NAME, SERVER_MSG_LOGIN, SERVER_MSG_PROFILE, SERVER_MSG_PROFILE_IMGS_NAME,
//     SERVER_MSG_REGISTRATION,
// };


use artcord_leptos_web_sockets::{WsPackage, WsRouteKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use std::net::IpAddr;
use std::time::Duration;

use super::prod_perm_key::ProdMsgPermKey;


#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub enum ClientMsg {
    GalleryInit {
        amount: u32,

        from: i64,
    },

    UserGalleryInit {
        amount: u32,

        from: i64,

        user_id: String,
    },

    User {
        user_id: String,
    },

    Login {
        email: String,
        password: String,
    },
    Logout,
    Register {
        email: String,
        password: String,
    },
}

impl artcord_leptos_web_sockets::Send<u128, ProdMsgPermKey> for ClientMsg {
    fn send_as_vec(
            package: &WsPackage<u128, ProdMsgPermKey, Self>,
        ) -> Result<Vec<u8>, String> where
        Self: Clone {
            Self::as_vec(package).map_err(|e| e.to_string())
    }
}

impl ClientMsg {
    // pub fn name(&self) -> &'static str {
    //     match self {
    //         ClientMsg::GalleryInit { amount: _, from: _ } => SERVER_MSG_IMGS_NAME,
    //         ClientMsg::UserGalleryInit {
    //             from: _,
    //             amount: _,
    //             user_id: _,
    //         } => SERVER_MSG_PROFILE_IMGS_NAME,
    //         ClientMsg::User { user_id: _ } => SERVER_MSG_PROFILE,
    //         ClientMsg::Register { email: _, password: _ } => SERVER_MSG_REGISTRATION,
    //         ClientMsg::Login { email: _, password: _ } => SERVER_MSG_LOGIN,
    //         ClientMsg::Logout => SERVER_MSG_LOGIN,
    //     }
    // }

    pub fn as_vec(package: &WsPackage<u128, ProdMsgPermKey, Self>) -> Result<Vec<u8>, bincode::Error> {
        let a = bincode::serialize::<WsPackage<u128, ProdMsgPermKey, Self>>(package);
        //log!("SERIALIZE {:?} {:?}", self, a);
        a
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<WsPackage<u128, ProdMsgPermKey, Self>, bincode::Error> {
        //log!("DESERIALIZE {:?}", bytes);
        let a = bincode::deserialize::<WsPackage<u128, ProdMsgPermKey, Self>>(bytes);
        a
    }

    pub fn throttle(
        &self,
        throttle_time: &mut HashMap<WsPath, (u64, HashMap<IpAddr, u64>)>,
        ip: &IpAddr,
        path: WsPath,
        current_time: i64,
        duration: i64,
        max_count: u64,
    ) -> bool {
        //println!("DEBUG: {:?}", &path);
        //println!("HASHMAP: {:?}", &throttle_time);

        //println!("ONE ONE");
        let Some((ref mut ms, ref mut clients)) = throttle_time.get_mut(&path) else {
            //println!("TWO TWO");
            let mut clients: HashMap<IpAddr, u64> = HashMap::new();
            clients.insert(ip.clone(), 1);
            throttle_time.insert(path, (current_time as u64, clients));
            //println!("TWO TWO HASHMAP: {:?}", &throttle_time);
            return false;
        };

        let Some(count) = clients.get_mut(ip) else {
            clients.insert(ip.clone(), 1);
            return false;
        };

        if *ms + (duration as u64) <= current_time as u64 {
            *ms = current_time as u64;
            *count = 1;
            return false;
        }

        if *count + 1 > max_count {
            return true;
        }

        *count += 1;

        false
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum WsPath {
    Gallery,
    UserGallery,
    User,
    Login,
    Register,
    Logout,
}

// #[derive(Clone, PartialEq, Eq, Debug, Hash)]
// pub struct Throttle {
//     pub max_connections: u64,
//     pub interval: Duration,
// }

// impl Throttle {
//     pub fn new() -> Self {

//     }
// }

impl WsPath {
    pub fn get_throttle(&self) -> (u64, Duration) {
        match self {
            WsPath::Gallery => (100, Duration::from_secs(1)),
            WsPath::UserGallery => (100, Duration::from_secs(1)),
            WsPath::User => (100, Duration::from_secs(1)),
            WsPath::Login => (100, Duration::from_secs(1)),
            WsPath::Register => (100, Duration::from_secs(1)),
            WsPath::Logout => (1, Duration::from_secs(30)),
        }
    }

    // pub fn to_ms(&self) -> Duration {
    //     match self {
    //         WsPath::Gallery => 60 * 1000,
    //         WsPath::UserGallery => 60 * 1000,
    //         WsPath::User => 60 * 1000,
    //         WsPath::Login => 60 * 1000,
    //         WsPath::Register => 60 * 1000,
    //         WsPath::Logout => 60 * 1000,
    //     }
    // }

    // pub fn to_count(&self) -> u64 {
    //     match self {
    //         WsPath::Gallery => 6000,
    //         WsPath::UserGallery => 6000,
    //         WsPath::User => 6000,
    //         WsPath::Login => 10,
    //         WsPath::Register => 10,
    //         WsPath::Logout => 10,
    //     }
    // }
}

// impl Into<WsPath> for ClientMsg {
//     fn into(self) -> WsPath {
//         match self {
//             ClientMsg::GalleryInit { amount, from } => WsPath::Gallery,
//             ClientMsg::UserGalleryInit {
//                 from,
//                 amount,
//                 user_id,
//             } => WsPath::UserGallery,
//             ClientMsg::User { user_id } => WsPath::User,
//         }
//     }
// }

impl From<&ClientMsg> for WsPath {
    fn from(value: &ClientMsg) -> Self {
        match value {
            ClientMsg::GalleryInit { amount: _, from: _ } => WsPath::Gallery,
            ClientMsg::UserGalleryInit {
                from: _,
                amount: _,
                user_id: _,
            } => WsPath::UserGallery,
            ClientMsg::User { user_id: _ } => WsPath::User,
            ClientMsg::Login { email: _, password: _ } => WsPath::Login,
            ClientMsg::Register { email: _, password: _ } => WsPath::Register,
            ClientMsg::Logout => WsPath::Logout,
        }
    }
}

#[cfg(test)]
mod client_msg_tests {
    
    use chrono::Utc;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::net::{IpAddr, Ipv4Addr};
    use std::rc::Rc;

    use super::{ClientMsg, WsPath};

    #[test]
    fn msg_throttle() {
        let current_time = Rc::new(RefCell::new(Utc::now().timestamp_millis()));
        let duration = 60 * 1000;
        let msg = Rc::new(RefCell::new(ClientMsg::GalleryInit {
            amount: 10,
            from: *current_time.borrow(),
        }));

        let max_count = 10;
        let throttle_times: Rc<RefCell<HashMap<WsPath, (u64, HashMap<IpAddr, u64>)>>> =
            Rc::new(RefCell::new(HashMap::new()));
        let ip = IpAddr::from(Ipv4Addr::new(127, 0, 0, 1));

        //let result = msg.throttle(&mut throttle_times, &ip, time);

        // assert!(result == false, "Expected throttle to be false.");
        //
        // let (ms, clients) = throttle_times.get(&path).expect(&format!("Expected hashmap to be created with {:?} key.", path));
        //
        // let count = clients.get(&ip).expect(&format!("Expected hashmap with {:?} key.", ip));

        //assert!(*count == 1, "Expected count to be 1.");
        let check = |start: u64, state: bool, check_index: bool| {
            for i in start..=max_count {
                let throttle_times = &mut *throttle_times.borrow_mut();
                let msg = msg.borrow();
                let path: WsPath = (&*msg).into();

                let result = msg.throttle(
                    throttle_times,
                    &ip,
                    path,
                    *current_time.borrow(),
                    duration,
                    max_count,
                );
                assert!(result == state, "Expected throttle to be {}.", state);

                let (_ms, clients) = throttle_times.get(&path).expect(&format!(
                    "Expected hashmap to be created with {:?} key.",
                    path
                ));
                let count = clients
                    .get(&ip)
                    .expect(&format!("Expected hashmap with {:?} key.", ip));

                if check_index {
                    assert!(
                        i == *count,
                        "Expected count to be equal a = {} == b = {}",
                        i,
                        *count
                    );
                } else {
                    assert!(
                        max_count == *count,
                        "Expected count to be equal a = {} == b = {}",
                        max_count,
                        *count
                    );
                }
            }
        };

        check(1, false, true);
        check(0, true, false);

        {
            let mut current_time = current_time.borrow_mut();
            *current_time = *current_time + duration;
        }

        check(1, false, true);
        check(0, true, false);

        {
            let mut msg = msg.borrow_mut();
            *msg = ClientMsg::User {
                user_id: "10".to_string(),
            };
        }

        check(1, false, true);
        check(0, true, false);
    }
}
