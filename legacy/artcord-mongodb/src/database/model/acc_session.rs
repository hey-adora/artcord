use artcord_state::global::{DbAccSession, DbAccSessionFieldName};
use bson::doc;
use mongodb::{Collection, Database};

use crate::database::{COLLECTION_ACC_SESSION_NAME, DB};

impl DB {
    pub async fn init_acc_session(database: &Database) -> Collection<DbAccSession> {
        database.collection::<DbAccSession>(&COLLECTION_ACC_SESSION_NAME)
    }
}

impl DB {
    pub async fn acc_session_find_one(
        &self,
        token: &str,
    ) -> Result<Option<DbAccSession>, mongodb::error::Error> {
        let acc = self
            .collection_acc_session
            .find_one(doc! { DbAccSessionFieldName::Token.name(): token }, None)
            .await?;

        Ok(acc)
    }
    pub async fn acc_session_insert_one(
        &self,
        acc_session: DbAccSession,
    ) -> Result<String, mongodb::error::Error> {
        let acc = self
            .collection_acc_session
            .insert_one(acc_session, None)
            .await?;

        Ok(acc.inserted_id.to_string())
    }
}
