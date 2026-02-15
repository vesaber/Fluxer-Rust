use fluxer::prelude::*;
use async_trait::async_trait;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn on_message(&self, ctx: Context, msg: Message) {
        // ignores bot messages
        if msg.author.bot.unwrap_or(false) {
            return;
        }

        // responds to !ping
        if msg.content.trim() == "!ping" {
            let _ = ctx.http.send_message(&msg.channel_id, "pong").await;
        }
    }
}

#[tokio::main]
async fn main() {
    let token = std::env::var("FLUXERTOKEN")
        .expect("Expected FLUXERTOKEN in environment");

    let mut client = Client::builder(token)
        .event_handler(Handler)
        .build();

    println!("Bot starting...");
    
    if let Err(e) = client.start().await {
        eprintln!("Client error: {}", e);
    }
}