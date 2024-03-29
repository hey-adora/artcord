use crate::database::DB;
use artcord_state::model::user::{User, UserFieldName};
use bson::{doc, Document};
use mongodb::{options::IndexOptions, Collection, Database, IndexModel};

pub const COLLECTION_USER_NAME: &'static str = "user";

impl DB {
    pub async fn init_user(database: &Database) -> Collection<User> {
        database.collection::<User>(COLLECTION_USER_NAME)
    }

    pub async fn init_user_index(collection: &Collection<User>) {
        let opts = IndexOptions::builder().unique(true).build();
        let index = IndexModel::builder()
            .keys(doc! { UserFieldName::AuthorId.name(): -1 })
            .options(opts)
            .build();

        collection
            .create_index(index, None)
            .await
            .expect("Failed to create collection index.");
    }
}

impl DB {
    pub async fn user_insert_one(&self, user: User) -> Result<String, mongodb::error::Error> {
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
                doc! { UserFieldName::AuthorId.name(): user_id },
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
    ) -> Result<Option<User>, mongodb::error::Error> {
        let result = self
            .collection_user
            .find_one(doc! {UserFieldName::AuthorId.name(): user_id}, None)
            .await?;
        //println!("wtf{:?}", user);
        // Ok(ServerMsg::Profile(user))
        Ok(result)
    }
}

