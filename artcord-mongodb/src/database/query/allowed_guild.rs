use crate::database::DB;
use artcord_state::model::allowed_guild::{AllowedGuild, AllowedGuildFieldName};
use bson::doc;
use futures::TryStreamExt;
use mongodb::{options::IndexOptions, Collection, Database, IndexModel};

const COLLECTION_ALLOWED_GUILD_NAME: &'static str = "allowed_guild";

impl DB {
    pub async fn init_allowed_guild(database: &Database) -> Collection<AllowedGuild> {
        let (index1, index2) = (
            {
                let opts = IndexOptions::builder().unique(true).build();
                IndexModel::builder()
                    .keys(doc! { AllowedGuildFieldName::GuildId.name(): -1 })
                    .options(opts)
                    .build()
            },
            {
                let opts = IndexOptions::builder().unique(true).build();
                IndexModel::builder()
                    .keys(doc! {  AllowedGuildFieldName::Name.name(): -1 })
                    .options(opts)
                    .build()
            },
        );

        let collection = database.collection::<AllowedGuild>(COLLECTION_ALLOWED_GUILD_NAME);

        collection
            .create_indexes([index1, index2], None)
            .await
            .expect("Failed to create collection index.");

        collection
    }
}

impl DB {
    pub async fn allowed_guild_insert_default(
        &self,
        guild_id: String,
    ) -> Result<Option<String>, mongodb::error::Error> {
        let name = String::from("DEFAULT");
        let allowed_guild = self
            .collection_allowed_guild
            .find_one(doc! { AllowedGuildFieldName::Name.name(): &name}, None)
            .await?;
        if allowed_guild.is_none() {
            let allowed_guild = self
                .collection_allowed_guild
                .insert_one(AllowedGuild::new(guild_id, name), None)
                .await?;
            return Ok(Some(allowed_guild.inserted_id.to_string()));
        }
        Ok(None)
    }

    pub async fn allowed_guild_insert(
        &self,
        new_guild: AllowedGuild,
    ) -> Result<Option<String>, mongodb::error::Error> {
        let allowed_guild = self
            .collection_allowed_guild
            .find_one(
                doc! {AllowedGuildFieldName::GuildId.name(): &new_guild.guild_id},
                None,
            )
            .await?;
        if allowed_guild.is_none() {
            let allowed_guild = self
                .collection_allowed_guild
                .insert_one(new_guild, None)
                .await?;
            return Ok(Some(allowed_guild.inserted_id.to_string()));
        }
        Ok(None)
    }

    pub async fn allowed_guild_remove_one(
        &self,
        guild_id: &str,
    ) -> Result<bool, mongodb::error::Error> {
        let result = self
            .collection_allowed_guild
            .delete_one(
                doc! { AllowedGuildFieldName::GuildId.name(): guild_id },
                None,
            )
            .await?;
        Ok(result.deleted_count > 0)
    }

    pub async fn allowed_guild_all(&self) -> Result<Vec<AllowedGuild>, mongodb::error::Error> {
        let allowed_guilds = self.collection_allowed_guild.find(None, None).await?;
        let allowed_guilds = allowed_guilds
            .try_collect()
            .await
            .unwrap_or_else(|_| vec![]);
        Ok(allowed_guilds)
    }

    pub async fn allowed_guild_exists(
        &self,
        guild_id: &str,
    ) -> Result<bool, mongodb::error::Error> {
        let result = self
            .collection_allowed_guild
            .count_documents(doc! {AllowedGuildFieldName::GuildId.name(): guild_id}, None)
            .await?;
        Ok(result > 0)
    }
}

