use fluxer::prelude::*;
use async_trait::async_trait;
use std::time::Instant;

const PREFIX: &str = "!";

struct Handler;

fn parse_command(content: &str) -> Option<(&str, &str)> {
    let trimmed = content.strip_prefix(PREFIX)?;
    match trimmed.find(' ') {
        Some(pos) => Some((&trimmed[..pos], trimmed[pos + 1..].trim())),
        None => Some((trimmed, "")),
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn on_ready(&self, _ctx: Context, ready: Ready) {
        println!("Logged in as {}", ready.user.username);
    }

    async fn on_message(&self, ctx: Context, msg: Message) {
        if msg.author.bot.unwrap_or(false) {
            return;
        }

        let content = match msg.content.as_deref() {
            Some(c) => c,
            None => return,
        };

        let channel_id = msg.channel_id.as_deref().unwrap_or_default();

        let (cmd, args) = match parse_command(content) {
            Some(v) => v,
            None => return,
        };

        match cmd {
            "ping" => {
                let start = Instant::now();
                let sent = ctx.http.send_message(channel_id, "Pong!").await;
                let elapsed = start.elapsed().as_millis();

                if let Ok(sent) = sent {
                    let _ = ctx.http.edit_message(
                        channel_id,
                        &sent.id,
                        &format!("Pong! {}ms", elapsed),
                    ).await;
                }
            }

            "say" => {
                if args.is_empty() {
                    let _ = ctx.http.send_message(channel_id, "Say what?").await;
                    return;
                }
                let _ = ctx.http.delete_message(channel_id, &msg.id).await;
                let _ = ctx.http.send_message(channel_id, args).await;
            }

            "embed" => {
                let (title, desc) = match args.split_once('|') {
                    Some((t, d)) => (t.trim(), d.trim()),
                    None => {
                        let _ = ctx.http.send_message(channel_id, "`!embed title | description`").await;
                        return;
                    }
                };

                let embed = EmbedBuilder::new()
                    .title(title)
                    .description(desc)
                    .color(0x5865F2)
                    .build();

                let _ = ctx.http.send_embed(channel_id, None, vec![embed]).await;
            }

            "react" => {
                let _ = ctx.http.add_reaction(channel_id, &msg.id, "❤️").await;
            }

            "purge" => {
                let count: u8 = args.parse().unwrap_or(0);
                if count == 0 || count > 100 {
                    let _ = ctx.http.send_message(channel_id, "1-100.").await;
                    return;
                }

                let query = GetMessagesQuery {
                    limit: Some(count),
                    ..Default::default()
                };

                if let Ok(messages) = ctx.http.get_messages(channel_id, query).await {
                    let ids: Vec<&str> = messages.iter().map(|m| m.id.as_str()).collect();
                    let _ = ctx.http.bulk_delete_messages(channel_id, ids).await;
                }
            }

            "serverinfo" => {
                let guild_id = match &msg.guild_id {
                    Some(id) => id.as_str(),
                    None => return,
                };

                if let Ok(guild) = ctx.http.get_guild(guild_id).await {
                    let name = guild.name.as_deref().unwrap_or("Unknown");

                    let members = ctx.http.get_guild_members(guild_id, Some(1000), None).await
                        .map(|m| m.len().to_string())
                        .unwrap_or("?".into());

                    let embed = EmbedBuilder::new()
                        .title(name)
                        .field("Members", &members, true)
                        .color(0x5865F2)
                        .build();

                    let _ = ctx.http.send_embed(channel_id, None, vec![embed]).await;
                }
            }

            _ => {}
        }
    }
}

#[tokio::main]
async fn main() {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let token = std::env::var("FLUXER_TOKEN")
        .expect("Set FLUXER_TOKEN to your bot token");

    let mut client = Client::builder(&token)
        // .api_url("http://localhost:48763/api/v1") this is for self hosted instances
        .event_handler(Handler)
        .build();

    if let Err(e) = client.start().await {
        eprintln!("Error: {}", e);
    }
}