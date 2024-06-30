use artcord_state::global;

impl crate::database::DB {
    pub async fn init_http_ip(database: &mongodb::Database) -> mongodb::Collection<artcord_state::global::DbHttpIp> {
        let (index1) = (
            {
                let opts = mongodb::options::IndexOptions::builder().unique(true).build();
                mongodb::IndexModel::builder()
                    .keys(bson::doc! { artcord_state::global::DbHttpIpFieldName::Id.name(): -1 })
                    .options(opts)
                    .build()
            },
            {
                let opts = mongodb::options::IndexOptions::builder().unique(true).build();
                mongodb::IndexModel::builder()
                    .keys(bson::doc! { artcord_state::global::DbHttpIpFieldName::Ip.name(): -1 })
                    .options(opts)
                    .build()
            },
        );
    
        let collection = database.collection::<artcord_state::global::DbHttpIp>(crate::database::COLLECTION_HTTP_IP_NAME);
    
        collection
            .create_indexes([index1.0, index1.1], None)
            .await
            .expect("Failed to create collection index.");
    
        collection
    }

    pub async fn http_ip_upser(&self,
        ip: &std::net::IpAddr,
        block_tracker: artcord_state::global::ThresholdTracker,
        ban_tracker: artcord_state::global::ThresholdTracker,
        banned_until: artcord_state::global::BanType,
        time: &chrono::DateTime<chrono::Utc>,
    ) -> Result<u64, crate::database::DBError> {
        let ip_str = ip.to_string();

        let db_ws_ip = self.collection_http_ip.find_one(bson::doc! {
            artcord_state::global::DbHttpIpFieldName::Ip.name(): &ip_str,
        }, None).await?;

        let Some(_) = db_ws_ip else {
            let http_ip = global::DbHttpIp::try_new(*ip, block_tracker, ban_tracker, banned_until, *time)?;
            let _ = self.collection_http_ip.insert_one(http_ip, None).await?;
            return Ok(1);
        };

        let block_tracker = bson::to_bson(&global::DbThresholdTracker::try_from(block_tracker)?)?;
        let ban_tracker = bson::to_bson(&global::DbThresholdTracker::try_from(ban_tracker)?)?;

        let result = self.collection_http_ip.update_one(bson::doc! {
            artcord_state::global::DbHttpIpFieldName::Ip.name(): &ip_str,
        }, bson::doc! {
            "$set": {
                global::DbHttpIpFieldName::BlockTracker.name(): block_tracker,
                global::DbHttpIpFieldName::BanTracker.name(): ban_tracker,
                global::DbHttpIpFieldName::ModifiedAt.name(): time.timestamp_millis(),
            }
        }, None).await?;

        Ok(result.modified_count)
    }


    pub async fn http_ip_find_one_by_ip(&self, ip: std::net::IpAddr) -> Result<Option<artcord_state::global::SavedHttpIp>, crate::database::DBError> {
        let ip = ip.to_string();
        let result = self.collection_http_ip.find_one(bson::doc! {
            artcord_state::global::DbHttpIpFieldName::Ip.name(): ip,
        }, None).await?;
        
        Ok(
            match result {
                Some(result) => {
                    let result: artcord_state::global::SavedHttpIp = result.try_into()?;
                    Some(result)
                }
                None => None
            }
        )
    }
}
