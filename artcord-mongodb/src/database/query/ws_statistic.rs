use crate::database::DB;
use artcord_state::model::ws_statistics::{WsStat, WsStatFieldName};
use bson::{doc, Document};
use futures::TryStreamExt;
use mongodb::{
    options::{FindOptions, IndexOptions},
    Collection, Database, IndexModel,
};

pub const COLLECTION_WS_STATISTIC_NAME: &'static str = "ws_statistic";

impl DB {
    pub async fn init_ws_statistic(database: &Database) -> Collection<WsStat> {
        let (index1) = (
            {
                let opts = IndexOptions::builder().unique(true).build();
                IndexModel::builder()
                    .keys(doc! { WsStatFieldName::Id.name(): -1 })
                    .options(opts)
                    .build()
            },
            // {
            //     let opts = IndexOptions::builder().unique(true).build();
            //     IndexModel::builder()
            //         .keys(doc! { UserFieldName::AuthorId.name(): -1 })
            //         .options(opts)
            //         .build()
            // },
        );

        let collection = database.collection::<WsStat>(COLLECTION_WS_STATISTIC_NAME);

        collection
            .create_indexes([index1.0], None)
            .await
            .expect("Failed to create collection index.");

        collection
    }
}

impl DB {
    pub async fn ws_statistic_insert_one(
        &self,
        statistic: WsStat,
    ) -> Result<String, mongodb::error::Error> {
        let result = self
            .collection_ws_statistic
            .insert_one(statistic, None)
            .await?;

        Ok(result.inserted_id.to_string())
    }

    pub async fn ws_statistic_insert_many(
        &self,
        statistics: Vec<WsStat>,
    ) -> Result<(), mongodb::error::Error> {
        let _ = self
            .collection_ws_statistic
            .insert_many(statistics, None)
            .await?;

        // Ok(result.inserted_ids)
        Ok(())
    }

    pub async fn ws_statistic_update_disconnect(
        &self,
        id: String,
    ) -> Result<(), mongodb::error::Error> {
        let _ = self
            .collection_ws_statistic
            .update_one(
                doc! { WsStatFieldName::Id.name(): id},
                doc! {
                    "$set": {
                        WsStatFieldName::IsConnected.name(): false
                    }
                },
                None,
            )
            .await?;

        Ok(())
    }

    pub async fn ws_statistic_all_latest(&self) -> Result<Vec<WsStat>, mongodb::error::Error> {
        let opts = FindOptions::builder()
            .sort(doc! { WsStatFieldName::CreatedAt.name(): -1 })
            .build();
        let result = self.collection_ws_statistic.find(doc! {}, opts).await?;
        let result = result.try_collect().await.unwrap_or_else(|_| vec![]);

        Ok(result)
    }

    // pub async fn user_find_one(
    //     &self,
    //     user_id: &str,
    // ) -> Result<Option<User>, mongodb::error::Error> {
    //     let result = self
    //         .collection_user
    //         .find_one(doc! {UserFieldName::AuthorId.name(): user_id}, None)
    //         .await?;
    //     //println!("wtf{:?}", user);
    //     // Ok(ServerMsg::Profile(user))
    //     Ok(result)
    // }
}
