use artcord_mongodb::database::DB;
use artcord_state::{
    message::prod_server_msg::{ServerMsg},
    misc::registration_invalid::{RegistrationInvalidMsg, BCRYPT_COST},
    model::acc::Acc,
};
use rand::Rng;
use std::sync::Arc;

use crate::ws_app::WsResError;

pub async fn ws_register(
    db: Arc<DB>,
    pepper: Arc<String>,
    email: String,
    password: String,
) -> Result<ServerMsg, WsResError> {
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
