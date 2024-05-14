use artcord_state::model::acc_session::AccSession;
use artcord_state::model::allowed_channel::AllowedChannel;
use artcord_state::model::allowed_guild::AllowedGuild;
use artcord_state::model::allowed_role::AllowedRole;
use artcord_state::model::auto_reaction::AutoReaction;
use artcord_state::model::img::Img;
use artcord_state::model::migration::Migration;
use artcord_state::model::user::User;
use artcord_state::model::{acc::Acc, ws_statistics::DbWsStat};
use cfg_if::cfg_if;

use mongodb::options::ClientOptions;

use mongodb::Client;

use thiserror::Error;
use tracing::{info, trace};

pub mod query;

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
    collection_migration: mongodb::Collection<Migration>,
    collection_ws_statistic: mongodb::Collection<DbWsStat>,
}

// const DATABASE_NAME: &'static str = "artcord";

impl DB {
    pub async fn new(database_name: impl AsRef<str>, mongo_url: impl AsRef<str>) -> Self {
        cfg_if! {
            if #[cfg(feature = "development")] {
                info!("Connecting to database: {}", mongo_url.as_ref());
            } else {
                info!("Connecting to database...");
            }
        }

        let mut client_options = ClientOptions::parse(mongo_url).await.unwrap();
        client_options.app_name = Some("My App".to_string());
        let client = Client::with_options(client_options).unwrap();

        let database = client.database(database_name.as_ref());

        Self::migrate(&database).await.expect("migration failed");

        // panic!("STOP");
        let collection_migration = DB::init_migration(&database).await;
        let collection_user = DB::init_user(&database).await;
        let collection_img = DB::init_img(&database).await;
        let collection_allowed_role = DB::init_allowed_role(&database).await;
        let collection_allowed_channel = DB::init_allowed_channel(&database).await;
        let collection_allowed_guild = DB::init_allowed_guild(&database).await;
        let collection_auto_reaction = DB::init_auto_reaction(&database).await;
        let collection_acc = DB::init_acc(&database).await;
        let collection_acc_session = DB::init_acc_session(&database).await;
        let collection_ws_statistic = DB::init_ws_statistic(&database).await;

        Self {
            database,
            client,
            collection_img,
            collection_user,
            collection_allowed_role,
            collection_allowed_channel,
            collection_allowed_guild,
            collection_auto_reaction,
            collection_acc,
            collection_acc_session,
            collection_migration,
            collection_ws_statistic,
        }
    }
}

#[derive(Error, Debug)]
pub enum DBError {
    // bson::document::ValueAccessError

    #[error("Mongodb: {0}.")]
    Mongo(#[from] mongodb::error::Error),

    #[error("Bson: {0}.")]
    Bson(#[from] bson::document::ValueAccessError),

    #[error("Bson DE: {0}.")]
    BsonDE(#[from] bson::de::Error),

    #[error("Chrono parse: {0}.")]
    Chrono(#[from] chrono::ParseError),

    // #[error("Not found: {0}.")]
    // NotFound(String),
}
