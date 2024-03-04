use std::collections::HashMap;
use std::sync::Arc;
use serenity::prelude::TypeMapKey;
use tokio::sync::RwLock;
use crate::database::models::allowed_role::AllowedRole;

impl TypeMapKey for AllowedRole {
    type Value = Arc<RwLock<HashMap<String, Self>>>;
}
