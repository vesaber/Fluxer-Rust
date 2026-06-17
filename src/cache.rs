//! In-memory cache populated automatically from gateway events.
//! Attached to every [`Context`](crate::client::Context) as `ctx.cache`.
//!
//! ```rust,no_run
//! # use fluxer::prelude::*;
//! # async fn example(ctx: Context, msg: Message) {
//! if let Some(guild) = ctx.cache.guild(msg.guild_id.as_deref().unwrap_or("")).await {
//!     println!("Guild name: {}", guild.name.unwrap_or_default());
//! }
//! println!("Bot is in {} guild(s)", ctx.cache.guild_count().await);
//! # }
//! ```

use crate::model::{Channel, Guild, Snowflake, User};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// In-memory gateway cache.
pub struct Cache {
    pub guilds: RwLock<HashMap<Snowflake, Guild>>,
    pub channels: RwLock<HashMap<Snowflake, Channel>>,
    pub users: RwLock<HashMap<Snowflake, User>>,
    pub current_user: RwLock<Option<User>>,
}

impl Cache {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            guilds: RwLock::new(HashMap::new()),
            channels: RwLock::new(HashMap::new()),
            users: RwLock::new(HashMap::new()),
            current_user: RwLock::new(None),
        })
    }

    /// Looks up a guild by ID. Populated from `GUILD_CREATE` / `GUILD_UPDATE` events.
    pub async fn guild(&self, id: &str) -> Option<Guild> {
        self.guilds.read().await.get(id).cloned()
    }

    /// Looks up a channel by ID. Populated from `GUILD_CREATE` and `CHANNEL_CREATE` / `CHANNEL_UPDATE` events.
    pub async fn channel(&self, id: &str) -> Option<Channel> {
        self.channels.read().await.get(id).cloned()
    }

    /// Looks up a user by ID. Populated from `GUILD_CREATE` member lists and `MESSAGE_CREATE` authors.
    pub async fn user(&self, id: &str) -> Option<User> {
        self.users.read().await.get(id).cloned()
    }

    /// Returns the bot's own `User` object, populated from the READY event.
    pub async fn current_user(&self) -> Option<User> {
        self.current_user.read().await.clone()
    }

    /// Returns the number of guilds currently in cache.
    pub async fn guild_count(&self) -> usize {
        self.guilds.read().await.len()
    }
}
