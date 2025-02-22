use crate::database::{COLLECTION_ACC_NAME, DB};
use artcord_state::global::{DbAcc, DbAccFieldName};
use bson::doc;
use mongodb::{options::IndexOptions, Collection, Database, IndexModel};

impl DB {
    pub async fn init_acc(database: &Database) -> Collection<DbAcc> {
        let opts = IndexOptions::builder().unique(true).build();
        let index = IndexModel::builder()
            .keys(doc! { DbAccFieldName::Email.name(): -1 })
            .options(opts)
            .build();

        let collection_acc = database.collection::<DbAcc>(&COLLECTION_ACC_NAME);
        collection_acc
            .create_index(index, None)
            .await
            .expect("Failed to create collection index.");

        collection_acc
    }
}

impl DB {
    pub async fn acc_find_one_by_id(
        &self,
        id: &str,
    ) -> Result<Option<DbAcc>, mongodb::error::Error> {
        let acc = self
            .collection_acc
            .find_one(doc! { DbAccFieldName::Id.name(): id }, None)
            .await?;

        Ok(acc)
    }
    pub async fn acc_find_one(&self, email: &str) -> Result<Option<DbAcc>, mongodb::error::Error> {
        let acc = self
            .collection_acc
            .find_one(doc! { DbAccFieldName::Email.name(): email }, None)
            .await?;

        Ok(acc)
    }
    pub async fn acc_insert_one(&self, acc: DbAcc) -> Result<String, mongodb::error::Error> {
        let result = self.collection_acc.insert_one(acc, None).await?;

        Ok(result.inserted_id.to_string())
    }
}
