use crate::database::models::acc::{Acc, AccFieldName};
use crate::database::models::acc_session::{AccSession, AccSessionFieldName};
use crate::database::models::allowed_channel::{AllowedChannel, AllowedChannelFieldName};
use crate::database::models::allowed_guild::{AllowedGuild, AllowedGuildFieldName};
use crate::database::models::allowed_role::{AllowedRole, AllowedRoleFieldName};
use crate::database::models::auto_reaction::{AutoReaction, AutoReactionFieldName};
use crate::database::models::img::{Img, ImgFieldName};
use crate::database::models::user::{User, UserFieldName};
use crate::message::server_msg::ServerMsg;
use crate::message::server_msg_img::{AggImg, AggImgFieldName};
use bson::oid::ObjectId;
use bson::{doc, Bson, DateTime, Document};
use chrono::Utc;
use futures::TryStreamExt;
use leptos::server_fn::const_format::concatcp;
use mongodb::options::{ClientOptions, IndexOptions};
use mongodb::results::{DeleteResult, InsertOneResult};
use mongodb::{Client, Cursor, IndexModel};
use rusqlite::Connection;
use serenity::prelude::TypeMapKey;
use std::str::FromStr;
use std::sync::Arc;
use thiserror::Error;

#[derive(Clone, Debug)]
pub struct DB {
    pub client: mongodb::Client,
    pub database: mongodb::Database,
    collection_img: mongodb::Collection<Img>,
    collection_user: mongodb::Collection<User>,
    collection_allowed_role: mongodb::Collection<AllowedRole>,
    collection_allowed_channel: mongodb::Collection<AllowedChannel>,
    collection_allowed_guild: mongodb::Collection<AllowedGuild>,
    collection_auto_reaction: mongodb::Collection<AutoReaction>,
    collection_acc: mongodb::Collection<Acc>,
    collection_acc_session: mongodb::Collection<AccSession>,
}

impl TypeMapKey for DB {
    type Value = Arc<Self>;
}

// Err(mongodb::error::Error::custom(Arc::new("invalid ReactionType type".to_string())) )

impl DB {
    pub async fn acc_session_find_one(
        &self,
        token: &str,
    ) -> Result<Option<AccSession>, mongodb::error::Error> {
        let acc = self
            .collection_acc_session
            .find_one(doc! { AccSessionFieldName::Token.name(): token }, None)
            .await?;

        Ok(acc)
    }
    pub async fn acc_session_insert_one(
        &self,
        acc_session: AccSession,
    ) -> Result<InsertOneResult, mongodb::error::Error> {
        let acc = self
            .collection_acc_session
            .insert_one(acc_session, None)
            .await?;

        Ok(acc)
    }

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
    pub async fn acc_insert_one(&self, acc: Acc) -> Result<Bson, mongodb::error::Error> {
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

        Ok(result.inserted_id)
    }

