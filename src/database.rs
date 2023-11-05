use cfg_if::cfg_if;

cfg_if! {
if #[cfg(feature = "ssr")] {
        use mongodb::bson::{doc, Binary};
        use mongodb::options::{DeleteOptions, FindOptions};
        use mongodb::{options::ClientOptions, Client};
        use serde::{Deserialize, Serialize};
        use serenity::prelude::TypeMapKey;
        use std::borrow::Borrow;
        use std::collections::HashMap;
        use std::fmt::{Display, Formatter};
        use std::sync::Arc;
        use tokio::sync::RwLock;
        use mongodb::bson::serde_helpers::serialize_hex_string_as_object_id;

        #[derive(Debug, Serialize, Deserialize, Clone)]
        pub struct AllowedRole {
            // #[serde(serialize_with = "serialize_hex_string_as_object_id")]
            pub _id: mongodb::bson::oid::ObjectId,
            pub guild_id: String,
            pub id: String,
            pub name: String,
            pub feature: String,
            pub modified_at: mongodb::bson::DateTime,
            pub created_at: mongodb::bson::DateTime,
        }

        impl TypeMapKey for AllowedRole {
            type Value = Arc<RwLock<HashMap<String, Self>>>;
        }

        #[derive(Debug, Serialize, Deserialize, Clone)]
        pub struct AllowedChannel {
            pub _id: mongodb::bson::oid::ObjectId,
            pub guild_id: String,
            pub id: String,
            pub name: String,
            pub feature: String,
            pub modified_at: mongodb::bson::DateTime,
            pub created_at: mongodb::bson::DateTime,
        }

        impl TypeMapKey for AllowedChannel {
            type Value = Arc<RwLock<HashMap<String, Self>>>;
        }

        #[derive(Debug, Serialize, Deserialize, Clone)]
        pub struct User {
            pub _id: mongodb::bson::oid::ObjectId,
            pub guild_id: String,
            pub id: String,
            pub name: String,
            pub pfp_hash: Option<String>,
            pub modified_at: mongodb::bson::DateTime,
            pub created_at: mongodb::bson::DateTime,
        }

        #[derive(Debug, Serialize, Deserialize, Clone)]
        pub struct Img {
            pub _id: mongodb::bson::oid::ObjectId,
            pub guild_id: String,
            pub user_id: String,
            pub msg_id: String,
            pub org_hash: String,
            pub format: String,
            pub width: u32,
            pub height: u32,
            pub has_high: bool,
            pub has_medium: bool,
            pub has_low: bool,
            pub modified_at: mongodb::bson::DateTime,
            pub created_at: mongodb::bson::DateTime,
        }

        #[derive(Clone, Debug)]
        pub struct DB {
            pub client: mongodb::Client,
            pub database: mongodb::Database,
            pub collection_img: mongodb::Collection<Img>,
            pub collection_user: mongodb::Collection<User>,
            pub collection_allowed_role: mongodb::Collection<AllowedRole>,
            pub collection_allowed_channel: mongodb::Collection<AllowedChannel>,
        }

        impl TypeMapKey for DB {
            type Value = Self;
        }

        pub async fn create_database() -> DB {
            let mut client_options = ClientOptions::parse("mongodb://root:example@localhost:27017")
                .await
                .unwrap();
            client_options.app_name = Some("My App".to_string());
            let client = Client::with_options(client_options).unwrap();

            let database = client.database("artcord");
            let collection_img = database.collection::<Img>("img");
            let collection_user = database.collection::<User>("user");
            let collection_allowed_channel = database.collection::<AllowedChannel>("allowed_channel");
            let collection_allowed_role = database.collection::<AllowedRole>("allowed_role");

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
            }
        }
    }
}
