use async_trait::async_trait;
use crate::model::Message;
use crate::client::Context;

#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn on_message(&self, ctx: Context, msg: Message) {
        let _ = (ctx, msg);
    }
}