    pub async fn allowed_role_find_one(
        &self,
        guild_id: &str,
        role_id: &str,
        feature_option: &str,
    ) -> Result<Option<AllowedRole>, mongodb::error::Error> {
        let role = self
            .collection_allowed_role
            .find_one(
                doc! { AllowedRoleFieldName::GuildId.name(): guild_id, AllowedRoleFieldName::RoleId.name(): role_id, AllowedRoleFieldName::Feature.name(): feature_option },
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
    ) -> Result<DeleteResult, mongodb::error::Error> {
        let result = self
            .collection_allowed_role
            .delete_one(
                doc! { AllowedRoleFieldName::GuildId.name(): guild_id, AllowedRoleFieldName::RoleId.name(): role_id, AllowedRoleFieldName::Feature.name(): feature_option },
                None,
            )
            .await?;

        Ok(result)
    }

    // pub async fn allowed_role_find_all(&self, guild_id: &str) -> Result<Vec<AllowedRole>, mongodb::error::Error> {
    //     let result = self
    //         .collection_allowed_role
    //         .find(doc! { "guild_id": guild_id }, None)
    //         .await?
    //         .try_collect()
    //         .await
    //         .unwrap_or_else(|_| vec![]);
    //
    //     Ok(result)
    // }

    pub async fn allowed_role_insert_one(
        &self,
        allowed_channel: AllowedRole,
    ) -> Result<InsertOneResult, mongodb::error::Error> {
        let result = self
            .collection_allowed_role
            .insert_one(allowed_channel, None)
            .await?;

        Ok(result)
    }

    pub async fn allowed_channel_insert_one(
        &self,
        allowed_channel: AllowedChannel,
    ) -> Result<InsertOneResult, mongodb::error::Error> {
        let result = self
            .collection_allowed_channel
            .insert_one(allowed_channel, None)
            .await?;

        Ok(result)
    }

    pub async fn allowed_role_find_all(
        &self,
        guild_id: &str,
    ) -> Result<Vec<AllowedRole>, mongodb::error::Error> {
        let result = self
            .collection_allowed_role
            .find(
                doc! { AllowedRoleFieldName::GuildId.name(): guild_id },
                None,
            )
            .await?;
        let result = result.try_collect().await.unwrap_or_else(|_| vec![]);

        Ok(result)
    }

    pub async fn allowed_channel_find_all(
        &self,
        guild_id: &str,
    ) -> Result<Vec<AllowedChannel>, mongodb::error::Error> {
        let result = self
            .collection_allowed_channel
            .find(
                doc! { AllowedChannelFieldName::GuildId.name(): guild_id },
                None,
            )
            .await?;
        let result = result.try_collect().await.unwrap_or_else(|_| vec![]);

        Ok(result)
    }

    pub async fn remove_channel(
        &self,
        channel_id: &str,
        feature: &str,
    ) -> Result<DeleteResult, mongodb::error::Error> {
        let result = self
            .collection_allowed_channel
            .delete_one(doc! { AllowedChannelFieldName::ChannelId.name(): channel_id, AllowedChannelFieldName::Feature.name(): feature }, None)
            .await?;

        Ok(result)
    }

    pub async fn user_insert_one(
        &self,
        user: User,
    ) -> Result<InsertOneResult, mongodb::error::Error> {
        let result = self.collection_user.insert_one(user, None).await?;

        Ok(result)
    }

    pub async fn user_update_one_raw(
        &self,
        user_id: &str,
        update: Document,
    ) -> Result<(), mongodb::error::Error> {
        self.collection_user
            .update_one(
                doc! { UserFieldName::Id.name(): user_id },
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
            .find_one(doc! {UserFieldName::Id.name(): user_id}, None)
            .await?;
        //println!("wtf{:?}", user);
        // Ok(ServerMsg::Profile(user))
        Ok(result)
    }

    pub async fn img_aggregate_user_gallery(
        &self,
        amount: u32,
        from: i64,
        user_id: &str,
    ) -> Result<ServerMsg, mongodb::error::Error> {
        let user = self
            .collection_user
            .find_one(doc! {UserFieldName::Id.name(): user_id}, None)
            .await?;
        if let None = user {
            return Ok(ServerMsg::ProfileImgs(None));
        }

        let pipeline = vec![
            doc! { "$sort": doc! { ImgFieldName::CreatedAt.name(): -1 } },
            doc! { "$match": doc! { ImgFieldName::CreatedAt.name(): { "$lt": from }, ImgFieldName::Show.name(): true, ImgFieldName::UserId.name(): user_id } },
            doc! { "$limit": Some( amount.clamp(25, 10000) as i64) },
            doc! { "$lookup": doc! { "from": COLLECTION_USER_NAME, "localField": ImgFieldName::UserId.name(), "foreignField": UserFieldName::Id.name(), "as": AggImgFieldName::User.name()} },
            doc! { "$unwind": format!("${}", AggImgFieldName::User.name()) },
        ];
        // println!("{:#?}", pipeline);

        let mut imgs = self.collection_img.aggregate(pipeline, None).await?;
        let imgs = imgs.try_collect().await.unwrap_or_else(|_| vec![]);

        let mut send_this: Vec<AggImg> = Vec::new();

        for img in imgs {
            let doc: AggImg = mongodb::bson::from_document(img)?;
            send_this.push(doc);
        }

        // while let Some(result) = imgs.try_next().await? {
        //     let doc: ServerMsgImg = mongodb::bson::from_document(result)?;
        //     send_this.push(doc);
        // }

        //println!("Len: {}", send_this.len());

        Ok(ServerMsg::ProfileImgs(Some(send_this)))
    }

    pub async fn img_aggregate_gallery(
        &self,
        amount: u32,
        from: i64,
    ) -> Result<ServerMsg, mongodb::error::Error> {
        let pipeline = vec![
            doc! { "$sort": doc! { ImgFieldName::CreatedAt.name(): -1 } },
            doc! { "$match": doc! { ImgFieldName::CreatedAt.name(): { "$lt": from }, ImgFieldName::Show.name(): true } },
            doc! { "$limit": Some( amount.clamp(25, 10000) as i64) },
            doc! { "$lookup": doc! { "from": COLLECTION_USER_NAME, "localField": ImgFieldName::UserId.name(), "foreignField": UserFieldName::Id.name(), "as": AggImgFieldName::User.name()} },
            doc! { "$unwind": format!("${}", AggImgFieldName::User.name()) },
        ];
        // println!("{:#?}", pipeline);

        let mut imgs = self.collection_img.aggregate(pipeline, None).await?;

        let mut send_this: Vec<AggImg> = Vec::new();

        while let Some(result) = imgs.try_next().await? {
            let doc: AggImg = mongodb::bson::from_document(result)?;
            //let a = doc.f
            send_this.push(doc);
            // println!("hh");
        }

        Ok(ServerMsg::Imgs(send_this))
    }

    pub async fn img_find_one(
        &self,
        guild_id: u64,
        file_hash: &str,
    ) -> Result<Option<Img>, mongodb::error::Error> {
        let found_img = self
            .collection_img
            .find_one(
                doc! {
                    ImgFieldName::GuildId.name(): guild_id.to_string(),
                    ImgFieldName::OrgHash.name(): file_hash.clone()
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
                doc! { ImgFieldName::GuildId.name(): guild_id.to_string(), ImgFieldName::Id.name(): msg_id.to_string() },
                doc! { "$set": { ImgFieldName::Show.name(): false } },
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
                doc! { ImgFieldName::GuildId.name(): guild_id.to_string(), ImgFieldName::OrgHash.name(): file_hash },
                doc! {
                    "$set": update
                },
                None,
            )
            .await?;

        Ok(())
    }

    pub async fn img_insert(&self, img: &Img) -> Result<(), mongodb::error::Error> {
        let is_ms = img.created_at < 9999999999999;
        if !is_ms {
            return Err(mongodb::error::Error::custom(Arc::new(format!(
                "{}: {}",
                ImgFieldName::CreatedAt.name(),
                is_ms
            ))));
        }
        let is_ms = img.modified_at < 9999999999999;
        if !is_ms {
            return Err(mongodb::error::Error::custom(Arc::new(format!(
                "{}: {}",
                ImgFieldName::ModifiedAt.name(),
                is_ms
            ))));
        }
        self.collection_img.insert_one(img, None).await?;
        Ok(())
    }

    pub async fn feature_exists(
        &self,
        guild_id: u64,
        channel_id: u64,
        feature: &str,
    ) -> Result<bool, mongodb::error::Error> {
        let channel = self
            .collection_allowed_channel
            .find_one(
                doc! { AllowedChannelFieldName::GuildId.name(): guild_id.to_string(), AllowedChannelFieldName::ChannelId.name(): channel_id.to_string(), AllowedChannelFieldName::Feature.name(): feature.to_string() },
                None,
            )
            .await?;
        Ok(channel.is_some())
    }

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
        // println!("{:#?}", &filter);
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

    pub async fn allowed_guild_insert_default(
        &self,
        guild_id: String,
    ) -> Result<Option<String>, mongodb::error::Error> {
        let name = String::from("DEFAULT");
        let allowed_guild = self
            .collection_allowed_guild
            .find_one(doc! { AllowedGuildFieldName::GuildId.name(): &guild_id, AllowedGuildFieldName::Name.name(): &name}, None)
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

const DATABASE_NAME: &'static str = "artcord";
const COLLECTION_ALLOWED_CHANNEL_NAME: &'static str = "allowed_channel";
const COLLECTION_ALLOWED_ROLE_NAME: &'static str = "allowed_role";
const COLLECTION_ALLOWED_GUILD_NAME: &'static str = "allowed_guild";
const COLLECTION_IMG_NAME: &'static str = "img";
const COLLECTION_USER_NAME: &'static str = "user";
const COLLECTION_ACC_NAME: &'static str = "acc";
const COLLECTION_ACC_SESSION_NAME: &'static str = "acc_session";
const COLLECTION_AUTO_REACTION_NAME: &'static str = "auto_reaction";

pub async fn create_database(mongo_url: String) -> DB {
    // let a = ImgFieldName::GuildId.name();
    println!("Connecting to database...");

    let mut client_options = ClientOptions::parse(mongo_url).await.unwrap();
    client_options.app_name = Some("My App".to_string());
    let client = Client::with_options(client_options).unwrap();

    let database = client.database(DATABASE_NAME);
    let collection_img = database.collection::<Img>(COLLECTION_IMG_NAME);

    let opts = IndexOptions::builder().unique(true).build();
    let index = IndexModel::builder()
        .keys(doc! { UserFieldName::Id.name(): -1 })
        .options(opts)
        .build();
    let collection_user = database.collection::<User>(COLLECTION_USER_NAME);
    collection_user
        .create_index(index, None)
        .await
        .expect("Failed to create collection index.");

    let collection_allowed_channel =
        database.collection::<AllowedChannel>(COLLECTION_ALLOWED_CHANNEL_NAME);
    let collection_allowed_role = database.collection::<AllowedRole>(COLLECTION_ALLOWED_ROLE_NAME);
    let collection_allowed_guild =
        database.collection::<AllowedGuild>(COLLECTION_ALLOWED_GUILD_NAME);

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

    let collection_acc_session = database.collection::<AccSession>(COLLECTION_ACC_SESSION_NAME);

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

    let db_list = client.list_database_names(doc! {}, None).await.unwrap();
    println!("Databases: {:?}", db_list);

    //let conn = Connection::open_in_memory().expect("Failed to create sqlite db");

    DB {
        database,
        client,
        collection_img,
        collection_user,
        collection_allowed_channel,
        collection_allowed_role,
        collection_allowed_guild,
        collection_auto_reaction,
        collection_acc,
        collection_acc_session,
    }
}

#[derive(Error, Debug)]
pub enum DBError {
    #[error("Mongodb: {0}.")]
    Mongo(#[from] mongodb::error::Error),

    #[error("Not found: {0}.")]
    NotFound(String),
}
