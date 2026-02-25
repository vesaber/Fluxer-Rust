use fluxer::prelude::*;
use async_trait::async_trait;
use tokio::sync::Mutex;
use tokio::task::AbortHandle;

const PREFIX: &str = "!";
const AUDIO_FILE: &str = "audio/audio.mp3";

struct Handler {
    playback: Mutex<Option<AbortHandle>>,
    voice: Mutex<Option<FluxerVoiceConnection>>,
}

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
        let guild_id = match msg.guild_id.as_deref() {
            Some(id) => id,
            None => return,
        };

        let (cmd, args) = match parse_command(content) {
            Some(v) => v,
            None => return,
        };

        match cmd {
            "join" => {
                if args.is_empty() {
                    let _ = ctx.http.send_message(channel_id, "`!join <voice_channel_id>`").await;
                    return;
                }

                match ctx.join_voice(guild_id, args).await {
                    Ok(conn) => {
                        *self.voice.lock().await = Some(conn);
                        let _ = ctx.http.send_message(channel_id, "Joined.").await;
                    }
                    Err(e) => {
                        let _ = ctx.http.send_message(channel_id, &format!("Failed: {}", e)).await;
                    }
                }
            }

            "leave" => {
                if let Some(handle) = self.playback.lock().await.take() {
                    handle.abort();
                }
                *self.voice.lock().await = None;
                let _ = ctx.leave_voice(guild_id).await;
                let _ = ctx.http.send_message(channel_id, "Left.").await;
            }

            "play" => {
                let conn = self.voice.lock().await;
                let conn = match conn.as_ref() {
                    Some(c) => c,
                    None => {
                        let _ = ctx.http.send_message(channel_id, "Not in a voice channel.").await;
                        return;
                    }
                };

                if let Some(handle) = self.playback.lock().await.take() {
                    handle.abort();
                }

                match conn.play_music(AUDIO_FILE, ctx.http.clone(), channel_id.to_string()).await {
                    Ok(handle) => {
                        *self.playback.lock().await = Some(handle);
                        let _ = ctx.http.send_message(channel_id, &format!("Playing `{}`.", AUDIO_FILE)).await;
                    }
                    Err(e) => {
                        let _ = ctx.http.send_message(channel_id, &format!("Failed: {}", e)).await;
                    }
                }
            }

            "stop" => {
                if let Some(handle) = self.playback.lock().await.take() {
                    handle.abort();
                    let _ = ctx.http.send_message(channel_id, "Stopped.").await;
                } else {
                    let _ = ctx.http.send_message(channel_id, "Nothing is playing.").await;
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

    let handler = Handler {
        playback: Mutex::new(None),
        voice: Mutex::new(None),
    };

    let mut client = Client::builder(&token)
        // .api_url("http://localhost:48763/api/v1") this is for self hosted instances
        .event_handler(handler)
        .build();

    if let Err(e) = client.start().await {
        eprintln!("Error: {}", e);
    }
}