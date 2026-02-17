use async_trait::async_trait;
use crate::model::{Message, Ready, Guild};
use crate::client::Context;

#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn on_ready(&self, _ctx: Context, _ready: Ready) {}

    async fn on_message(&self, _ctx: Context, _msg: Message) {}

    async fn on_guild_create(&self, _ctx: Context, _guild: Guild) {}
}