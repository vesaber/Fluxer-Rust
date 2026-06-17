use fluxer::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn on_ready(&self, _ctx: Context, ready: Ready) {
        println!(
            "[ready] logged in as {} (id: {})",
            ready.user.username, ready.user.id
        );
        println!("[ready] session_id={}", ready.session_id);
        if let Some(guilds) = &ready.guilds {
            println!("[ready] {} guild(s) incoming", guilds.len());
        }
    }

    async fn on_message(&self, _ctx: Context, msg: Message) {
        println!(
            "[message_create] {}#{} in channel={} guild={} | {:?}",
            msg.author.username,
            msg.author.discriminator.as_deref().unwrap_or("0"),
            msg.channel_id.as_deref().unwrap_or("?"),
            msg.guild_id.as_deref().unwrap_or("DM"),
            msg.content.as_deref().unwrap_or("")
        );
    }

    async fn on_message_update(&self, _ctx: Context, e: MessageUpdate) {
        println!(
            "[message_update] id={} channel={} | {:?}",
            e.id,
            e.channel_id.as_deref().unwrap_or("?"),
            e.content.as_deref().unwrap_or("(no content change)")
        );
    }

    async fn on_message_delete(&self, _ctx: Context, e: MessageDelete) {
        println!(
            "[message_delete] id={} channel={}",
            e.id,
            e.channel_id.as_deref().unwrap_or("?")
        );
    }

    async fn on_message_delete_bulk(&self, _ctx: Context, e: MessageDeleteBulk) {
        println!(
            "[message_delete_bulk] {} messages in channel={}",
            e.ids.len(),
            e.channel_id.as_deref().unwrap_or("?")
        );
    }

    async fn on_reaction_add(&self, _ctx: Context, e: ReactionAdd) {
        println!(
            "[reaction_add] user={} emoji={} message={} channel={}",
            e.user_id,
            e.emoji.to_reaction_string(),
            e.message_id,
            e.channel_id.as_deref().unwrap_or("?")
        );
    }

    async fn on_reaction_remove(&self, _ctx: Context, e: ReactionRemove) {
        println!(
            "[reaction_remove] user={} emoji={} message={}",
            e.user_id,
            e.emoji.to_reaction_string(),
            e.message_id
        );
    }

    async fn on_reaction_remove_all(&self, _ctx: Context, e: ReactionRemoveAll) {
        println!(
            "[reaction_remove_all] message={} channel={}",
            e.message_id,
            e.channel_id.as_deref().unwrap_or("?")
        );
    }

    async fn on_reaction_remove_emoji(&self, _ctx: Context, e: ReactionRemoveEmoji) {
        println!(
            "[reaction_remove_emoji] emoji={} message={}",
            e.emoji.to_reaction_string(),
            e.message_id
        );
    }

    async fn on_message_reaction_add_many(&self, _ctx: Context, e: MessageReactionAddMany) {
        println!(
            "[reaction_add_many] {} reactions on message={} channel={}",
            e.reactions.len(),
            e.message_id,
            e.channel_id.as_deref().unwrap_or("?")
        );
    }

    async fn on_typing_start(&self, _ctx: Context, e: TypingStart) {
        println!(
            "[typing_start] user={} channel={} guild={}",
            e.user_id,
            e.channel_id.as_deref().unwrap_or("?"),
            e.guild_id.as_deref().unwrap_or("DM")
        );
    }

    async fn on_guild_create(&self, _ctx: Context, guild: Guild) {
        println!(
            "[guild_create] {} (id={}) members={:?}",
            guild.name.as_deref().unwrap_or("?"),
            guild.id,
            guild.member_count
        );
    }

    async fn on_guild_update(&self, _ctx: Context, guild: Guild) {
        println!(
            "[guild_update] {} (id={})",
            guild.name.as_deref().unwrap_or("?"),
            guild.id
        );
    }

    async fn on_guild_delete(&self, _ctx: Context, guild: UnavailableGuild) {
        let reason = if guild.unavailable.unwrap_or(false) {
            "outage"
        } else {
            "removed"
        };
        println!("[guild_delete] id={} reason={}", guild.id, reason);
    }

    async fn on_channel_create(&self, _ctx: Context, ch: Channel) {
        println!(
            "[channel_create] #{} (id={}) guild={}",
            ch.name.as_deref().unwrap_or("?"),
            ch.id,
            ch.guild_id.as_deref().unwrap_or("DM")
        );
    }

    async fn on_channel_update(&self, _ctx: Context, ch: Channel) {
        println!(
            "[channel_update] #{} (id={})",
            ch.name.as_deref().unwrap_or("?"),
            ch.id
        );
    }

    async fn on_channel_delete(&self, _ctx: Context, ch: Channel) {
        println!(
            "[channel_delete] #{} (id={})",
            ch.name.as_deref().unwrap_or("?"),
            ch.id
        );
    }

    async fn on_channel_pins_update(&self, _ctx: Context, e: ChannelPinsUpdate) {
        println!(
            "[channel_pins_update] channel={} guild={}",
            e.channel_id.as_deref().unwrap_or("?"),
            e.guild_id.as_deref().unwrap_or("DM")
        );
    }

    async fn on_channel_recipient_add(&self, _ctx: Context, e: ChannelRecipientAdd) {
        println!(
            "[channel_recipient_add] {} joined channel={}",
            e.user.username, e.channel_id
        );
    }

    async fn on_channel_recipient_remove(&self, _ctx: Context, e: ChannelRecipientRemove) {
        println!(
            "[channel_recipient_remove] {} left channel={}",
            e.user.username, e.channel_id
        );
    }

    async fn on_guild_member_add(&self, _ctx: Context, e: GuildMemberAdd) {
        let name = e
            .member
            .user
            .as_ref()
            .map(|u| u.username.as_str())
            .unwrap_or("?");
        println!("[guild_member_add] {} joined guild={}", name, e.guild_id);
    }

    async fn on_guild_member_update(&self, _ctx: Context, e: GuildMemberUpdate) {
        println!(
            "[guild_member_update] user={} guild={} nick={:?} roles={}",
            e.user.id,
            e.guild_id,
            e.nick,
            e.roles.len()
        );
    }

    async fn on_guild_member_remove(&self, _ctx: Context, e: GuildMemberRemove) {
        println!(
            "[guild_member_remove] {} left/kicked from guild={}",
            e.user.username, e.guild_id
        );
    }

    async fn on_guild_member_list_update(&self, _ctx: Context, e: GuildMemberListUpdate) {
        println!(
            "[guild_member_list_update] guild={} online={} ops={}",
            e.guild_id,
            e.online_count.unwrap_or(0),
            e.ops.len()
        );
    }

    async fn on_guild_ban_add(&self, _ctx: Context, e: GuildBanAdd) {
        println!(
            "[guild_ban_add] {} banned from guild={}",
            e.user.username, e.guild_id
        );
    }

    async fn on_guild_ban_remove(&self, _ctx: Context, e: GuildBanRemove) {
        println!(
            "[guild_ban_remove] {} unbanned from guild={}",
            e.user.username, e.guild_id
        );
    }

    async fn on_guild_role_create(&self, _ctx: Context, e: GuildRoleCreate) {
        println!(
            "[guild_role_create] \"{}\" (id={}) in guild={}",
            e.role.name, e.role.id, e.guild_id
        );
    }

    async fn on_guild_role_update(&self, _ctx: Context, e: GuildRoleUpdate) {
        println!(
            "[guild_role_update] \"{}\" (id={}) in guild={}",
            e.role.name, e.role.id, e.guild_id
        );
    }

    async fn on_guild_role_delete(&self, _ctx: Context, e: GuildRoleDelete) {
        println!(
            "[guild_role_delete] role={} guild={}",
            e.role_id, e.guild_id
        );
    }

    async fn on_guild_emojis_update(&self, _ctx: Context, e: GuildEmojisUpdate) {
        println!(
            "[guild_emojis_update] {} emoji(s) in guild={}",
            e.emojis.len(),
            e.guild_id
        );
    }

    async fn on_guild_stickers_update(&self, _ctx: Context, e: GuildStickersUpdate) {
        println!(
            "[guild_stickers_update] {} sticker(s) in guild={}",
            e.stickers.len(),
            e.guild_id
        );
    }

    async fn on_invite_create(&self, _ctx: Context, e: InviteCreate) {
        println!(
            "[invite_create] code={:?} channel={} guild={}",
            e.code,
            e.channel_id.as_deref().unwrap_or("?"),
            e.guild_id.as_deref().unwrap_or("DM")
        );
    }

    async fn on_invite_delete(&self, _ctx: Context, e: InviteDelete) {
        println!(
            "[invite_delete] code={} channel={} guild={}",
            e.code,
            e.channel_id.as_deref().unwrap_or("?"),
            e.guild_id.as_deref().unwrap_or("DM")
        );
    }

    async fn on_webhooks_update(&self, _ctx: Context, e: WebhooksUpdate) {
        println!(
            "[webhooks_update] channel={} guild={}",
            e.channel_id,
            e.guild_id.as_deref().unwrap_or("?")
        );
    }

    async fn on_presence_update(&self, _ctx: Context, e: PresenceUpdate) {
        let status = e.status.as_deref().unwrap_or("?");
        let mobile = if e.mobile.unwrap_or(false) {
            " [mobile]"
        } else {
            ""
        };
        let afk = if e.afk.unwrap_or(false) { " [afk]" } else { "" };
        let custom = match &e.custom_status {
            Some(cs) => {
                let text = cs.text.as_deref().unwrap_or("");
                let emoji = cs.emoji_name.as_deref().unwrap_or("");
                if text.is_empty() && emoji.is_empty() {
                    String::new()
                } else if emoji.is_empty() {
                    format!(" | \"{}\"", text)
                } else if text.is_empty() {
                    format!(" | {}", emoji)
                } else {
                    format!(" | {} \"{}\"", emoji, text)
                }
            }
            None => String::new(),
        };
        println!(
            "[presence_update] {} → {}{}{}{}",
            e.user.username, status, mobile, afk, custom
        );
    }

    async fn on_presence_update_bulk(&self, _ctx: Context, e: PresenceUpdateBulk) {
        println!(
            "[presence_update_bulk] {} update(s) guild={}",
            e.presences.len(),
            e.guild_id.as_deref().unwrap_or("?")
        );
    }

    async fn on_user_update(&self, _ctx: Context, e: UserUpdate) {
        println!(
            "[user_update] {}#{} (id={})",
            e.username,
            e.discriminator.as_deref().unwrap_or("0"),
            e.id
        );
    }

    async fn on_user_settings_update(&self, _ctx: Context, _e: UserSettingsUpdate) {
        println!("[user_settings_update]");
    }

    async fn on_message_ack(&self, _ctx: Context, e: MessageAck) {
        println!(
            "[message_ack] channel={} message={}",
            e.channel_id.as_deref().unwrap_or("?"),
            e.message_id.as_deref().unwrap_or("?")
        );
    }

    async fn on_sessions_replace(&self, _ctx: Context, e: SessionsReplace) {
        println!("[sessions_replace] {} session(s):", e.0.len());
        for s in &e.0 {
            let mobile = if s.mobile.unwrap_or(false) {
                " [mobile]"
            } else {
                ""
            };
            let afk = if s.afk.unwrap_or(false) { " [afk]" } else { "" };
            println!(
                "  {} → {}{}{}",
                s.session_id,
                s.status.as_deref().unwrap_or("?"),
                mobile,
                afk
            );
        }
    }

    async fn on_relationship_add(&self, _ctx: Context, e: RelationshipAdd) {
        let kind = match e.kind {
            1 => "friend",
            2 => "blocked",
            3 => "incoming request",
            4 => "outgoing request",
            _ => "unknown",
        };
        println!("[relationship_add] id={} type={}", e.id, kind);
    }

    async fn on_relationship_update(&self, _ctx: Context, e: RelationshipUpdate) {
        let kind = match e.kind {
            1 => "friend",
            2 => "blocked",
            3 => "incoming request",
            4 => "outgoing request",
            _ => "unknown",
        };
        println!("[relationship_update] id={} type={}", e.id, kind);
    }

    async fn on_relationship_remove(&self, _ctx: Context, e: RelationshipRemove) {
        println!("[relationship_remove] id={}", e.id);
    }

    async fn on_call_create(&self, _ctx: Context, e: CallCreate) {
        println!(
            "[call_create] channel={} ringing={} in_call={}",
            e.channel_id,
            e.ringing.as_ref().map(|r| r.len()).unwrap_or(0),
            e.voice_states.as_ref().map(|v| v.len()).unwrap_or(0)
        );
    }

    async fn on_call_update(&self, _ctx: Context, e: CallUpdate) {
        println!(
            "[call_update] channel={} ringing={} in_call={}",
            e.channel_id,
            e.ringing.as_ref().map(|r| r.len()).unwrap_or(0),
            e.voice_states.as_ref().map(|v| v.len()).unwrap_or(0)
        );
    }

    async fn on_call_delete(&self, _ctx: Context, e: CallDelete) {
        println!("[call_delete] channel={}", e.channel_id);
    }

    async fn on_passive_updates(&self, _ctx: Context, e: PassiveUpdates) {
        println!(
            "[passive_updates] guild={} created={} updated={} deleted={}",
            e.guild_id,
            e.created_channels.as_ref().map(|v| v.len()).unwrap_or(0),
            e.updated_channels.as_ref().map(|v| v.len()).unwrap_or(0),
            e.deleted_channel_ids.as_ref().map(|v| v.len()).unwrap_or(0)
        );
    }

    async fn on_guild_sync(&self, _ctx: Context, e: GuildSync) {
        println!(
            "[guild_sync] guild={} members={} channels={} online={}",
            e.id,
            e.member_count.unwrap_or(0),
            e.channels.as_ref().map(|v| v.len()).unwrap_or(0),
            e.online_count.unwrap_or(0)
        );
    }

    async fn on_guild_members_chunk(&self, _ctx: Context, e: GuildMembersChunk) {
        println!(
            "[guild_members_chunk] guild={} chunk={}/{} members={}",
            e.guild_id,
            e.chunk_index + 1,
            e.chunk_count,
            e.members.len()
        );
    }

    async fn on_voice_state_update(&self, _ctx: Context, e: VoiceStateUpdate) {
        println!("[voice_state_update] user={} guild={} channel={} self_mute={:?} self_deaf={:?} stream={:?} video={:?} server_mute={:?} server_deaf={:?} mobile={:?} connection={} version={:?} stream_keys={:?}",
            e.user_id,
            e.guild_id.as_deref().unwrap_or("DM"),
            e.channel_id.as_deref().unwrap_or("none"),
            e.self_mute,
            e.self_deaf,
            e.self_stream,
            e.self_video,
            e.mute,
            e.deaf,
            e.is_mobile,
            e.connection_id.as_deref().unwrap_or("?"),
            e.version,
            e.viewer_stream_keys);
    }

    async fn on_voice_server_update(&self, _ctx: Context, e: VoiceServerUpdate) {
        println!(
            "[voice_server_update] guild={} endpoint={}",
            e.guild_id.as_deref().unwrap_or("DM"),
            e.endpoint.as_deref().unwrap_or("?")
        );
    }

    async fn on_resumed(&self, _ctx: Context, _e: Resumed) {
        println!("[resumed] session resumed successfully");
    }

    async fn on_channel_pins_ack(&self, _ctx: Context, e: ChannelPinsAck) {
        println!("[channel_pins_ack] channel={}", e.channel_id);
    }

    async fn on_user_pinned_dms_update(&self, _ctx: Context, e: UserPinnedDmsUpdate) {
        println!("[user_pinned_dms_update] {} pinned DMs", e.pinned.len());
    }

    async fn on_user_note_update(&self, _ctx: Context, e: UserNoteUpdate) {
        println!("[user_note_update] user={} note={}", e.id, e.note);
    }

    async fn on_user_connections_update(&self, _ctx: Context, _e: UserConnectionsUpdate) {
        println!("[user_connections_update]");
    }

    async fn on_user_guild_settings_update(&self, _ctx: Context, e: UserGuildSettingsUpdate) {
        println!(
            "[user_guild_settings_update] guild={}",
            e.guild_id.as_deref().unwrap_or("?")
        );
    }

    async fn on_auth_session_change(&self, _ctx: Context, _e: AuthSessionChange) {
        println!("[auth_session_change]");
    }

    async fn on_saved_message_create(&self, _ctx: Context, e: SavedMessageCreate) {
        println!("[saved_message_create] message={}", e.message.id);
    }

    async fn on_saved_message_delete(&self, _ctx: Context, e: SavedMessageDelete) {
        println!("[saved_message_delete] message={}", e.message_id);
    }

    async fn on_recent_mention_delete(&self, _ctx: Context, e: RecentMentionDelete) {
        println!("[recent_mention_delete] message={}", e.message_id);
    }

    async fn on_guild_role_update_bulk(&self, _ctx: Context, e: GuildRoleUpdateBulk) {
        println!(
            "[guild_role_update_bulk] guild={} {} role(s)",
            e.guild_id,
            e.roles.len()
        );
    }

    async fn on_channel_update_bulk(&self, _ctx: Context, e: ChannelUpdateBulk) {
        println!(
            "[channel_update_bulk] guild={} {} channel(s)",
            e.guild_id.as_deref().unwrap_or("?"),
            e.channels.len()
        );
    }

    async fn on_guild_audit_log_entry_create(&self, _ctx: Context, e: GuildAuditLogEntryCreate) {
        println!("[guild_audit_log_entry_create] id={} guild={} action={:?} user={} target={} reason={:?}",
            e.id,
            e.guild_id.as_deref().unwrap_or("?"),
            e.action_type,
            e.user_id.as_deref().unwrap_or("?"),
            e.target_id.as_deref().unwrap_or("?"),
            e.reason.as_deref().unwrap_or("none"));
        if let Some(changes) = &e.changes {
            println!(
                "  changes: {}",
                serde_json::to_string(changes).unwrap_or_default()
            );
        }
        if let Some(options) = &e.options {
            println!(
                "  options: {}",
                serde_json::to_string(options).unwrap_or_default()
            );
        }
    }

    async fn on_unknown_event(&self, _ctx: Context, event_type: String, data: serde_json::Value) {
        println!(
            "[unknown] {} | {}",
            event_type,
            serde_json::to_string_pretty(&data).unwrap_or_default()
        );
    }
}

#[tokio::main]
async fn main() {
    let token = std::env::var("FLUXER_TOKEN").expect("Set FLUXER_TOKEN to your token");

    let mut client = Client::builder(&token)
        // .user_token() // Uncomment if using a user token and not a bot token
        .event_handler(Handler)
        .build();

    if let Err(e) = client.start().await {
        eprintln!("Error: {}", e);
    }
}
