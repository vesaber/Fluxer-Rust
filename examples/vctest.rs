use async_trait::async_trait;
use fluxer::prelude::*;

struct VoiceBot;

#[async_trait]
impl EventHandler for VoiceBot {
    async fn on_message(&self, ctx: Context, msg: Message) {
        if msg.author.bot.unwrap_or(false) {
            return;
        }

        if !msg.content.starts_with("!play ") {
            return;
        }

        let parts: Vec<&str> = msg.content.splitn(3, ' ').collect();
        if parts.len() < 3 {
            let _ = ctx.http
                .send_message(&msg.channel_id, "Usage: !play <channel_id> <file_location>")
                .await;
            return;
        }

        let channel_id = parts[1];
        let audio = parts[2];

        let guild_id = match &msg.guild_id {
            Some(id) => id,
            None => {
                let _ = ctx.http
                    .send_message(&msg.channel_id, "This command must be used in a guild")
                    .await;
                return;
            }
        };

        let conn = match ctx.join_voice(guild_id, channel_id).await {
            Ok(c) => c,
            Err(e) => {
                let _ = ctx.http
                    .send_message(&msg.channel_id, &format!("Voice join failed: {}", e))
                    .await;
                return;
            }
        };

        if let Err(e) = conn.play_music(
            audio,
            ctx.http.clone(),
            msg.channel_id.clone(),
        ).await {
            let _ = ctx.http
                .send_message(&msg.channel_id, &format!("Playback error: {}", e))
                .await;
            return;
        }

        let _ = ctx.http
            .send_message(&msg.channel_id, &format!("Streaming `{}`...", audio))
            .await;
    }
}

#[tokio::main]
async fn main() {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let token = std::env::var("FLUXERTOKEN").expect("FLUXERTOKEN not set");

    let mut client = Client::builder(token)
        .event_handler(VoiceBot)
        .build();

    println!("Bot starting...");
    if let Err(e) = client.start().await {
        eprintln!("Client error: {}", e);
    }
}