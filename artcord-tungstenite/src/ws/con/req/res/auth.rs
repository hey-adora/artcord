use artcord_mongodb::database::DB;
use artcord_state::{
    message::prod_server_msg::ServerMsg,
    misc::registration_invalid::{RegistrationInvalidMsg, BCRYPT_COST},
    model::{acc::Acc, acc_session::AccSession},
};
use rand::Rng;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::WS_TOKEN_SIZE;

use super::ResErr;

pub async fn login(
    db: Arc<DB>,
    email: String,
    password: String,
    pepper: Arc<String>,
    jwt_secret: Arc<Vec<u8>>,
) -> Result<ServerMsg, ResErr> {
    println!("LOGIN '{}' '{}'", email, password);

    let acc = db.acc_find_one(&email).await?;
    let Some(acc) = acc else {
        return Ok(ServerMsg::LoginErr(
            "Invalid email or password.".to_string(),
        ));
    };

    let password = format!("{}{}", &password, &pepper);
    let good = bcrypt::verify(password, &acc.password)?;
    if good == false {
        return Ok(ServerMsg::LoginErr(
            "Invalid email or password.".to_string(),
        ));
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

    Ok(ServerMsg::LoginSuccess {
        user_id: acc.id,
        token,
    })
}

pub async fn register(
    db: Arc<DB>,
    pepper: Arc<String>,
    email: String,
    password: String,
) -> Result<ServerMsg, ResErr> {
    let email_code: String = (0..25)
        .map(|_| char::from(rand::thread_rng().gen_range(32..127)))
        .collect();

    let (invalid, email_error, password_error) =
        RegistrationInvalidMsg::validate_registration(&email, &password);

    if invalid == true {
        return Ok(ServerMsg::RegistrationErr(
            RegistrationInvalidMsg {
                general_error: None,
                password_error,
                email_error,
            },
        ));
    }

    let acc = db.acc_find_one(&email).await?;
    if let Some(acc) = acc {
        return Ok(ServerMsg::RegistrationErr(
            RegistrationInvalidMsg::new()
                .general(format!("Account with email '{}' already exists.", &email)),
        ));
    };

    let password = format!("{}{}", &password, &pepper);
    let password_hash = bcrypt::hash(&password, BCRYPT_COST)?;
    // let Ok(password_hash) = password_hash else {
    //     return Err(::from(password_hash.err().unwrap()));
    // };

    let acc = Acc::new(&email, &password_hash, &email_code);

    let result = db
        .acc_insert_one(acc)
        .await
        .and_then(|e| Ok(ServerMsg::RegistrationSuccess))?;
    // .or_else(|e| Err(ServerMsgCreationError::from(e)))?;

    Ok(result)
}

pub async fn logout(acc: Arc<RwLock<Option<Acc>>>) -> Result<ServerMsg, ResErr> {
    let mut acc = acc.write().await;

    *acc = None;

    Ok(ServerMsg::LoggedOut)
}
