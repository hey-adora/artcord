use crate::database::models::allowed_channel::AllowedChannel;
use crate::database::models::allowed_guild::AllowedGuild;
use crate::database::models::allowed_role::AllowedRole;
use crate::database::models::auto_reaction::AutoReaction;
use crate::database::models::img::Img;
use crate::database::models::user::User;
use crate::server::server_msg::ServerMsg;
use crate::server::server_msg_img::ServerMsgImg;
use bson::{doc, DateTime, Document};
use futures::TryStreamExt;
use mongodb::options::{ClientOptions, IndexOptions};
use mongodb::{Client, IndexModel};
use serenity::prelude::TypeMapKey;
use std::sync::Arc;
use thiserror::Error;

#[derive(Clone, Debug)]
pub struct DB {
    pub client: mongodb::Client,
    pub database: mongodb::Database,
    collection_img: mongodb::Collection<Img>,
    pub collection_user: mongodb::Collection<User>,
    pub collection_allowed_role: mongodb::Collection<AllowedRole>,
    pub collection_allowed_channel: mongodb::Collection<AllowedChannel>,
    collection_allowed_guild: mongodb::Collection<AllowedGuild>,
    pub collection_auto_reaction: mongodb::Collection<AutoReaction>,
}

impl TypeMapKey for DB {
    type Value = Arc<Self>;
}

// Err(mongodb::error::Error::custom(Arc::new("invalid ReactionType type".to_string())) )

impl DB {
    pub async fn user_find_one(&self, user_id: &str) -> Result<ServerMsg, mongodb::error::Error> {
        let user = self
            .collection_user
            .find_one(doc! {"id": user_id}, None)
            .await?;
        //println!("wtf{:?}", user);
        Ok(ServerMsg::Profile(user))
    }

    pub async fn img_aggregate_user_gallery(
        &self,
        amount: u32,
        from: DateTime,
        user_id: &str,
    ) -> Result<ServerMsg, mongodb::error::Error> {
        let user = self
            .collection_user
            .find_one(doc! {"id": user_id}, None)
            .await?;
        if let None = user {
            return Ok(ServerMsg::ProfileImgs(None));
        }

        let pipeline = vec![
            doc! { "$sort": doc! { "created_at": -1 } },
            doc! { "$match": doc! { "created_at": { "$lt": from }, "show": true, "user_id": user_id } },
            doc! { "$limit": Some( amount.clamp(25, 10000) as i64) },
            doc! { "$lookup": doc! { "from": "user", "localField": "user_id", "foreignField": "id", "as": "user"} },
            doc! { "$unwind": "$user" },
        ];
        // println!("{:#?}", pipeline);

        let mut imgs = self.collection_img.aggregate(pipeline, None).await?;

        let mut send_this: Vec<ServerMsgImg> = Vec::new();

        while let Some(result) = imgs.try_next().await? {
            let doc: ServerMsgImg = mongodb::bson::from_document(result)?;
            send_this.push(doc);
        }

        //println!("Len: {}", send_this.len());

        Ok(ServerMsg::ProfileImgs(Some(send_this)))
    }

    pub async fn img_aggregate_gallery(
        &self,
        amount: u32,
        from: DateTime,
    ) -> Result<ServerMsg, mongodb::error::Error> {
        let pipeline = vec![
            doc! { "$sort": doc! { "created_at": -1 } },
            doc! { "$match": doc! { "created_at": { "$lt": from }, "show": true } },
            doc! { "$limit": Some( amount.clamp(25, 10000) as i64) },
            doc! { "$lookup": doc! { "from": "user", "localField": "user_id", "foreignField": "id", "as": "user"} },
            doc! { "$unwind": "$user" },
        ];
        // println!("{:#?}", pipeline);

        let mut imgs = self.collection_img.aggregate(pipeline, None).await?;

        let mut send_this: Vec<ServerMsgImg> = Vec::new();

        while let Some(result) = imgs.try_next().await? {
            let doc: ServerMsgImg = mongodb::bson::from_document(result)?;
            send_this.push(doc);
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
                    "guild_id": guild_id.to_string(),
                    "org_hash": file_hash.clone()
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
                doc! { "guild_id": guild_id.to_string(), "id": msg_id.to_string() },
                doc! { "$set": { "show": false } },
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
                doc! { "guild_id": guild_id.to_string(), "org_hash": file_hash },
                doc! {
                    "$set": update
                },
                None,
            )
            .await?;

        Ok(())
    }

    pub async fn img_insert(&self, img: &Img) -> Result<(), mongodb::error::Error> {
        let is_ms = img.created_at.timestamp_millis() < 9999999999999;
        if !is_ms {
            return Err(mongodb::error::Error::custom(Arc::new(format!(
                "created_at: {}",
                is_ms
            ))));
        }
        let is_ms = img.modified_at.timestamp_millis() < 9999999999999;
        if !is_ms {
            return Err(mongodb::error::Error::custom(Arc::new(format!(
                "modified_at: {}",
                is_ms
            ))));
        }
        self.collection_img.insert_one(img, None).await?;
        Ok(())
    }

