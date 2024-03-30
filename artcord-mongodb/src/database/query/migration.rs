use artcord_state::model::{
    migration::{Migration, MigrationFieldName},
    user::User,
};
use bson::{doc, Document};
use futures::TryStreamExt;
use mongodb::{options::IndexOptions, Collection, Database, IndexModel};
use thiserror::Error;
use tracing::{info, warn};

use crate::database::DB;

const COLLECTION_MIGRATION_NAME: &'static str = "migration";

impl DB {
    pub async fn init_migration(database: &Database) -> Collection<Migration> {
        let opts = IndexOptions::builder().unique(true).build();
        let index = IndexModel::builder()
            .keys(doc! { MigrationFieldName::Name.name(): -1 })
            .options(opts)
            .build();

        let collection = database.collection::<Migration>(COLLECTION_MIGRATION_NAME);

        collection
            .create_index(index, None)
            .await
            .expect("Failed to create collection index.");

        collection
    }
}

impl DB {
    pub async fn migrate(database: &Database) -> Result<(), MigrationError> {
        let latest_migration_v: u32 = 1;

        let collection_migration: Collection<Document> = database.collection("migration");
        let current_migration = collection_migration
            .find_one(doc! {"name": "current"}, None)
            .await
            .expect("migration: failed to get migration collection");
        if let Some(current_migration) = current_migration {
            let version = current_migration
                .get("version")
                .expect("migration: missing version field")
                .as_i32()
                .expect("migration: failed to get version as i32") as u32;
            match version {
                0 => {
                    info!(
                        "migration: migrating from version: {}, to latest: {}....",
                        version, latest_migration_v
                    );
                    Self::migration_0(database).await?;
                }
                _ => {
                    info!("migration: version: {}", version);
                    return Ok(());
                }
            }
            info!(
                "migration: migration complete to version: {}",
                latest_migration_v
            );
            collection_migration
                .update_one(
                    doc! {
                        "name": "current"
                    },
                    doc! {
                        "$set": {
                            "version": latest_migration_v
                        }
                    },
                    None,
                )
                .await
                .expect("migration: failed to update version");
        } else {
            warn!(
                "migration: nothing found in migration collection, assumed to be latest: : {}",
                latest_migration_v
            );
            collection_migration
                .insert_one(
                    doc! {
                        "name": "current",
                        "version": latest_migration_v,
                        "created_at": bson::DateTime::now(),
                        "modified_at": bson::DateTime::now(),
                    },
                    None,
                )
                .await
                .expect("migration: failed to insert default version");
        }

        Ok(())
    }

