use artcord_mongodb::database::DB;
use artcord_state::global;
use chrono::{DateTime, Utc};
use rand::Rng;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::BCRYPT_COST;

use super::ResErr;

pub async fn login(
    db: &DB,
    email: &str,
    password: &str,
    pepper: &str,
    jwt_secret: &Vec<u8>,
    time: &DateTime<Utc>,
) -> Result<global::ServerMsg, ResErr> {
    println!("LOGIN '{}' '{}'", email, password);

    let acc = db.acc_find_one(&email).await?;
    let Some(acc) = acc else {
        return Ok(global::ServerMsg::LoginErr(
            "Invalid email or password.".to_string(),
        ));
    };

    let password = format!("{}{}", &password, &pepper);
    let good = bcrypt::verify(password, &acc.password)?;
    if good == false {
        return Ok(global::ServerMsg::LoginErr(
            "Invalid email or password.".to_string(),
        ));
    }

    let token: String = (0..69)
        .map(|_| char::from(rand::thread_rng().gen_range(32..127)))
        .collect();
    // let header = Header::new(Algorithm::HS512);
    // let token = encode(&header, &token, &EncodingKey::from_secret(&jwt_secret))?;

    let acc_session = global::DbAccSession::new(
        acc.id.clone(),
        "127.0.0.1".to_string(),
        "Firefox".to_string(),
        token.clone(),
        time,
    );
    db.acc_session_insert_one(acc_session).await?;

    Ok(global::ServerMsg::LoginSuccess {
        user_id: acc.id,
        token,
    })
}

pub async fn register(
    db: &DB,
    pepper: &str,
    email: &str,
    password: &str,
    time: &DateTime<Utc>,
) -> Result<global::ServerMsg, ResErr> {
    let email_code: String = (0..25)
        .map(|_| char::from(rand::thread_rng().gen_range(32..127)))
        .collect();

    let (invalid, email_error, password_error) =
        global::RegistrationInvalidMsg::validate_registration(&email, &password);

    if invalid == true {
        return Ok(global::ServerMsg::RegistrationErr(
            global::RegistrationInvalidMsg {
                general_error: None,
                password_error,
                email_error,
            },
        ));
    }

    let acc = db.acc_find_one(&email).await?;
    if let Some(acc) = acc {
        return Ok(global::ServerMsg::RegistrationErr(
            global::RegistrationInvalidMsg::new()
                .general(format!("Account with email '{}' already exists.", &email)),
        ));
    };

    let password = format!("{}{}", &password, &pepper);
    let password_hash = bcrypt::hash(&password, BCRYPT_COST)?;
    // let Ok(password_hash) = password_hash else {
    //     return Err(::from(password_hash.err().unwrap()));
    // };

    let acc = global::DbAcc::new(&email, &password_hash, &email_code, time);

    let result = db
        .acc_insert_one(acc)
        .await
        .and_then(|e| Ok(global::ServerMsg::RegistrationSuccess))?;
    // .or_else(|e| Err(ServerMsgCreationError::from(e)))?;

    Ok(result)
}

// pub async fn logout(acc: Arc<RwLock<Option<global::DbAcc>>>) -> Result<global::ServerMsg, ResErr> {
//     let mut acc = acc.write().await;

//     *acc = None;

//     Ok(global::ServerMsg::LoggedOut)
// }
