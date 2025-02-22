use std::collections::HashMap;
use std::net::IpAddr;
use std::num::TryFromIntError;
use std::str::FromStr;

use crate::database::{DBError, COLLECTION_WS_IP_MANAGER_NAME, DB};
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
    pub async fn init_ws_ip_manager(database: &Database) -> Collection<global::DbWsIpManager> {
        let (index1) = (
            {
                let opts = IndexOptions::builder().unique(true).build();
                IndexModel::builder()
                    .keys(doc! { global::DbWsIpManagerFieldName::Id.name(): -1 })
                    .options(opts)
                    .build()
            },
            {
                let opts = IndexOptions::builder().unique(true).build();
                IndexModel::builder()
                    .keys(doc! { global::DbWsIpManagerFieldName::Ip.name(): -1 })
                    .options(opts)
                    .build()
            },
        );

        let collection =
            database.collection::<global::DbWsIpManager>(COLLECTION_WS_IP_MANAGER_NAME);

        collection
            .create_indexes([index1.0, index1.1], None)
            .await
            .expect("Failed to create collection index.");

        collection
    }

    pub async fn ws_ip_manager_upsert(
        &self,
        ip: IpAddr,
        req_stats: HashMap<global::ClientPathType, global::WsConReqStat>,
        time: &DateTime<Utc>,
    ) -> Result<(), DBError> {
        let db_ip_manager = self
            .collection_ws_ip_manager
            .find_one(
                doc! { global::DbWsIpManagerFieldName::Ip.name(): ip.to_string() },
                None,
            )
            .await?;

        let Some(ws_ip_manager) = db_ip_manager else {
            let new_db_ip_manager = global::DbWsIpManager::try_new(ip, req_stats, *time)?;
            let _ = self.collection_ws_ip_manager.insert_one(new_db_ip_manager, None).await?;
            return Ok(());
        };

        let req_stats = global::req_stats_to_db(req_stats)?;
        let req_stats = bson::to_bson(&req_stats)?;

        let _ = self.collection_ws_ip_manager
            .update_one(
                doc! {
                    global::DbWsIpManagerFieldName::Ip.name(): ws_ip_manager.ip,
                },
                doc! {
                    "$set": {
                        global::DbWsIpManagerFieldName::ReqStats.name() : req_stats,
                        global::DbWsIpManagerFieldName::ModifiedAt.name(): time.timestamp_millis(),
                    }
                },
                None,
            )
            .await?;

        Ok(())
    }

    pub async fn ws_ip_manager_find_one_by_ip(
        &self,
        ip: IpAddr,
    ) -> Result<Option<global::SavedWsIpManager>, DBError> {
        let ip = ip.to_string();
        let result = self
            .collection_ws_ip_manager
            .find_one(doc! { global::DbWsIpManagerFieldName::Ip.name(): ip }, None)
            .await?;

        Ok(match result {
            Some(result) => {
                let result: global::SavedWsIpManager = result.try_into()?;
                Some(result)
            }
            None => None,
        })
    }
}
