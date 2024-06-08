use chrono::{DateTime, TimeDelta, TimeZone, Utc};
use field_types::FieldName;
use leptos::RwSignal;
use serde::de::value;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::net::SocketAddr;
use std::num::TryFromIntError;
use std::sync::mpsc;
use std::{collections::HashMap, fmt::Debug, str::FromStr};
use strum::IntoEnumIterator;
use strum::VariantNames;
use thiserror::Error;
use tracing::error;

use crate::message::prod_client_msg::ClientMsg;
use crate::message::prod_client_msg::ClientPathType;
use crate::misc::throttle_connection::IpBanReason;
use crate::misc::throttle_threshold::{
    AllowCon, DbThrottleDoubleLayer, DbThrottleDoubleLayerFromError, Threshold,
    ThrottleDoubleLayer, ThrottleDoubleLayerFromError,
};

pub type TempConIdType = u128;