    pub async fn migration_0(database: &Database) -> Result<(), MigrationError> {
        {
            let collection: Collection<Document> = database.collection("user");
            let mut cursor = collection.find(doc! {}, None).await?;

            while let Some(document) = cursor.try_next().await? {
                // let user: User = user.into();
                let _id = document.get("_id").expect("migration: failed to get _id");
                let created_at = document
                    .get("created_at")
                    .expect("migration: failed to get field")
                    .as_datetime()
                    .expect("migration: failed to get field");

                let modified_at = document
                    .get("modified_at")
                    .expect("migration: failed to get field")
                    .as_datetime()
                    .expect("migration: failed to get field");

                let new_created_at = created_at.timestamp_millis();
                let new_modified_at = modified_at.timestamp_millis();

                collection
                    .update_one(
                        doc! { "_id": _id },
                        doc! {
                            "$rename": {
                                "id": "author_id"
                            },
                        },
                        None,
                    )
                    .await
                    .expect("migration failed");
                collection
                    .update_one(
                        doc! { "_id": _id },
                        doc! {
                            "$set": {
                                "id": uuid::Uuid::new_v4().to_string(),
                                "created_at": new_created_at,
                                "modified_at": new_modified_at
                            },
                        },
                        None,
                    )
                    .await
                    .expect("migration failed");
            }
        }

        {
            let collection: Collection<Document> = database.collection("img");
            let mut cursor = collection.find(doc! {}, None).await?;

            while let Some(document) = cursor.try_next().await? {
                // let user: User = user.into();
                let _id = document.get("_id").expect("migration: failed to get _id");
                let created_at = document
                    .get("created_at")
                    .expect("migration: failed to get field")
                    .as_datetime()
                    .expect("migration: failed to get field");

                let modified_at = document
                    .get("modified_at")
                    .expect("migration: failed to get field")
                    .as_datetime()
                    .expect("migration: failed to get field");

                let new_created_at = created_at.timestamp_millis();
                let new_modified_at = modified_at.timestamp_millis();

                collection
                    .update_one(
                        doc! { "_id": _id },
                        doc! {
                           "$rename": {
                                "id": "msg_id"
                            },
                        },
                        None,
                    )
                    .await
                    .expect("migration failed");

                collection
                    .update_one(
                        doc! { "_id": _id },
                        doc! {
                            "$set": {
                                "id": uuid::Uuid::new_v4().to_string(),
                                "created_at": new_created_at,
                                "modified_at": new_modified_at
                            },
                        },
                        None,
                    )
                    .await
                    .expect("migration failed");
            }
        }

        {
            let collection: Collection<Document> = database.collection("auto_reaction");
            let mut cursor = collection.find(doc! {}, None).await?;

            while let Some(document) = cursor.try_next().await? {
                // let user: User = user.into();
                let _id = document.get("_id").expect("migration: failed to get _id");
                let created_at = document
                    .get("created_at")
                    .expect("migration: failed to get field")
                    .as_datetime()
                    .expect("migration: failed to get field");

                let modified_at = document
                    .get("modified_at")
                    .expect("migration: failed to get field")
                    .as_datetime()
                    .expect("migration: failed to get field");

                let new_created_at = created_at.timestamp_millis();
                let new_modified_at = modified_at.timestamp_millis();

                collection
                    .update_one(
                        doc! { "_id": _id },
                        doc! {
                           "$rename": {
                                "id": "emoji_id"
                            },
                        },
                        None,
                    )
                    .await
                    .expect("migration failed");

                collection
                    .update_one(
                        doc! { "_id": _id },
                        doc! {
                            "$set": {
                                "id": uuid::Uuid::new_v4().to_string(),
                                "created_at": new_created_at,
                                "modified_at": new_modified_at
                            },
                        },
                        None,
                    )
                    .await
                    .expect("migration failed");
            }
        }
        {
            let collection: Collection<Document> = database.collection("allowed_role");
            let mut cursor = collection.find(doc! {}, None).await?;

            while let Some(document) = cursor.try_next().await? {
                // let user: User = user.into();
                let _id = document.get("_id").expect("migration: failed to get _id");
                let created_at = document
                    .get("created_at")
                    .expect("migration: failed to get field")
                    .as_datetime()
                    .expect("migration: failed to get field");

                let modified_at = document
                    .get("modified_at")
                    .expect("migration: failed to get field")
                    .as_datetime()
                    .expect("migration: failed to get field");

                let new_created_at = created_at.timestamp_millis();
                let new_modified_at = modified_at.timestamp_millis();

                collection
                    .update_one(
                        doc! { "_id": _id },
                        doc! {
                           "$rename": {
                                "id": "role_id"
                            },
                        },
                        None,
                    )
                    .await
                    .expect("migration failed");

                collection
                    .update_one(
                        doc! { "_id": _id },
                        doc! {
                            "$set": {
                                "id": uuid::Uuid::new_v4().to_string(),
                                "created_at": new_created_at,
                                "modified_at": new_modified_at
                            },
                        },
                        None,
                    )
                    .await
                    .expect("migration failed");
            }
        }
        {
            let collection: Collection<Document> = database.collection("allowed_guild");
            let mut cursor = collection.find(doc! {}, None).await?;

            while let Some(document) = cursor.try_next().await? {
                // let user: User = user.into();
                let _id = document.get("_id").expect("migration: failed to get _id");
                let created_at = document
                    .get("created_at")
                    .expect("migration: failed to get field")
                    .as_datetime()
                    .expect("migration: failed to get field");

                let modified_at = document
                    .get("modified_at")
                    .expect("migration: failed to get field")
                    .as_datetime()
                    .expect("migration: failed to get field");

                let new_created_at = created_at.timestamp_millis();
                let new_modified_at = modified_at.timestamp_millis();

                collection
                    .update_one(
                        doc! { "_id": _id },
                        doc! {
                           "$rename": {
                                "id": "guild_id"
                            },
                        },
                        None,
                    )
                    .await
                    .expect("migration failed");

                collection
                    .update_one(
                        doc! { "_id": _id },
                        doc! {
                            "$set": {
                                "id": uuid::Uuid::new_v4().to_string(),
                                "created_at": new_created_at,
                                "modified_at": new_modified_at
                            },
                        },
                        None,
                    )
                    .await
                    .expect("migration failed");
            }
        }
        {
            let collection: Collection<Document> = database.collection("allowed_channel");
            let mut cursor = collection.find(doc! {}, None).await?;

            while let Some(document) = cursor.try_next().await? {
                // let user: User = user.into();
                let _id = document.get("_id").expect("migration: failed to get _id");
                let created_at = document
                    .get("created_at")
                    .expect("migration: failed to get field")
                    .as_datetime()
                    .expect("migration: failed to get field");

                let modified_at = document
                    .get("modified_at")
                    .expect("migration: failed to get field")
                    .as_datetime()
                    .expect("migration: failed to get field");

                let new_created_at = created_at.timestamp_millis();
                let new_modified_at = modified_at.timestamp_millis();

                collection
                    .update_one(
                        doc! { "_id": _id },
                        doc! {
                           "$rename": {
                                "id": "channel_id"
                            },
                        },
                        None,
                    )
                    .await
                    .expect("migration failed");

                collection
                    .update_one(
                        doc! { "_id": _id },
                        doc! {
                            "$set": {
                                "id": uuid::Uuid::new_v4().to_string(),
                                "created_at": new_created_at,
                                "modified_at": new_modified_at
                            },
                        },
                        None,
                    )
                    .await
                    .expect("migration failed");
            }
        }

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum MigrationError {
    #[error("Mongodb: {0}.")]
    Mongo(#[from] mongodb::error::Error),

    #[error("Bson: {0}.")]
    Bson(#[from] bson::binary::Error),
}
