use artcord_leptos_web_sockets::WsPackage;
use chrono::TimeDelta;
use enum_index_derive::EnumIndex;
use field_types::FieldName;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::{EnumCount, EnumIter, EnumString, IntoStaticStr, VariantArray, VariantNames};

use std::fmt::Display;
use std::net::IpAddr;
use std::time::Duration;

use crate::misc::throttle_connection::IpBanReason;
use crate::misc::throttle_threshold::Threshold;

use super::prod_perm_key::ProdMsgPermKey;










impl ClientMsg {
   

    pub fn throttle(
        &self,
        throttle_time: &mut HashMap<ClientPathType, (u64, HashMap<IpAddr, u64>)>,
        ip: &IpAddr,
        path: ClientPathType,
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
