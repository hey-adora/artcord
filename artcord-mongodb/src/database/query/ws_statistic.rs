use std::str::FromStr;

use crate::database::{DBError, DB};
use artcord_state::{
    model::ws_statistics::{DbWsStat, DbWsStatFieldName}, util::DAY_IN_MS,
};
use bson::{doc, Document};
use chrono::naive::NaiveDate;
use chrono::{DateTime, Utc};
use futures::TryStreamExt;
use mongodb::{
    options::{FindOptions, IndexOptions},
    Collection, Database, IndexModel,
};
use tracing::{debug, error};

pub const COLLECTION_WS_STATISTIC_NAME: &'static str = "ws_statistic";

impl DB {
    pub async fn init_ws_statistic(database: &Database) -> Collection<DbWsStat> {
        let (index1) = (
            {
                let opts = IndexOptions::builder().unique(true).build();
                IndexModel::builder()
                    .keys(doc! { DbWsStatFieldName::Id.name(): -1 })
                    .options(opts)
                    .build()
            },
            {
                let opts = IndexOptions::builder().unique(true).build();
                IndexModel::builder()
                    .keys(doc! { DbWsStatFieldName::ConId.name(): -1 })
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

        let collection = database.collection::<DbWsStat>(COLLECTION_WS_STATISTIC_NAME);

        collection
            .create_indexes([index1.0, index1.1], None)
            .await
            .expect("Failed to create collection index.");

        collection
    }
}

impl DB {
    pub async fn ws_statistic_insert_one(
        &self,
        statistic: DbWsStat,
    ) -> Result<String, mongodb::error::Error> {
        let result = self
            .collection_ws_statistic
            .insert_one(statistic, None)
            .await?;

        Ok(result.inserted_id.to_string())
    }

    pub async fn ws_statistic_insert_many(
        &self,
        statistics: Vec<DbWsStat>,
    ) -> Result<(), mongodb::error::Error> {
        let _ = self
            .collection_ws_statistic
            .insert_many(statistics, None)
            .await?;

        // Ok(result.inserted_ids)
        Ok(())
    }

    // pub async fn ws_statistic_update_disconnect(
    //     &self,
    //     id: String,
    // ) -> Result<(), mongodb::error::Error> {
    //     let _ = self
    //         .collection_ws_statistic
    //         .update_one(
    //             doc! { WsStatFieldName::Id.name(): id},
    //             doc! {
    //                 "$set": {
    //                     WsStatFieldName::IsConnected.name(): false
    //                 }
    //             },
    //             None,
    //         )
    //         .await?;

    //     Ok(())
    // }

    pub async fn ws_statistic_all_latest(&self) -> Result<Vec<DbWsStat>, mongodb::error::Error> {
        let opts = FindOptions::builder()
            .sort(doc! { DbWsStatFieldName::CreatedAt.name(): -1 })
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
                doc! { "$sort": doc! { DbWsStatFieldName::CreatedAt.name(): -1 } },
                doc! { "$match": doc! { DbWsStatFieldName::CreatedAt.name(): { "$lt": from } } },
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
    ) -> Result<(u64, Option<i64>, Vec<DbWsStat>), DBError> {
        let amount = amount.clamp(1, 10000);

        let mut stats_pipeline = vec![doc! {
            "$sort": { DbWsStatFieldName::CreatedAt.name(): -1 }
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
                            DbWsStatFieldName::CreatedAt.name(): -1
                        }
                    },
                    {
                        "$count": "total_count"
                    }
                ],
                "latest": [
                    {
                        "$sort": {
                            DbWsStatFieldName::CreatedAt.name(): -1
                        }
                    },
                    {
                        "$limit": 1
                    },
                    {
                        "$project": {
                            DbWsStatFieldName::CreatedAt.name(): 1
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
        //debug!("STATS: {:#?}", &stats_with_paginatoin);

        let mut output: Vec<DbWsStat> = Vec::new();

        let Some(stats_with_paginatoin) = stats_with_paginatoin.first() else {
            return Ok((0, None, output));
        };

        let total_count = stats_with_paginatoin
            .get_array("total_count")
            .map(|v| {
                v.first()
                    .map(|v| {
                        v.as_document()
                            .map(|v| v.get("total_count").map(|v| v.as_i32()))
                    })
                    .flatten()
                    .flatten()
                    .flatten()
            })?
            .unwrap_or(0) as u64;

        let latest = stats_with_paginatoin.get_array("latest").map(|v| {
            v.first()
                .map(|v| {
                    v.as_document()
                        .map(|v| v.get(DbWsStatFieldName::CreatedAt.name()).map(|v| v.as_i64()))
                })
                .flatten()
                .flatten()
                .flatten()
        })?;

        let stats = stats_with_paginatoin
            .get_array("stats")
            .map(|v| {
                v.iter()
                    .map(|v| {
                        v.as_document().map(|v| {
                            mongodb::bson::from_document::<DbWsStat>(v.clone())
                                .map_err(|err| DBError::from(err))
                        })
                    })
                    .flatten()
            })
            .map(|v| v.collect::<Result<Vec<DbWsStat>, DBError>>())??;

        // while let Some(result) = stats.try_next().await? {   bson::de::Error
        //     let doc: WsStat = mongodb::bson::from_document(result)?;
        //     //let a = doc.f
        //     output.push(doc);
        //     // println!("hh");
        // }
        //debug!("STATS: {:#?}", &stats);

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
    ) -> Result<Vec<DbWsStat>, mongodb::error::Error> {
        let amount = amount.clamp(1, 10000);
        let mut pipeline = vec![doc! { "$sort": doc! { DbWsStatFieldName::CreatedAt.name(): -1 } }];

        pipeline
            .push(doc! { "$match": doc! { DbWsStatFieldName::CreatedAt.name(): { "$lt": from } } });

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

        let mut output: Vec<DbWsStat> = Vec::new();

        while let Some(result) = stats.try_next().await? {
            let doc: DbWsStat = mongodb::bson::from_document(result)?;
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

    pub async fn ws_stats_graph(
        &self,
        from: i64,
        to: i64,
        unique_id: bool,
    ) -> Result<Vec<f64>, DBError> {
        let mut pipeline = vec![
            doc! { "$sort": { DbWsStatFieldName::CreatedAt.name(): 1 } },
            doc! { "$match": { DbWsStatFieldName::CreatedAt.name(): { "$lt": from, "$gt": to } } },
        ];

        if unique_id {
            pipeline.push(
                doc! { "$group": {
                    "_id": {
                        "date": {
                            "$dateToString": {
                                "date": {
                                    "$toDate": format!("${}", DbWsStatFieldName::CreatedAt.name()),
                                },
                                "format": "%Y-%m-%d"
                            }
                        },
                        "ip": "$ip",
                    },
                } },
            );
            pipeline.push(
                doc! { "$group": {
                    "_id": {
                        "date": "$_id.date",
                    },
                    "count": { "$count": { } }
                } },
            );
        } else {
            pipeline.push(
                doc! { "$group": {
                    "_id": {
                        "date": {
                            "$dateToString": {
                                "date": {
                                    "$toDate": format!("${}", DbWsStatFieldName::CreatedAt.name()),
                                },
                                "format": "%Y-%m-%d"
                            }
                        },
                    },
                    "count": { "$count": { } }
                } },
            );
        }

        pipeline.push(doc! { "$sort": { "_id.date": 1 } });

        //debug!("db: pipes: {:#?}", &pipeline);

        let mut stats: mongodb::Cursor<Document> = self
            .collection_ws_statistic
            .aggregate(pipeline, None)
            .await?;

        let stats = stats.try_collect().await.unwrap_or_else(|_| vec![]);

        let mut output: Vec<f64> = Vec::new();

        //let first = stats.first();
        // let Some(first) = first else {
        //     return Ok(output);
        // };

        // let created_at = first
        //         .get_document("_id")
        //         .and_then(|_id|_id.get_str("date"))
        //         .map(|date| {
        //             NaiveDate::parse_from_str(date, "%Y-%m-%d")
        //                 .map(|date| DateTime::<Utc>::from_naive_utc_and_offset(date.into(), Utc))
        //         })??
        //         .timestamp_millis();

     

        let mut prev_created_at: Option<i64> = None;
        for stat in stats {
            
            let created_at = stat
                .get_document("_id")
                .and_then(|_id|_id.get_str("date"))
                .map(|date| {
                    NaiveDate::parse_from_str(date, "%Y-%m-%d")
                        .map(|date| DateTime::<Utc>::from_naive_utc_and_offset(date.into(), Utc))
                })??
                .timestamp_millis();
            let count = stat.get_i32("count")? as f64;

            //debug!("db: graph: {}", stat);

            

            if let Some(prev_created_at) = prev_created_at {
                //debug!("({} - {}) / {} > 1 = {} | {} {}", created_at, prev_created_at, DAY_IN_MS, (created_at - prev_created_at) / DAY_IN_MS > 1, created_at - prev_created_at, (created_at - prev_created_at) / DAY_IN_MS);
                if (created_at - prev_created_at) / DAY_IN_MS > 1 {
                    for day_i in (prev_created_at + DAY_IN_MS..created_at).step_by(DAY_IN_MS as usize) {
                        output.push(day_i as f64);
                        output.push(0.0);
                    }
                }
            }

            output.push(created_at as f64);
            output.push(count);
            prev_created_at = Some(created_at);
            //output.push(doc);
        }

        //let created_at = 1715990400000;
        
        if let Some(created_at) = prev_created_at {
            let diff = from - created_at;
            if diff > 0 {
                for day_i in (created_at + DAY_IN_MS..from).step_by(DAY_IN_MS as usize) {
                    output.push(day_i as f64);
                    output.push(0.0);
                }
            }
        } else {
            for day_i in (to..from).step_by(DAY_IN_MS as usize) {
                output.push(day_i as f64);
                output.push(0.0);
            }
        }

        //debug!("db: graph: {}", stat);
        Ok(output)
    }
}
