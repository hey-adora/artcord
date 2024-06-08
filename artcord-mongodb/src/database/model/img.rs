use std::sync::Arc;

use crate::database::{COLLECTION_IMG_NAME, DB};
use artcord_state::global::{DbImg, DbImgFieldName};
use bson::{doc, Document};
use mongodb::{options::IndexOptions, Collection, Database, IndexModel};
use field_types::FieldName;
use serde::{Deserialize, Serialize};




impl DB {
    pub async fn init_img(database: &Database) -> Collection<DbImg> {
        let opts = IndexOptions::builder().unique(true).build();
        let index = IndexModel::builder()
            .keys(doc! { DbImgFieldName::Id.name(): -1 })
            .options(opts)
            .build();

        let collection = database.collection::<DbImg>(&COLLECTION_IMG_NAME);

        collection
            .create_index(index, None)
            .await
            .expect("Failed to create collection index.");

        collection
    }
}

impl DB {
    pub async fn img_find_one(
        &self,
        guild_id: u64,
        file_hash: &str,
    ) -> Result<Option<DbImg>, mongodb::error::Error> {
        let found_img = self
            .collection_img
            .find_one(
                doc! {
                    DbImgFieldName::GuildId.name(): guild_id.to_string(),
                    DbImgFieldName::OrgHash.name(): file_hash
                },
                None,
            )
            .await?;
        Ok(found_img)
    }

    pub async fn img_hide(
        &self,
        guild_id: u64,
        msg_id: u64,
    ) -> Result<bool, mongodb::error::Error> {
        let result = self
            .collection_img
            .update_one(
                doc! { DbImgFieldName::GuildId.name(): guild_id.to_string(), DbImgFieldName::MsgId.name(): msg_id.to_string() },
                doc! { "$set": { DbImgFieldName::Show.name(): false } },
                None,
            )
            .await?;

        Ok(result.matched_count > 0)
    }

    pub async fn img_update_one_by_hash(
        &self,
        guild_id: u64,
        file_hash: &str,
        update: Document,
    ) -> Result<(), mongodb::error::Error> {
        self.collection_img
            .update_one(
                doc! { DbImgFieldName::GuildId.name(): guild_id.to_string(), DbImgFieldName::OrgHash.name(): file_hash },
                doc! {
                    "$set": update
                },
                None,
            )
            .await?;

        Ok(())
    }

    pub async fn img_insert(&self, img: &DbImg) -> Result<(), mongodb::error::Error> {
        let is_ms = img.created_at < 9999999999999;
        if !is_ms {
            return Err(mongodb::error::Error::custom(Arc::new(format!(
                "{}: {}",
                DbImgFieldName::CreatedAt.name(),
                is_ms
            ))));
        }
        let is_ms = img.modified_at < 9999999999999;
        if !is_ms {
            return Err(mongodb::error::Error::custom(Arc::new(format!(
                "{}: {}",
                DbImgFieldName::ModifiedAt.name(),
                is_ms
            ))));
        }
        self.collection_img.insert_one(img, None).await?;
        Ok(())
    }
}
