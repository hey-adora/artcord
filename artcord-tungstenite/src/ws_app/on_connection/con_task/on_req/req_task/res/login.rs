use artcord_mongodb::database::DB;
use artcord_state::{
    message::prod_server_msg::{ServerMsg},
    model::acc_session::AccSession,
};
// use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use rand::Rng;
use std::sync::Arc;

use crate::{ws::WsResError, WS_TOKEN_SIZE};


