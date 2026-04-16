# Fluxer-Rust

A Rust written API Wrapper for the Discord alternative [Fluxer](https://fluxer.app/).

**Note: the crate is** `fluxer-rust` **but the library name is** `fluxer`, **so imports use** `use fluxer::prelude::*`.

```
cargo add fluxer-rust
```

## Usage

```rust
use fluxer::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn on_ready(&self, _ctx: Context, ready: Ready) {
        println!("Logged in as {}", ready.user.username);
    }

    async fn on_message(&self, ctx: Context, msg: Message) {
        if msg.content.as_deref() == Some("!ping") {
            let ch = msg.channel_id.as_deref().unwrap_or_default();
            let _ = ctx.http.send_message(ch, "Pong!").await;
        }
    }
}

#[tokio::main]
async fn main() {
    let token = std::env::var("FLUXER_TOKEN").expect("FLUXER_TOKEN not set");
    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .build();

    if let Err(e) = client.start().await {
        eprintln!("{}", e);
    }
}
```

## Voice

Requires ffmpeg. On Ubuntu/Debian you also need a few system packages:

```
sudo apt-get install -y pkg-config libglib2.0-dev libva-dev
```

```rust
let conn = ctx.join_voice(guild_id, channel_id).await?;
let handle = conn.play_music("track.mp3", ctx.http.clone(), channel_id.to_string()).await?;
handle.abort(); // stop playback
ctx.leave_voice(guild_id).await?;
```

## Selfbot

To connect as a user account rather than a bot, pass `.user_token()` to the builder.

```rust
Client::builder(&token).user_token().event_handler(Handler).build()
```

## Self-hosted Instances

To use the API of a self-hosted instance rather than fluxer's API, pass `.api_url("<API url>")` to the builder.

```rust
Client::builder(&token).api_url("http://localhost:48763/api/v1").event_handler(Handler).build()
```

## Examples

[Examples](examples/)

```
cargo run --example bot          # simple bot
cargo run --example voice        # voice join/play/stop
cargo run --example event_logger # logs all gateway events
cargo run --example selfbot      # user token selfbot
```

## License

Apache-2.0

## Donations

Monero: `46yingAoPTfECB2fXLQPiq9uwPezBQYjPhn3B4yScozaLiaiM4NpmRqS4MJ4Ja5gewJs5pLzV2RMV9USvwDNhQvbF6LExTU`\
Bitcoin: `bc1q5wudz9xqctkswrzphjc48kt5hurkz63e74jece`