    pub async fn reset_img_time(&self, guild_id: u64) -> Result<(), mongodb::error::Error> {
        let ops = mongodb::options::FindOptions::builder()
            .sort(doc! {"created_at": 1})
            .build();
        let mut cursor = self.collection_img.find(None, ops).await?;
        let mut modified_count = 0;
        let mut count = 0;

        while let Some(img) = cursor.try_next().await? {
            if count % 1000 == 0 {
                println!("{}, {}", count, modified_count);
            }

            let mut update_doc = Document::new();

            let ms = img.created_at.timestamp_millis();
            let is_ms = ms < 9999999999999;
            if !is_ms {
                let to_ms = if is_ms { ms } else { ms / 1000000 };
                let created_at = DateTime::from_millis(to_ms);
                update_doc.insert("created_at", created_at);
            }

            let ms = img.modified_at.timestamp_millis();
            let is_ms = ms < 9999999999999;
            if !is_ms {
                let to_ms = if is_ms { ms } else { ms / 1000000 };
                let modified_at = DateTime::from_millis(to_ms);
                update_doc.insert("modified_at", modified_at);
            }

            if update_doc.len() > 0 {
                self.collection_img
                    .update_one(doc! { "_id": img._id }, doc! {"$set": update_doc}, None)
                    .await?;
                modified_count += 1;
            }

            count += 1;
        }
        println!("{}, {}", count, modified_count);

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
                doc! { "guild_id": guild_id.to_string(), "id": channel_id.to_string(), "feature": feature.to_string() },
                None,
            )
            .await?;
        Ok(channel.is_some())
    }

    pub async fn auto_reaction_delete_one(
        &self,
        auto_reaction: &AutoReaction,
    ) -> Result<bool, mongodb::error::Error> {
        let result = self.collection_auto_reaction.delete_one(doc!{ "id": &auto_reaction.id, "guild_id": auto_reaction.guild_id.as_str(), "name": &auto_reaction.name, "animated": auto_reaction.animated, "unicode": &auto_reaction.unicode }, None).await?;
        Ok(result.deleted_count > 0)
    }

    pub async fn auto_reactoin_exists(
        &self,
        auto_reaction: &AutoReaction,
    ) -> Result<bool, mongodb::error::Error> {
        let result = self.collection_auto_reaction.find_one(doc! { "id": &auto_reaction.id, "guild_id": auto_reaction.guild_id.as_str(), "name": &auto_reaction.name, "animated": auto_reaction.animated, "unicode": &auto_reaction.unicode }, None).await?;
        Ok(result.is_some())
    }

    pub async fn auto_reactoin_delete_many(
        &self,
        auto_reactions: Vec<AutoReaction>,
    ) -> Result<(), mongodb::error::Error> {
        let filter = doc! { "$or": auto_reactions.into_iter().map(|auto_reaction| doc! { "id": &auto_reaction.id, "guild_id": auto_reaction.guild_id.as_str(), "name": &auto_reaction.name, "animated": auto_reaction.animated, "unicode": &auto_reaction.unicode }).collect::<Vec<Document>>() };
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
            .find(doc! {"guild_id": guild_id.to_string()}, None)
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
            .find_one(doc! {"id": &guild_id, "name": &name}, None)
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
            .find_one(doc! {"id": &new_guild.id}, None)
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
            .delete_one(doc! { "id": guild_id }, None)
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
            .count_documents(doc! {"id": guild_id}, None)
            .await?;
        Ok(result > 0)
    }
}

pub async fn create_database(mongo_url: String) -> DB {
    let mut client_options = ClientOptions::parse(mongo_url).await.unwrap();
    client_options.app_name = Some("My App".to_string());
    let client = Client::with_options(client_options).unwrap();

    let database = client.database("artcord");
    let collection_img = database.collection::<Img>("img");
    let collection_user = database.collection::<User>("user");
    let collection_allowed_channel = database.collection::<AllowedChannel>("allowed_channel");
    let collection_allowed_role = database.collection::<AllowedRole>("allowed_role");
    let collection_allowed_guild = database.collection::<AllowedGuild>("allowed_guild");

    let opts = IndexOptions::builder().unique(true).build();
    let index = IndexModel::builder()
        .keys(doc! {"guild_id": -1,  "unicode": -1, "id": -1, "name": -1, "animated": -1 })
        .options(opts)
        .build();

    let collection_auto_reaction = database.collection::<AutoReaction>("auto_reaction");
    collection_auto_reaction
        .create_index(index, None)
        .await
        .expect("Failed to create collection index.");

    println!("Connecting to database...");
    let db_list = client.list_database_names(doc! {}, None).await.unwrap();
    println!("Databases: {:?}", db_list);

    DB {
        database,
        client,
        collection_img,
        collection_user,
        collection_allowed_channel,
        collection_allowed_role,
        collection_allowed_guild,
        collection_auto_reaction,
    }
}

#[derive(Error, Debug)]
pub enum DBError {
    #[error("Mongodb: {0}.")]
    Mongo(#[from] mongodb::error::Error),

    #[error("Not found: {0}.")]
    NotFound(String),
}
