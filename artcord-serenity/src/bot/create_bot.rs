use crate::bot::events;
use crate::database::create_database::DB;
use crate::database::models::auto_reaction::AutoReaction;
use serenity::client::Context;
use serenity::framework::StandardFramework;
use serenity::model::channel::Reaction;
use serenity::model::id::{ChannelId, GuildId, MessageId};
use serenity::model::prelude::Interaction;
use serenity::prelude::{GatewayIntents, TypeMapKey};
use serenity::{async_trait, Client};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ArcStr;
impl TypeMapKey for ArcStr {
    type Value = Arc<str>;
}

struct BotHandler;

#[async_trait]
impl serenity::client::EventHandler for BotHandler {
    async fn reaction_remove(&self, ctx: Context, remove_reaction: Reaction) {
        events::reaction_remove::reaction_remove(ctx, remove_reaction).await;
    }

    async fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
        events::reaction_add::reaction_add(ctx, add_reaction).await;
    }

    async fn message(&self, ctx: Context, msg: serenity::model::channel::Message) {
        events::message::message(ctx, msg).await;
    }

    async fn message_delete(
        &self,
        ctx: Context,
        _channel_id: ChannelId,
        deleted_message_id: MessageId,
        guild_id: Option<GuildId>,
    ) {
        events::message_delete::message_delete(ctx, _channel_id, deleted_message_id, guild_id)
            .await;
    }

    async fn message_delete_bulk(
        &self,
        ctx: Context,
        _channel_id: ChannelId,
        multiple_deleted_messages_id: Vec<MessageId>,
        guild_id: Option<GuildId>,
    ) {
        events::message_delete_bulk::message_delete_bulk(
            ctx,
            _channel_id,
            multiple_deleted_messages_id,
            guild_id,
        )
        .await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        events::interaction_create::interaction_create(ctx, interaction).await;
    }

    async fn ready(&self, ctx: Context, ready: serenity::model::gateway::Ready) {
        events::ready::ready(ctx, ready).await;
    }
}

pub struct ReactionQueue {
    pub msg_id: u64,
    pub channel_id: u64,
    pub reactions: Vec<AutoReaction>,
    pub add: bool,
}

impl ReactionQueue {
    pub fn new(channel_id: u64, msg_id: u64, add: bool) -> Self {
        Self {
            msg_id,
            channel_id,
            reactions: Vec::new(),
            add,
        }
    }
}

impl TypeMapKey for ReactionQueue {
    type Value = Arc<RwLock<HashMap<u64, Self>>>;
}

pub async fn create_bot(db: Arc<DB>, token: String, gallery_root_dir: &str) -> serenity::Client {
    let framework = StandardFramework::new();

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::GUILD_MESSAGE_REACTIONS
        | GatewayIntents::MESSAGE_CONTENT;
    let client = Client::builder(token, intents)
        .event_handler(BotHandler)
        .framework(framework)
        .await
        .expect("Error creating client");

    let reaction_queue = Arc::new(RwLock::new(HashMap::new()));
    {
        let mut data = client.data.write().await;
        data.insert::<DB>(db);
        data.insert::<ReactionQueue>(reaction_queue);
        data.insert::<ArcStr>(Arc::from(gallery_root_dir));
    }

    client
}
