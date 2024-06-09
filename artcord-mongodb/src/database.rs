use artcord_state::global;
use cfg_if::cfg_if;

use mongodb::options::ClientOptions;

use mongodb::Client;

use thiserror::Error;
use tracing::{info, trace};

pub mod model;

const COLLECTION_ACC_SESSION_NAME: &str = "acc_session";
const COLLECTION_ACC_NAME: &str = "acc";
const COLLECTION_ALLOWED_CHANNEL_NAME: &str = "allowed_channel";
const COLLECTION_ALLOWED_GUILD_NAME: &str = "allowed_guild";
const COLLECTION_ALLOWED_ROLE_NAME: &str = "allowed_role";
const COLLECTION_AUTO_REACTION_NAME: &str = "auto_reaction";
const COLLECTION_IMG_NAME: &str = "img";
const COLLECTION_MIGRATION_NAME: &str = "migration";
const COLLECTION_USER_NAME: &str = "user";
const COLLECTION_WS_STATISTIC_NAME: &str = "ws_statistic";
const COLLECTION_WS_IP_MANAGER_NAME: &str = "ws_ip_manager";

#[derive(Clone, Debug)]
pub struct DB {
    pub client: mongodb::Client,
    pub database: mongodb::Database,
    collection_img: mongodb::Collection<global::DbImg>,
    collection_user: mongodb::Collection<global::DbUser>,
    collection_allowed_role: mongodb::Collection<global::DbAllowedRole>,
    collection_allowed_channel: mongodb::Collection<global::DbAllowedChannel>,
    collection_allowed_guild: mongodb::Collection<global::DbAllowedGuild>,
    collection_auto_reaction: mongodb::Collection<global::DbAutoReaction>,
    collection_acc: mongodb::Collection<global::DbAcc>,
    collection_acc_session: mongodb::Collection<global::DbAccSession>,
    collection_migration: mongodb::Collection<global::DbMigration>,
    collection_ws_statistic: mongodb::Collection<global::DbWsCon>,
    collection_ws_ip_manager: mongodb::Collection<global::DbWsIpManager>,
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
        let collection_ws_ip_manager = DB::init_ws_ip_manager(&database).await;

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
            collection_ws_ip_manager,
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

    #[error("Bson DE: {0}.")]
    BsonSE(#[from] bson::ser::Error),

    #[error("Chrono parse: {0}.")]
    Chrono(#[from] chrono::ParseError),

    #[error("ws_ip_manager_from_db conversion err: {0}.")]
    WsIpManagerFromDb(#[from] global::WsIpManagerFromDb),

    #[error("ws_ip_manager_to_db conversion err: {0}.")]
    WsIpManagerToDb(#[from] global::WsIpManagerToDb),

    #[error("req_stats conversion err: {0}.")]
    WsConReqStatToDbErr(#[from] global::WsConReqStatToDbErr),

    // #[error("Not found: {0}.")]
    // NotFound(String),
}
