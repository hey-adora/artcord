use std::collections::HashMap;
use std::net::IpAddr;
use std::num::TryFromIntError;
use std::str::FromStr;

use crate::database::{DBError, COLLECTION_WS_STATISTIC_NAME, DB};
use artcord_state::global;
use bson::{doc, Document};
use chrono::naive::NaiveDate;
use chrono::{DateTime, Utc};
use field_types::FieldName;
use futures::TryStreamExt;
use mongodb::{
    options::{FindOptions, IndexOptions},
    Collection, Database, IndexModel,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error};
use std::net::SocketAddr;
use thiserror::Error;
use strum::VariantNames;

impl DB {
    pub async fn init_ws_statistic(database: &Database) -> Collection<global::DbWsCon> {
        let (index1) = (
            {
                let opts = IndexOptions::builder().unique(true).build();
                IndexModel::builder()
                    .keys(doc! { global::DbWsConFieldName::Id.name(): -1 })
                    .options(opts)
                    .build()
            },
            {
                let opts = IndexOptions::builder().unique(true).build();
                IndexModel::builder()
                    .keys(doc! { global::DbWsConFieldName::ConId.name(): -1 })
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

        let collection = database.collection::<global::DbWsCon>(&COLLECTION_WS_STATISTIC_NAME);

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
        statistic: global::DbWsCon,
    ) -> Result<String, mongodb::error::Error> {
        let result = self
            .collection_ws_statistic
            .insert_one(statistic, None)
            .await?;

        Ok(result.inserted_id.to_string())
    }

    pub async fn ws_statistic_insert_many(
        &self,
        statistics: Vec<global::DbWsCon>,
    ) -> Result<(), mongodb::error::Error> {
        let _ = self
            .collection_ws_statistic
            .insert_many(statistics, None)
            .await?;

        Ok(())
    }

    pub async fn ws_statistic_all_latest(&self) -> Result<Vec<global::DbWsCon>, mongodb::error::Error> {
        let opts = FindOptions::builder()
            .sort(doc! { global::DbWsConFieldName::CreatedAt.name(): -1 })
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
                doc! { "$sort": doc! { global::DbWsConFieldName::CreatedAt.name(): -1 } },
                doc! { "$match": doc! { global::DbWsConFieldName::CreatedAt.name(): { "$lt": from } } },
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
        Ok(count)
    }

    pub async fn ws_statistic_with_pagination_latest(
        &self,
        page: u64,
        amount: u64,
    ) -> Result<(u64, Option<i64>, Vec<global::DbWsCon>), DBError> {
        let amount = amount.clamp(1, 10000);

        let mut stats_pipeline = vec![doc! {
            "$sort": { global::DbWsConFieldName::CreatedAt.name(): -1 }
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
                            global::DbWsConFieldName::CreatedAt.name(): -1
                        }
                    },
                    {
                        "$count": "total_count"
                    }
                ],
                "latest": [
                    {
                        "$sort": {
                            global::DbWsConFieldName::CreatedAt.name(): -1
                        }
                    },
                    {
                        "$limit": 1
                    },
                    {
                        "$project": {
                            global::DbWsConFieldName::CreatedAt.name(): 1
                        }
                    }
                ],
                "stats": stats_pipeline
            }

        }];

        let stats_with_paginatoin = self
            .collection_ws_statistic
            .aggregate(pipeline, None)
            .await?;
        let stats_with_paginatoin = stats_with_paginatoin
            .try_collect()
            .await
            .unwrap_or_else(|_| vec![]);

        let mut output: Vec<global::DbWsCon> = Vec::new();

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
                    v.as_document().map(|v| {
                        v.get(global::DbWsConFieldName::CreatedAt.name())
                            .map(|v| v.as_i64())
                    })
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
                            mongodb::bson::from_document::<global::DbWsCon>(v.clone())
                                .map_err(|err| DBError::from(err))
                        })
                    })
                    .flatten()
            })
            .map(|v| v.collect::<Result<Vec<global::DbWsCon>, DBError>>())??;

        Ok((total_count, latest, stats))
    }

    pub async fn ws_statistic_paged_latest(
        &self,
        page: u64,
        amount: u64,
        from: i64,
    ) -> Result<Vec<global::DbWsCon>, mongodb::error::Error> {
        let amount = amount.clamp(1, 10000);
        let mut pipeline =
            vec![doc! { "$sort": doc! { global::DbWsConFieldName::CreatedAt.name(): -1 } }];

        pipeline.push(
            doc! { "$match": doc! { global::DbWsConFieldName::CreatedAt.name(): { "$lt": from } } },
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

        let mut output: Vec<global::DbWsCon> = Vec::new();

        while let Some(result) = stats.try_next().await? {
            let doc: global::DbWsCon = mongodb::bson::from_document(result)?;
            output.push(doc);
        }

        Ok(output)
    }

    pub async fn ws_stats_graph(
        &self,
        from: i64,
        to: i64,
        unique_id: bool,
    ) -> Result<Vec<f64>, DBError> {
        let mut pipeline = vec![
            doc! { "$sort": { global::DbWsConFieldName::CreatedAt.name(): 1 } },
            doc! { "$match": { global::DbWsConFieldName::CreatedAt.name(): { "$lt": from, "$gt": to } } },
        ];

        if unique_id {
            pipeline.push(doc! { "$group": {
                "_id": {
                    "date": {
                        "$dateToString": {
                            "date": {
                                "$toDate": format!("${}", global::DbWsConFieldName::CreatedAt.name()),
                            },
                            "format": "%Y-%m-%d"
                        }
                    },
                    "ip": "$ip",
                },
            } });
            pipeline.push(doc! { "$group": {
                "_id": {
                    "date": "$_id.date",
                },
                "count": { "$count": { } }
            } });
        } else {
            pipeline.push(doc! { "$group": {
                "_id": {
                    "date": {
                        "$dateToString": {
                            "date": {
                                "$toDate": format!("${}", global::DbWsConFieldName::CreatedAt.name()),
                            },
                            "format": "%Y-%m-%d"
                        }
                    },
                },
                "count": { "$count": { } }
            } });
        }

        pipeline.push(doc! { "$sort": { "_id.date": 1 } });

        let mut stats: mongodb::Cursor<Document> = self
            .collection_ws_statistic
            .aggregate(pipeline, None)
            .await?;
        let stats = stats.try_collect().await.unwrap_or_else(|_| vec![]);
        let mut output: Vec<f64> = Vec::new();
        let mut prev_created_at: Option<i64> = None;

        for stat in stats {
            let created_at = stat
                .get_document("_id")
                .and_then(|_id| _id.get_str("date"))
                .map(|date| {
                    NaiveDate::parse_from_str(date, "%Y-%m-%d")
                        .map(|date| DateTime::<Utc>::from_naive_utc_and_offset(date.into(), Utc))
                })??
                .timestamp_millis();
            let count = stat.get_i32("count")? as f64;

            if let Some(prev_created_at) = prev_created_at {
                if (created_at - prev_created_at) / global::DAY_IN_MS > 1 {
                    for day_i in
                        (prev_created_at + global::DAY_IN_MS..created_at).step_by(global::DAY_IN_MS as usize)
                    {
                        output.push(day_i as f64);
                        output.push(0.0);
                    }
                }
            }

            output.push(created_at as f64);
            output.push(count);
            prev_created_at = Some(created_at);
        }

        if let Some(created_at) = prev_created_at {
            let diff = from - created_at;
            if diff > 0 {
                for day_i in (created_at + global::DAY_IN_MS..from).step_by(global::DAY_IN_MS as usize) {
                    output.push(day_i as f64);
                    output.push(0.0);
                }
            }
        } else {
            for day_i in (to..from).step_by(global::DAY_IN_MS as usize) {
                output.push(day_i as f64);
                output.push(0.0);
            }
        }

        Ok(output)
    }
}

