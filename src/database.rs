use bson::{oid::ObjectId, DateTime};
use bytecheck::CheckBytes;
use cfg_if::cfg_if;
use rkyv::{
    ser::Serializer,
    string::{ArchivedString, StringResolver},
    with::{ArchiveWith, DeserializeWith, SerializeWith},
    Archive, Archived, Fallible,
};
use serde::{Deserialize, Serialize};

#[derive(
    rkyv::Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    Debug,
    Serialize,
    Deserialize,
    Clone,
    CheckBytes,
)]
#[archive(compare(PartialEq), check_bytes)]
#[archive_attr(derive(Debug))]
#[repr(transparent)]
pub struct ArchivedDateTime(Archived<i64>);

#[derive(Debug, CheckBytes)]
#[repr(transparent)]
pub struct ArchivedObjectId(ArchivedString);

#[derive(
    rkyv::Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    Debug,
    Serialize,
    Deserialize,
    Clone,
    PartialEq,
)]
#[archive(compare(PartialEq), check_bytes)]
#[archive_attr(derive(Debug))]
pub struct User {
    #[with(OBJ)]
    pub _id: ObjectId,

    pub guild_id: String,
    pub id: String,
    pub name: String,
    pub pfp_hash: Option<String>,

    #[with(DT)]
    pub modified_at: DateTime,

    #[with(DT)]
    pub created_at: DateTime,
}

impl PartialEq<ObjectId> for ArchivedObjectId {
    fn eq(&self, other: &ObjectId) -> bool {
        self.0 == other.to_string()
    }
}

impl PartialEq<DateTime> for ArchivedDateTime {
    fn eq(&self, other: &DateTime) -> bool {
        self.0 == other.timestamp_millis()
    }
}

pub struct OBJ;
impl ArchiveWith<ObjectId> for OBJ {
    type Archived = ArchivedObjectId;
    type Resolver = StringResolver;

    unsafe fn resolve_with(
        id: &ObjectId,
        pos: usize,
        resolver: Self::Resolver,
        out: *mut Self::Archived,
    ) {
        id.to_string().resolve(pos, resolver, out.cast())
    }
}

impl<S: Fallible + Serializer + ?Sized> SerializeWith<ObjectId, S> for OBJ {
    fn serialize_with(id: &ObjectId, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        ArchivedString::serialize_from_str(id.to_string().as_str(), serializer)
    }
}

impl<D: Fallible + ?Sized> DeserializeWith<ArchivedObjectId, ObjectId, D> for OBJ {
    fn deserialize_with(
        archived: &ArchivedObjectId,
        _deserializer: &mut D,
    ) -> Result<ObjectId, D::Error> {
        Ok(ObjectId::parse_str(archived.0.as_str()).unwrap_or_default())
    }
}

pub struct DT;

impl ArchiveWith<DateTime> for DT {
    type Archived = ArchivedDateTime;
    type Resolver = ();

    unsafe fn resolve_with(
        datetime: &DateTime,
        pos: usize,
        resolver: Self::Resolver,
        out: *mut Self::Archived,
    ) {
        datetime
            .timestamp_millis()
            .resolve(pos, resolver, out.cast());
    }
}

impl<S: Fallible + ?Sized> SerializeWith<DateTime, S> for DT {
    fn serialize_with(
        _datetime: &DateTime,
        _serializer: &mut S,
    ) -> Result<Self::Resolver, S::Error> {
        Ok(())
    }
}

impl<D: Fallible + ?Sized> DeserializeWith<ArchivedDateTime, DateTime, D> for DT {
    fn deserialize_with(
        archived: &ArchivedDateTime,
        _deserializer: &mut D,
    ) -> Result<DateTime, D::Error> {
        Ok(DateTime::from_millis(archived.0))
    }
}

cfg_if! {
if #[cfg(feature = "ssr")] {
        use mongodb::bson::{doc};

        use mongodb::{options::ClientOptions, Client};
        use serenity::prelude::TypeMapKey;
        use std::collections::HashMap;
        use std::sync::Arc;
        use tokio::sync::RwLock;


        #[derive(Debug, Serialize, Deserialize, Clone)]
        pub struct AllowedRole {
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
        pub struct Img {
            pub _id: mongodb::bson::oid::ObjectId,
            pub show: bool,
            pub guild_id: String,
            pub user_id: String,
            pub channel_id: String,
            pub id: String,
            pub org_url: String,
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
