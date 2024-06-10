use std::collections::HashMap;
use std::net::IpAddr;
use std::num::TryFromIntError;
use std::str::FromStr;

use crate::database::{DBError, COLLECTION_WS_IP_NAME, DB};
use artcord_state::global;
use bson::{doc, Document};
use chrono::naive::NaiveDate;
use chrono::{DateTime, Utc};
use field_types::FieldName;
use futures::TryStreamExt;
use mongodb::{
    options::{FindOptions, IndexOptions, UpdateOptions},
    Collection, Database, IndexModel,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use strum::VariantNames;
use thiserror::Error;
use tracing::{debug, error};

impl DB {
    pub async fn init_ws_ip(database: &Database) -> Collection<global::DbWsIp> {
        let (index1) = (
            {
                let opts = IndexOptions::builder().unique(true).build();
                IndexModel::builder()
                    .keys(doc! { global::DbWsIpFieldName::Id.name(): -1 })
                    .options(opts)
                    .build()
            },
            {
                let opts = IndexOptions::builder().unique(true).build();
                IndexModel::builder()
                    .keys(doc! { global::DbWsIpFieldName::Ip.name(): -1 })
                    .options(opts)
                    .build()
            },
        );

        let collection = database.collection::<global::DbWsIp>(COLLECTION_WS_IP_NAME);

        collection
            .create_indexes([index1.0, index1.1], None)
            .await
            .expect("Failed to create collection index.");

        collection
    }

    pub async fn ws_ip_upsert(
        &self,
        ip: &IpAddr,
        total_allow_amount: u64,
        total_block_amount: u64,
        total_banned_amount: u64,
        total_already_banned_amount: u64,
        con_count_tracker: global::ThresholdTracker,
        con_flicker_tracker: global::ThresholdTracker,
        banned_until: global::BanType,
        time: &DateTime<Utc>,
    ) -> Result<u64, DBError> {
        let db_ws_ip = self
            .collection_ws_ip
            .find_one(
                doc! { global::DbWsIpFieldName::Ip.name(): ip.to_string() },
                None,
            )
            .await?;

        let Some(ws_ip) = db_ws_ip else {
            let ws_ip = global::DbWsIp::try_new(
                *ip,
                total_allow_amount,
                total_block_amount,
                total_banned_amount,
                total_already_banned_amount,
                con_count_tracker,
                con_flicker_tracker,
                banned_until,
                *time,
            )?;
            let _ = self
                .collection_ws_ip
                .insert_one(ws_ip, None)
                .await?;
            return Ok(1);
        };

        let con_count_tracker = bson::to_bson(&global::DbThresholdTracker::try_from(con_count_tracker)?)?;
        let con_flicker_tracker = bson::to_bson(&global::DbThresholdTracker::try_from(con_flicker_tracker)?)?;
        let result = self
            .collection_ws_ip
            .update_one(
                doc! {
                    global::DbWsIpFieldName::Ip.name(): ws_ip.ip,
                },
                doc! {
                    "$set": {
                        global::DbWsIpFieldName::TotalAllowAmount.name() : total_allow_amount as i64,
                        global::DbWsIpFieldName::TotalBlockAmount.name() : total_block_amount as i64,
                        global::DbWsIpFieldName::TotalBannedAmount.name() : total_banned_amount as i64,
                        global::DbWsIpFieldName::TotalAlreadyBannedAmount.name() : total_already_banned_amount as i64,
                        global::DbWsIpFieldName::ConCountTracker.name() : con_count_tracker,
                        global::DbWsIpFieldName::ConFlickerTracker.name() : con_flicker_tracker,
                        global::DbWsIpFieldName::ModifiedAt.name(): time.timestamp_millis(),
                    }
                },
                None,
            )
            .await?;

        Ok(result.modified_count)
    }

    pub async fn ws_ip_find_one_by_ip(
        &self,
        ip: IpAddr,
    ) -> Result<Option<global::SavedWsIp>, DBError> {
        let ip = ip.to_string();
        let result = self
            .collection_ws_ip
            .find_one(doc! { global::DbWsIpFieldName::Ip.name(): ip }, None)
            .await?;

        Ok(match result {
            Some(result) => {
                let result: global::SavedWsIp = result.try_into()?;
                Some(result)
            }
            None => None,
        })
    }
}

