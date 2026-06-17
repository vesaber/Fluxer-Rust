use fluxer::prelude::*;
use std::time::Instant;

const PREFIX: &str = "sb!";

struct Handler;

fn parse_command(content: &str) -> Option<(&str, &str)> {
    let trimmed = content.strip_prefix(PREFIX)?;
    match trimmed.find(' ') {
        Some(pos) => Some((&trimmed[..pos], trimmed[pos + 1..].trim())),
        None => Some((trimmed, "")),
    }
}

async fn is_me(ctx: &Context, msg: &Message) -> bool {
    ctx.cache
        .current_user()
        .await
        .map(|u| u.id == msg.author.id)
        .unwrap_or(false)
}

#[async_trait]
impl EventHandler for Handler {
    async fn on_ready(&self, _ctx: Context, ready: Ready) {
        println!("[ready] logged in as {}", ready.user.username);
    }

    async fn on_message(&self, ctx: Context, msg: Message) {
        if !is_me(&ctx, &msg).await {
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
                    let _ = ctx
                        .http
                        .edit_message(channel_id, &sent.id, &format!("Pong! `{}ms`", elapsed))
                        .await;
                }
            }

            "8ball" => {
                if args.is_empty() {
                    let _ = ctx
                        .http
                        .edit_message(channel_id, &msg.id, "Ask a question!")
                        .await;
                    return;
                }
                let responses = [
                    "Yes",
                    "No",
                    "Maybe",
                    "Definitely",
                    "Absolutely not",
                    "Ask again later",
                    "Better not tell you now",
                    "Cannot predict now",
                    "Don't count on it",
                    "It is certain",
                    "Most likely",
                    "Outlook good",
                    "Reply hazy, try again",
                    "Signs point to yes",
                    "Very doubtful",
                    "Without a doubt",
                ];
                let idx = (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
                    % responses.len() as u128) as usize;
                let answer = responses[idx];
                let _ = ctx.http.edit_message(channel_id, &msg.id, answer).await;
            }

            "flip" => {
                let result = if std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
                    .is_multiple_of(2)
                {
                    "Heads"
                } else {
                    "Tails"
                };
                let _ = ctx.http.edit_message(channel_id, &msg.id, result).await;
            }

            "roll" => {
                let sides: u32 = args.parse().unwrap_or(6);
                if !(2..=100).contains(&sides) {
                    let _ = ctx
                        .http
                        .edit_message(channel_id, &msg.id, "Use 2-100 sides")
                        .await;
                    return;
                }
                let result = (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
                    % sides as u128) as u32
                    + 1;
                let _ = ctx
                    .http
                    .edit_message(channel_id, &msg.id, &format!("Rolled {}/{}", result, sides))
                    .await;
            }

            "purge" => {
                let count: u8 = args.parse().unwrap_or(10).min(100);
                let query = GetMessagesQuery {
                    limit: Some(count),
                    before: None,
                    after: None,
                    around: None,
                };
                match ctx.http.get_messages(channel_id, query).await {
                    Ok(messages) => {
                        let my_id = ctx
                            .cache
                            .current_user()
                            .await
                            .map(|u| u.id)
                            .unwrap_or_default();
                        let my_messages: Vec<_> = messages
                            .iter()
                            .filter(|m| m.author.id == my_id)
                            .map(|m| m.id.as_str())
                            .collect();

                        if !my_messages.is_empty() {
                            let _ = ctx.http.bulk_delete_messages(channel_id, my_messages).await;
                        }
                    }
                    Err(e) => eprintln!("[purge] failed: {}", e),
                }
            }

            "embed" => {
                if args.is_empty() {
                    let _ = ctx
                        .http
                        .edit_message(channel_id, &msg.id, "Usage: sb!embed <text>")
                        .await;
                    return;
                }
                let embed = EmbedBuilder::new()
                    .description(args)
                    .color(0x5865F2)
                    .build();

                let _ = ctx.http.delete_message(channel_id, &msg.id).await;
                let _ = ctx.http.send_embed(channel_id, None, vec![embed]).await;
            }

            "edit" => {
                if args.is_empty() {
                    return;
                }
                let _ = ctx.http.edit_message(channel_id, &msg.id, args).await;
            }

            "help" => {
                let help_text = "**Selfbot Commands**\n\
                    `sb!ping` - Check latency\n\
                    `sb!8ball <question>` - Ask the magic 8ball\n\
                    `sb!flip` - Flip a coin\n\
                    `sb!roll [sides]` - Roll a die (default 6)\n\
                    `sb!purge [count]` - Delete your recent messages\n\
                    `sb!embed <text>` - Send an embed\n\
                    `sb!edit <text>` - Edit your message\n\
                    `sb!help` - Show this message";
                let _ = ctx.http.edit_message(channel_id, &msg.id, help_text).await;
            }

            _ => {}
        }
    }
}

#[tokio::main]
async fn main() {
    let token = std::env::var("FLUXER_TOKEN").expect("Set FLUXER_TOKEN to your user token");

    let mut client = Client::builder(&token)
        .user_token()
        .event_handler(Handler)
        .build();

    if let Err(e) = client.start().await {
        eprintln!("Error: {}", e);
    }
}
