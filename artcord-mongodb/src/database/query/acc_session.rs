use artcord_state::model::acc_session::{AccSession, AccSessionFieldName};
use bson::doc;
use mongodb::{Collection, Database};

use crate::database::DB;

const COLLECTION_ACC_SESSION_NAME: &'static str = "acc_session";

impl DB {
    pub async fn init_acc_session(database: &Database) -> Collection<AccSession> {
        database.collection::<AccSession>(COLLECTION_ACC_SESSION_NAME)
    }
}

impl DB {
    pub async fn acc_session_find_one(
        &self,
        token: &str,
    ) -> Result<Option<AccSession>, mongodb::error::Error> {
        let acc = self
            .collection_acc_session
            .find_one(doc! { AccSessionFieldName::Token.name(): token }, None)
            .await?;

        Ok(acc)
    }
    pub async fn acc_session_insert_one(
        &self,
        acc_session: AccSession,
    ) -> Result<String, mongodb::error::Error> {
        let acc = self
            .collection_acc_session
            .insert_one(acc_session, None)
            .await?;

        Ok(acc.inserted_id.to_string())
    }

  

}