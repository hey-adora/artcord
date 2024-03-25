use std::collections::HashMap;
use std::sync::Arc;
use serenity::prelude::TypeMapKey;
use tokio::sync::RwLock;
use artcord_state::model::allowed_role::AllowedRole;

impl TypeMapKey for AllowedRole {
    type Value = Arc<RwLock<HashMap<String, Self>>>;
}
