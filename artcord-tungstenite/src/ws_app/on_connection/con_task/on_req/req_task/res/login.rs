use artcord_mongodb::database::DB;
use artcord_state::{
    message::prod_server_msg::{LoginRes, ServerMsg},
    model::acc_session::AccSession,
};
// use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use rand::Rng;
use std::sync::Arc;

use crate::{ws_app::WsResError, WS_TOKEN_SIZE};

pub async fn ws_login(
    db: Arc<DB>,
    email: String,
    password: String,
    pepper: Arc<String>,
    jwt_secret: Arc<Vec<u8>>,
) -> Result<ServerMsg, WsResError> {
    println!("LOGIN '{}' '{}'", email, password);

    let acc = db.acc_find_one(&email).await?;
    let Some(acc) = acc else {
        return Ok(ServerMsg::Login(LoginRes::Err(
            "Invalid email or password.".to_string(),
        )));
    };

    let password = format!("{}{}", &password, &pepper);
    let good = bcrypt::verify(password, &acc.password)?;
    if good == false {
        return Ok(ServerMsg::Login(LoginRes::Err(
            "Invalid email or password.".to_string(),
        )));
    }

    let token: String = (0..WS_TOKEN_SIZE)
        .map(|_| char::from(rand::thread_rng().gen_range(32..127)))
        .collect();
    // let header = Header::new(Algorithm::HS512);
    // let token = encode(&header, &token, &EncodingKey::from_secret(&jwt_secret))?;

    let acc_session = AccSession::new(
        acc.id.clone(),
        "127.0.0.1".to_string(),
        "Firefox".to_string(),
        token.clone(),
    );
    db.acc_session_insert_one(acc_session).await?;

    Ok(ServerMsg::Login(LoginRes::Success {
        user_id: acc.id,
        token,
    }))
}
