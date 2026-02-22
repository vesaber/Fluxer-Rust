use async_trait::async_trait;
use crate::client::Context;
use crate::model::*;

#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn on_ready(&self, _ctx: Context, _ready: Ready) {}

    async fn on_message(&self, _ctx: Context, _msg: Message) {}

    async fn on_message_update(&self, _ctx: Context, _update: MessageUpdate) {}

    async fn on_message_delete(&self, _ctx: Context, _delete: MessageDelete) {}

    async fn on_message_delete_bulk(&self, _ctx: Context, _delete: MessageDeleteBulk) {}

    async fn on_reaction_add(&self, _ctx: Context, _reaction: ReactionAdd) {}

    async fn on_reaction_remove(&self, _ctx: Context, _reaction: ReactionRemove) {}

    async fn on_reaction_remove_all(&self, _ctx: Context, _event: ReactionRemoveAll) {}

    async fn on_reaction_remove_emoji(&self, _ctx: Context, _event: ReactionRemoveEmoji) {}

    async fn on_typing_start(&self, _ctx: Context, _event: TypingStart) {}

    async fn on_channel_create(&self, _ctx: Context, _channel: Channel) {}

    async fn on_channel_update(&self, _ctx: Context, _channel: Channel) {}

    async fn on_channel_delete(&self, _ctx: Context, _channel: Channel) {}

    async fn on_channel_pins_update(&self, _ctx: Context, _event: ChannelPinsUpdate) {}

    async fn on_guild_create(&self, _ctx: Context, _guild: Guild) {}

    async fn on_guild_update(&self, _ctx: Context, _guild: Guild) {}

    async fn on_guild_delete(&self, _ctx: Context, _guild: UnavailableGuild) {}

    async fn on_guild_member_add(&self, _ctx: Context, _event: GuildMemberAdd) {}

    async fn on_guild_member_update(&self, _ctx: Context, _event: GuildMemberUpdate) {}

    async fn on_guild_member_remove(&self, _ctx: Context, _event: GuildMemberRemove) {}

    async fn on_guild_ban_add(&self, _ctx: Context, _event: GuildBanAdd) {}

    async fn on_guild_ban_remove(&self, _ctx: Context, _event: GuildBanRemove) {}

    async fn on_guild_role_create(&self, _ctx: Context, _event: GuildRoleCreate) {}

    async fn on_guild_role_update(&self, _ctx: Context, _event: GuildRoleUpdate) {}

    async fn on_guild_role_delete(&self, _ctx: Context, _event: GuildRoleDelete) {}

    async fn on_guild_emojis_update(&self, _ctx: Context, _event: GuildEmojisUpdate) {}

    async fn on_guild_stickers_update(&self, _ctx: Context, _event: GuildStickersUpdate) {}

    async fn on_guild_role_update_bulk(&self, _ctx: Context, _event: GuildRoleUpdateBulk) {}

    async fn on_channel_update_bulk(&self, _ctx: Context, _event: ChannelUpdateBulk) {}

    async fn on_invite_create(&self, _ctx: Context, _event: InviteCreate) {}

    async fn on_invite_delete(&self, _ctx: Context, _event: InviteDelete) {}

    async fn on_webhooks_update(&self, _ctx: Context, _event: WebhooksUpdate) {}
}