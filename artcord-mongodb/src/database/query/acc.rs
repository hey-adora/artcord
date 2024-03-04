use crate::database::DB;
use artcord_state::model::acc::{Acc, AccFieldName};
use bson::doc;
use mongodb::{options::IndexOptions, Collection, Database, IndexModel};

const COLLECTION_ACC_NAME: &'static str = "acc";

impl DB {
    pub async fn init_acc(database: &Database) -> Collection<Acc> {
        let opts = IndexOptions::builder().unique(true).build();
        let index = IndexModel::builder()
            .keys(doc! { AccFieldName::Email.name(): -1 })
            .options(opts)
            .build();
        
        let collection_acc = database.collection::<Acc>(COLLECTION_ACC_NAME);
        collection_acc
            .create_index(index, None)
            .await
            .expect("Failed to create collection index.");

        collection_acc
    }
}

impl DB {
    pub async fn acc_find_one_by_id(&self, id: &str) -> Result<Option<Acc>, mongodb::error::Error> {
        let acc = self
            .collection_acc
            .find_one(doc! { AccFieldName::Id.name(): id }, None)
            .await?;

        Ok(acc)
    }
    pub async fn acc_find_one(&self, email: &str) -> Result<Option<Acc>, mongodb::error::Error> {
        let acc = self
            .collection_acc
            .find_one(doc! { AccFieldName::Email.name(): email }, None)
            .await?;

        Ok(acc)
    }
    pub async fn acc_insert_one(&self, acc: Acc) -> Result<String, mongodb::error::Error> {
        // let acc = self
        //     .collection_acc
        //     .find_one(doc! { "email": email }, None)
        //     .await?;
        // if let Some(_) = acc {
        //     return Err(mongodb::error::Error::custom(Arc::new(format!(
        //         "Email '{}' is already registered.",
        //         email
        //     ))));
        // }

        let result = self.collection_acc.insert_one(acc, None).await?;

        //Err(mongodb::error::Error::custom(Arc::new("invalid ReactionType type".to_string())) )

        Ok(result.inserted_id.to_string())
    }
}