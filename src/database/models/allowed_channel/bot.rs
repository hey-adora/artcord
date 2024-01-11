use crate::database::models::allowed_channel::AllowedChannel;
use serenity::prelude::TypeMapKey;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

impl TypeMapKey for AllowedChannel {
    type Value = Arc<RwLock<HashMap<String, Self>>>;
}
