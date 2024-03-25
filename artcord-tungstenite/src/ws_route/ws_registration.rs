use artcord_mongodb::database::DB;
use artcord_state::model::acc::Acc;
use crate::message::server_msg::ServerMsg;
use crate::server::registration_invalid::{RegistrationInvalidMsg, BCRYPT_COST};
use crate::server::ws_connection::ServerMsgCreationError;
use rand::Rng;
use std::sync::Arc;

pub async fn ws_register(
    db: Arc<DB>,
    pepper: Arc<String>,
    email: String,
    password: String,
) -> Result<ServerMsg, ServerMsgCreationError> {
    let email_code: String = (0..25)
        .map(|_| char::from(rand::thread_rng().gen_range(32..127)))
        .collect();

    let (invalid, email_error, password_error) =
        RegistrationInvalidMsg::validate_registration(&email, &password);

    if invalid == true {
        return Ok(ServerMsg::RegistrationInvalid(RegistrationInvalidMsg {
            general_error: None,
            password_error,
            email_error,
        }));
    }

    let acc = db.acc_find_one(&email).await?;
    if let Some(acc) = acc {
        return Ok(ServerMsg::RegistrationInvalid(
            RegistrationInvalidMsg::new()
                .general(format!("Account with email '{}' already exists.", &email)),
        ));
    };

    let password = format!("{}{}", &password, &pepper);
    let password_hash = bcrypt::hash(&password, BCRYPT_COST);
    let Ok(password_hash) = password_hash else {
        return Err(ServerMsgCreationError::from(password_hash.err().unwrap()));
    };

    let acc = Acc::new(&email, &password_hash, &email_code);

    let result = db
        .acc_insert_one(acc)
        .await
        .and_then(|e| Ok(ServerMsg::RegistrationCompleted))
        .or_else(|e| Err(ServerMsgCreationError::from(e)))?;

    Ok(result)
}
