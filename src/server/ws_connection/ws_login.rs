use crate::database::create_database::DB;
use crate::database::models::acc_session::AccSession;
use crate::message::server_msg::ServerMsg;
use crate::server::create_server::TOKEN_SIZE;
use crate::server::ws_connection::ServerMsgCreationError;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use rand::Rng;
use std::sync::Arc;

pub async fn ws_login(
    db: Arc<DB>,
    email: String,
    password: String,
    pepper: Arc<String>,
    jwt_secret: Arc<Vec<u8>>,
) -> Result<ServerMsg, ServerMsgCreationError> {
    println!("LOGIN '{}' '{}'", email, password);

    let acc = db.acc_find_one(&email).await?;
    let Some(acc) = acc else {
        return Ok(ServerMsg::LoginInvalid(
            "Invalid email or password.".to_string(),
        ));
    };

    let password = format!("{}{}", &password, &pepper);
    let good = bcrypt::verify(password, &acc.password)?;
    if good == false {
        return Ok(ServerMsg::LoginInvalid(
            "Invalid email or password.".to_string(),
        ));
    }

    let token: String = (0..TOKEN_SIZE)
        .map(|_| char::from(rand::thread_rng().gen_range(32..127)))
        .collect();
    let header = Header::new(Algorithm::HS512);
    let token = encode(&header, &token, &EncodingKey::from_secret(&jwt_secret))?;

    let acc_session = crate::database::models::acc_session::AccSession::new(
        acc.id,
        "127.0.0.1".to_string(),
        "Firefox".to_string(),
        token.clone(),
    );
    db.acc_session_insert_one(acc_session).await?;

    Ok(ServerMsg::LoginComplete {
        token,
        user_id: acc.email,
    })
}
