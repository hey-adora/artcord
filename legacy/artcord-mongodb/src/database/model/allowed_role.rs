use crate::database::{COLLECTION_ALLOWED_ROLE_NAME, DB};
use artcord_state::global::{DbAllowedRole, DbAllowedRoleFieldName};
use bson::doc;
use futures::TryStreamExt;
use mongodb::{options::IndexOptions, Collection, Database, IndexModel};
use field_types::FieldName;
use serde::{Deserialize, Serialize};
use chrono::Utc;

impl DB {
    pub async fn init_allowed_role(database: &Database) -> Collection<DbAllowedRole> {
        let opts = IndexOptions::builder().unique(true).build();
        let index = IndexModel::builder()
            .keys(doc! { DbAllowedRoleFieldName::GuildId.name(): -1, DbAllowedRoleFieldName::RoleId.name(): -1, DbAllowedRoleFieldName::Feature.name(): -1 })
            .options(opts)
            .build();

        let collection = database.collection::<DbAllowedRole>(&COLLECTION_ALLOWED_ROLE_NAME);

        collection
        .create_index(index, None)
        .await
        .expect("Failed to create collection index.");

        collection
    }
}

impl DB {
    pub async fn allowed_role_find_one(
        &self,
        guild_id: &str,
        role_id: &str,
        feature_option: &str,
    ) -> Result<Option<DbAllowedRole>, mongodb::error::Error> {
        let role = self
            .collection_allowed_role
            .find_one(
                doc! { DbAllowedRoleFieldName::GuildId.name(): guild_id, DbAllowedRoleFieldName::RoleId.name(): role_id, DbAllowedRoleFieldName::Feature.name(): feature_option },
                None,
            )
            .await?;

        Ok(role)
    }

    pub async fn remove_allowed_role(
        &self,
        guild_id: &str,
        role_id: &str,
        feature_option: &str,
    ) -> Result<u64, mongodb::error::Error> {
        let result = self
            .collection_allowed_role
            .delete_one(
                doc! { DbAllowedRoleFieldName::GuildId.name(): guild_id, DbAllowedRoleFieldName::RoleId.name(): role_id, DbAllowedRoleFieldName::Feature.name(): feature_option },
                None,
            )
            .await?;

        Ok(result.deleted_count)
    }

    pub async fn allowed_role_insert_one(
        &self,
        allowed_channel: DbAllowedRole,
    ) -> Result<String, mongodb::error::Error> {
        let result = self
            .collection_allowed_role
            .insert_one(allowed_channel, None)
            .await?;

        Ok(result.inserted_id.to_string())
    }

    pub async fn allowed_role_find_all(
        &self,
        guild_id: &str,
    ) -> Result<Vec<DbAllowedRole>, mongodb::error::Error> {
        let result = self
            .collection_allowed_role
            .find(
                doc! { DbAllowedRoleFieldName::GuildId.name(): guild_id },
                None,
            )
            .await?;
        let result = result.try_collect().await.unwrap_or_else(|_| vec![]);

        Ok(result)
    }

}