use crate::database::{COLLECTION_USER_NAME, DB};
use artcord_state::global::{DbUser, DbUserFieldName};
use bson::{doc, Document};
use mongodb::{options::IndexOptions, Collection, Database, IndexModel};
use field_types::FieldName;
use serde::{Deserialize, Serialize};





impl DB {
    pub async fn init_user(database: &Database) -> Collection<DbUser> {
        let (index1, index2) = (
            {
                let opts = IndexOptions::builder().unique(true).build();
                IndexModel::builder()
                    .keys(doc! { DbUserFieldName::Id.name(): -1 })
                    .options(opts)
                    .build()
            },
            {
                let opts = IndexOptions::builder().unique(true).build();
                IndexModel::builder()
                    .keys(doc! { DbUserFieldName::AuthorId.name(): -1 })
                    .options(opts)
                    .build()
            },
        );

        let collection = database.collection::<DbUser>(&COLLECTION_USER_NAME);

        collection
            .create_indexes([index1, index2], None)
            .await
            .expect("Failed to create collection index.");

        collection
    }
}

impl DB {
    pub async fn user_insert_one(&self, user: DbUser) -> Result<String, mongodb::error::Error> {
        let result = self.collection_user.insert_one(user, None).await?;

        Ok(result.inserted_id.to_string())
    }

    pub async fn user_update_one_raw(
        &self,
        user_id: &str,
        update: Document,
    ) -> Result<(), mongodb::error::Error> {
        self.collection_user
            .update_one(
                doc! { DbUserFieldName::AuthorId.name(): user_id },
                doc! {
                    "$set": update.clone()
                },
                None,
            )
            .await?;

        Ok(())
    }

    pub async fn user_find_one(
        &self,
        user_id: &str,
    ) -> Result<Option<DbUser>, mongodb::error::Error> {
        let result = self
            .collection_user
            .find_one(doc! {DbUserFieldName::AuthorId.name(): user_id}, None)
            .await?;
        //println!("wtf{:?}", user);
        // Ok(ServerMsg::Profile(user))
        Ok(result)
    }
}
