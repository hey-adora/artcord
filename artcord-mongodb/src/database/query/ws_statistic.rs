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

    pub async fn ws_statistic_total_amount(
        &self,
    ) -> Result<u64, mongodb::error::Error> {
        self.collection_ws_statistic.count_documents(doc! {}, None).await
    }

    pub async fn ws_statistic_paged_latest(
        &self,
        page: u64,
        amount: u64,
    ) -> Result<Vec<WsStat>, mongodb::error::Error> {
        let amount = amount.clamp(1, 10000);
        let mut pipeline = vec![doc! { "$sort": doc! { WsStatFieldName::CreatedAt.name(): -1 } }];
        if page > 0 {
            let skip = (page * amount) as i64;

            pipeline.push(doc! { "$skip": skip});
        }
        let limit = amount as i64;
        pipeline.push(doc! { "$limit":  limit});
        // println!("{:#?}", pipeline);

        let mut stats = self
            .collection_ws_statistic
            .aggregate(pipeline, None)
            .await?;

        let mut output: Vec<WsStat> = Vec::new();

        while let Some(result) = stats.try_next().await? {
            let doc: WsStat = mongodb::bson::from_document(result)?;
            //let a = doc.f
            output.push(doc);
            // println!("hh");
        }

        Ok(output)
        // let opts = FindOptions::builder()
        //     .sort(doc! { WsStatFieldName::CreatedAt.name(): -1 })
        //     .build();
        // let result = self.collection_ws_statistic.find(doc! {}, opts).await?;
        // let result = result.try_collect().await.unwrap_or_else(|_| vec![]);
        //
        // Ok(result)
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
