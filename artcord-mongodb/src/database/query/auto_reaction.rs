use artcord_state::model::auto_reaction::{AutoReaction, AutoReactionFieldName};
use bson::{doc, Document};
use futures::TryStreamExt;
use mongodb::{options::IndexOptions, Collection, Database, IndexModel};

use crate::database::DB;

const COLLECTION_AUTO_REACTION_NAME: &'static str = "auto_reaction";

impl DB {
    pub async fn init_auto_reaction(database: &Database) -> Collection<AutoReaction> {
        let opts = IndexOptions::builder().unique(true).build();
        let index = IndexModel::builder()
            .keys(doc! {AutoReactionFieldName::GuildId.name(): -1,  AutoReactionFieldName::Unicode.name(): -1, AutoReactionFieldName::EmojiId.name(): -1, AutoReactionFieldName::Name.name(): -1, AutoReactionFieldName::Animated.name(): -1 })
            .options(opts)
            .build();
        let collection_auto_reaction =
            database.collection::<AutoReaction>(COLLECTION_AUTO_REACTION_NAME);

        collection_auto_reaction
            .create_index(index, None)
            .await
            .expect("Failed to create collection index.");

        collection_auto_reaction
    }
}

impl DB {
    pub async fn auto_reaction_delete_one(
        &self,
        auto_reaction: &AutoReaction,
    ) -> Result<bool, mongodb::error::Error> {
        let result = self.collection_auto_reaction.delete_one(doc!{ AutoReactionFieldName::EmojiId.name(): &auto_reaction.emoji_id, AutoReactionFieldName::GuildId.name(): auto_reaction.guild_id.as_str(), AutoReactionFieldName::Name.name(): &auto_reaction.name, AutoReactionFieldName::Animated.name(): auto_reaction.animated, AutoReactionFieldName::Unicode.name(): &auto_reaction.unicode }, None).await?;
        Ok(result.deleted_count > 0)
    }

    pub async fn auto_reactoin_exists(
        &self,
        auto_reaction: &AutoReaction,
    ) -> Result<bool, mongodb::error::Error> {
        let result = self.collection_auto_reaction.find_one(doc! { AutoReactionFieldName::EmojiId.name(): &auto_reaction.emoji_id, AutoReactionFieldName::GuildId.name(): auto_reaction.guild_id.as_str(), AutoReactionFieldName::Name.name(): &auto_reaction.name, AutoReactionFieldName::Animated.name(): auto_reaction.animated, AutoReactionFieldName::Unicode.name(): &auto_reaction.unicode }, None).await?;
        Ok(result.is_some())
    }

    pub async fn auto_reactoin_delete_many(
        &self,
        auto_reactions: Vec<AutoReaction>,
    ) -> Result<(), mongodb::error::Error> {
        let filter = doc! { "$or": auto_reactions.into_iter().map(|auto_reaction| doc! { AutoReactionFieldName::EmojiId.name(): &auto_reaction.emoji_id, AutoReactionFieldName::GuildId.name(): auto_reaction.guild_id.as_str(), AutoReactionFieldName::Name.name(): &auto_reaction.name, AutoReactionFieldName::Animated.name(): auto_reaction.animated, AutoReactionFieldName::Unicode.name(): &auto_reaction.unicode }).collect::<Vec<Document>>() };
        self.collection_auto_reaction
            .delete_many(filter, None)
            .await?;
        Ok(())
    }

    pub async fn auto_reactoin_insert_many_from_type(
        &self,
        auto_reactions: Vec<AutoReaction>,
    ) -> Result<(), mongodb::error::Error> {
        self.collection_auto_reaction
            .insert_many(auto_reactions, None)
            .await?;
        Ok(())
    }

    pub async fn auto_reactions(
        &self,
        guild_id: u64,
    ) -> Result<Vec<AutoReaction>, mongodb::error::Error> {
        let result = self
            .collection_auto_reaction
            .find(
                doc! {AutoReactionFieldName::GuildId.name(): guild_id.to_string()},
                None,
            )
            .await?;
        let result = result.try_collect().await.unwrap_or_else(|_| vec![]);
        Ok(result)
    }
}