use artcord_state::global::{DbAllowedChannel, DbAllowedChannelFieldName};
use bson::doc;
use futures::TryStreamExt;
use mongodb::{options::IndexOptions, Collection, Database, IndexModel};
use field_types::FieldName;
use serde::{Deserialize, Serialize};

use crate::database::{COLLECTION_ALLOWED_CHANNEL_NAME, DB};

impl DB {
    pub async fn init_allowed_channel(database: &Database) -> Collection<DbAllowedChannel> {
        let opts = IndexOptions::builder().unique(true).build();
        let index = IndexModel::builder()
            .keys(doc! { DbAllowedChannelFieldName::GuildId.name(): -1, DbAllowedChannelFieldName::ChannelId.name(): -1, DbAllowedChannelFieldName::Feature.name(): -1 })
            .options(opts)
            .build();

        let collection = database.collection::<DbAllowedChannel>(&COLLECTION_ALLOWED_CHANNEL_NAME);

        collection
            .create_index(index, None)
            .await
            .expect("Failed to create collection index.");

        collection
    }
}

impl DB {
    pub async fn allowed_channel_exists(
        &self,
        guild_id: u64,
        channel_id: u64,
        feature: &str,
    ) -> Result<bool, mongodb::error::Error> {
        let channel = self
            .collection_allowed_channel
            .find_one(
                doc! { DbAllowedChannelFieldName::GuildId.name(): guild_id.to_string(), DbAllowedChannelFieldName::ChannelId.name(): channel_id.to_string(), DbAllowedChannelFieldName::Feature.name(): feature.to_string() },
                None,
            )
            .await?;
        Ok(channel.is_some())
    }

    pub async fn allowed_channel_insert_one(
        &self,
        allowed_channel: DbAllowedChannel,
    ) -> Result<String, mongodb::error::Error> {
        let result = self
            .collection_allowed_channel
            .insert_one(allowed_channel, None)
            .await?;

        Ok(result.inserted_id.to_string())
    }

    pub async fn allowed_channel_find_all(
        &self,
        guild_id: &str,
    ) -> Result<Vec<DbAllowedChannel>, mongodb::error::Error> {
        let result = self
            .collection_allowed_channel
            .find(
                doc! { DbAllowedChannelFieldName::GuildId.name(): guild_id },
                None,
            )
            .await?;
        let result = result.try_collect().await.unwrap_or_else(|_| vec![]);

        Ok(result)
    }

    pub async fn allowed_channel_remove(
        &self,
        channel_id: &str,
        feature: &str,
    ) -> Result<u64, mongodb::error::Error> {
        let result = self
            .collection_allowed_channel
            .delete_one(doc! { DbAllowedChannelFieldName::ChannelId.name(): channel_id, DbAllowedChannelFieldName::Feature.name(): feature }, None)
            .await?;

        Ok(result.deleted_count)
    }
}
