use crate::database::DT;
use crate::server::server_msg::WebSerializeError;
use bson::DateTime;
use chrono::Utc;
use rkyv::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;

#[derive(rkyv::Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
#[archive(compare(PartialEq), check_bytes)]
#[archive_attr(derive(Debug))]
pub enum ClientMsg {
    GalleryInit {
        amount: u32,

        #[with(DT)]
        from: DateTime,
    },

    UserGalleryInit {
        amount: u32,

        #[with(DT)]
        from: DateTime,

        user_id: String,
    },

    User {
        user_id: String,
    },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum WsPath {
    Gallery,
    UserGallery,
    User,
}

impl WsPath {
    pub fn to_ms(&self) -> u64 {
        match self {
            WsPath::Gallery => 60 * 1000,
            WsPath::UserGallery => 60 * 1000,
            WsPath::User => 60 * 1000,
        }
    }

    pub fn to_count(&self) -> u64 {
        match self {
            WsPath::Gallery => 10,
            WsPath::UserGallery => 10,
            WsPath::User => 10,
        }
    }
}

impl Into<WsPath> for ClientMsg {
    fn into(self) -> WsPath {
        match self {
            ClientMsg::GalleryInit { amount, from } => WsPath::Gallery,
            ClientMsg::UserGalleryInit {
                from,
                amount,
                user_id,
            } => WsPath::UserGallery,
            ClientMsg::User { user_id } => WsPath::User,
        }
    }
}

impl From<&ClientMsg> for WsPath {
    fn from(value: &ClientMsg) -> Self {
        match value {
            ClientMsg::GalleryInit { amount, from } => WsPath::Gallery,
            ClientMsg::UserGalleryInit {
                from,
                amount,
                user_id,
            } => WsPath::UserGallery,
            ClientMsg::User { user_id } => WsPath::User,
        }
    }
}

impl ClientMsg {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, WebSerializeError> {
        let server_msg: Self = rkyv::check_archived_root::<Self>(bytes)
            .or_else(|e| {
                Err(WebSerializeError::InvalidBytes(format!(
                    "check_archived_root failed: {}",
                    e
                )))
            })?
            .deserialize(&mut rkyv::Infallible)
            .or_else(|e| {
                Err(WebSerializeError::InvalidBytes(format!(
                    "deserialize failed: {:?}",
                    e
                )))
            })?;

        Ok(server_msg)
    }

    pub fn throttle(
        &self,
        throttle_time: &mut HashMap<WsPath, (u64, HashMap<IpAddr, u64>)>,
        ip: &IpAddr,
        current_time: i64,
        duration: i64,
        max_count: u64,
    ) -> bool {
        let path: WsPath = self.into();
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

#[cfg(test)]
mod ClientMsgTests {
    use crate::server::client_msg::{ClientMsg, WsPath};
    use bson::DateTime;
    use chrono::Utc;
    use std::cell::{Cell, RefCell};
    use std::collections::HashMap;
    use std::net::{IpAddr, Ipv4Addr};
    use std::rc::Rc;

    #[test]
    fn msg_throttle() {
        let mut current_time = Rc::new(RefCell::new(Utc::now().timestamp_millis()));
        let duration = 60 * 1000;
        let msg = Rc::new(RefCell::new(ClientMsg::GalleryInit {
            amount: 10,
            from: DateTime::from_millis(*current_time.borrow()),
        }));

        let max_count = 10;
        let mut throttle_times: Rc<RefCell<HashMap<WsPath, (u64, HashMap<IpAddr, u64>)>>> =
            Rc::new(RefCell::new(HashMap::new()));
        let mut ip = IpAddr::from(Ipv4Addr::new(127, 0, 0, 1));

        //let result = msg.throttle(&mut throttle_times, &ip, time);

        // assert!(result == false, "Expected throttle to be false.");
        //
        // let (ms, clients) = throttle_times.get(&path).expect(&format!("Expected hashmap to be created with {:?} key.", path));
        //
        // let count = clients.get(&ip).expect(&format!("Expected hashmap with {:?} key.", ip));

        //assert!(*count == 1, "Expected count to be 1.");
        let mut check = |start: u64, state: bool, check_index: bool| {
            for i in start..=max_count {
                let throttle_times = &mut *throttle_times.borrow_mut();
                let msg = msg.borrow();
                let path: WsPath = msg.clone().into();

                let result = msg.throttle(
                    throttle_times,
                    &ip,
                    *current_time.borrow(),
                    duration,
                    max_count,
                );
                assert!(result == state, "Expected throttle to be {}.", state);

                let (ms, clients) = throttle_times.get(&path).expect(&format!(
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
