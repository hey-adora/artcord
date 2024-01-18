use crate::database::create_database::DB;
use crate::database::models::acc::Acc;
use crate::server::registration_invalid::{RegistrationInvalidMsg, BCRYPT_COST};
use crate::server::server_msg::ServerMsg;
use crate::server::ws_connection::ServerMsgCreationError;
use rand::Rng;
use std::sync::Arc;

pub async fn ws_register(
    db: Arc<DB>,
    pepper: Arc<String>,
    email: String,
    password: String,
) -> Result<ServerMsg, ServerMsgCreationError> {
    println!("2");
    // let salt: String = (0..256)
    //     .map(|_| char::from(rand::thread_rng().gen_range(32..127)))
    //     .collect();
    let email_code: String = (0..25)
        .map(|_| char::from(rand::thread_rng().gen_range(32..127)))
        .collect();
    println!("3");

    let (invalid, email_error, password_error) =
        RegistrationInvalidMsg::validate_registration(&email, &password);

    if invalid == true {
        println!(
            "INVALID: {} {:?} {:?}",
            invalid, email_error, password_error
        );
        return Ok(ServerMsg::None);
    }

    let acc = db.acc_find_one(&email).await?;
    if let Some(acc) = acc {
        println!("Acc already exists.");
        return Ok(ServerMsg::None);
    };

    let password = format!("{}{}", &password, &pepper);
    let password_hash = bcrypt::hash(&password, BCRYPT_COST);
    let Ok(password_hash) = password_hash else {
        return Err(ServerMsgCreationError::from(password_hash.err().unwrap()));
    };

    let acc = Acc::new(&email, &password_hash, &email_code);

    let result = db
        .create_acc(acc)
        .await
        .and_then(|e| Ok(ServerMsg::RegistrationCompleted))
        .or_else(|e| Err(ServerMsgCreationError::from(e)))?;

    Ok(result)
}
