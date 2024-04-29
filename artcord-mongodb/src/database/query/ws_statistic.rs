use crate::database::{DBError, DB};
use artcord_state::model::ws_statistics::{WsStat, WsStatFieldName};
use bson::{doc, Document};
use futures::TryStreamExt;
use mongodb::{
    options::{FindOptions, IndexOptions},
    Collection, Database, IndexModel,
};
use tracing::{debug, error};

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
        from: Option<i64>,
    ) -> Result<u64, mongodb::error::Error> {
        let mut pipeline = if let Some(from) = from {
            vec![
                doc! { "$sort": doc! { WsStatFieldName::CreatedAt.name(): -1 } },
                doc! { "$match": doc! { WsStatFieldName::CreatedAt.name(): { "$lt": from } } },
                doc! { "$count": "total" },
            ]
        } else {
            vec![doc! { "$count": "total" }]
        };

        let stats = self
            .collection_ws_statistic
            .aggregate(pipeline, None)
            .await?;
        let stats = stats.try_collect().await.unwrap_or_else(|_| vec![]);

        let count = stats
            .first()
            .map(|count| {
                count
                    .get_i32("total")
                    .map(|count| count as u64)
                    .inspect_err(|err| error!("error getting ws stats page count field: {}", err))
                    .unwrap_or(0)
            })
            .unwrap_or(0);
        // for stat in stats {
        //     let count = stat
        // }

        Ok(count)
    }

    pub async fn ws_statistic_with_pagination_latest(
        &self,
        page: u64,
        amount: u64,
        //   from: Option<i64>,
    ) -> Result<(u64, Option<i64>, Vec<WsStat>), DBError> {
        let amount = amount.clamp(1, 10000);

        let mut stats_pipeline = vec![doc! {
            "$sort": { WsStatFieldName::CreatedAt.name(): -1 }
        }];

        if page > 0 {
            let skip = (page * amount) as i64;

            stats_pipeline.push(doc! { "$skip": skip});
        }
        let limit = amount as i64;
        stats_pipeline.push(doc! { "$limit":  limit});

        let mut pipeline = vec![doc! {
            "$facet": {
                "total_count": [
                    {
                        "$sort": {
                            "created_at": -1
                        }
                    },
                    {
                        "$count": "total_count"
                    }
                ],
                "latest": [
                    {
                        "$sort": {
                            "created_at": -1
                        }
                    },
                    {
                        "$limit": 1
                    },
                    {
                        "$project": {
                            "created_at": 1
                        }
                    }
                ],
                "stats": stats_pipeline
            }

        }];

        // if let Some(from) = from {
        //     pipeline.push(doc! { "$match": doc! { WsStatFieldName::CreatedAt.name(): { "$lt": from } } });
        // }

        // println!("{:#?}", pipeline);

        // let mut stats = self
        //     .collection_ws_statistic
        //     .aggregate(pipeline, None)
        //     .await?;

        let stats_with_paginatoin = self
            .collection_ws_statistic
            .aggregate(pipeline, None)
            .await?;
        let stats_with_paginatoin = stats_with_paginatoin
            .try_collect()
            .await
            .unwrap_or_else(|_| vec![]);
        debug!("STATS: {:#?}", &stats_with_paginatoin);

        let mut output: Vec<WsStat> = Vec::new();

        let Some(stats_with_paginatoin) = stats_with_paginatoin.first() else {
            return Ok((0, None, output));
        };

        let total_count = stats_with_paginatoin.get_array("total_count").map(|v| {
            v.first()
                .map(|v| {
                    v.as_document()
                        .map(|v| v.get("total_count").map(|v| v.as_i32()))
                })
                .flatten()
                .flatten()
                .flatten()
        })?.unwrap_or(0) as u64;

        let latest = stats_with_paginatoin
            .get_array("latest")
            .map(|v| {
                v.first()
                    .map(|v| {
                        v.as_document()
                            .map(|v| v.get("created_at").map(|v| v.as_i64()))
                    })
                    .flatten()
                    .flatten()
                    .flatten()
            })?;
        
        let stats = stats_with_paginatoin
            .get_array("stats")
            .map(|v| {
                v.iter().map(|v| {
                    v.as_document()
                        .map(|v| mongodb::bson::from_document::<WsStat>(v.clone()).map_err(|err| DBError::from(err)))
                }).flatten()
            }).map(|v| v.collect::<Result<Vec<WsStat>, DBError>>())??;

        // while let Some(result) = stats.try_next().await? {   bson::de::Error
        //     let doc: WsStat = mongodb::bson::from_document(result)?;
        //     //let a = doc.f
        //     output.push(doc);
        //     // println!("hh");
        // }

        Ok((total_count, latest, stats))
        // let opts = FindOptions::builder()
        //     .sort(doc! { WsStatFieldName::CreatedAt.name(): -1 })
        //     .build();
        // let result = self.collection_ws_statistic.find(doc! {}, opts).await?;
        // let result = result.try_collect().await.unwrap_or_else(|_| vec![]);
        //
        // Ok(result)
    }

    pub async fn ws_statistic_paged_latest(
        &self,
        page: u64,
        amount: u64,
        from: i64,
    ) -> Result<Vec<WsStat>, mongodb::error::Error> {
        let amount = amount.clamp(1, 10000);
        let mut pipeline = vec![doc! { "$sort": doc! { WsStatFieldName::CreatedAt.name(): -1 } }];

        pipeline.push(
            doc! { "$match": doc! { WsStatFieldName::CreatedAt.name(): { "$lt": from } } },
        );

        if page > 0 {
            let skip = (page * amount) as i64;

            pipeline.push(doc! { "$skip": skip});
        }
        let limit = amount as i64;
        pipeline.push(doc! { "$limit":  limit});
        println!("pipeline: ws_statistic_paged_latest: {:#?}", pipeline);

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