//! # fluxer-rs
//!
//! Rust API wrapper for [Fluxer](https://fluxer.app). Handles the gateway
//! connection, heartbeating, reconnection, and event dispatch.
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use fluxer::prelude::*;
//! use async_trait::async_trait;
//!
//! struct MyHandler;
//!
//! #[async_trait]
//! impl EventHandler for MyHandler {
//!     async fn on_ready(&self, _ctx: Context, ready: Ready) {
//!         println!("Logged in as {}", ready.user.username);
//!     }
//!
//!     async fn on_message(&self, ctx: Context, msg: Message) {
//!         if msg.content.as_deref() == Some("!ping") {
//!             let channel_id = msg.channel_id.as_deref().unwrap_or_default();
//!             let _ = ctx.http.send_message(channel_id, "Pong!").await;
//!         }
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     rustls::crypto::ring::default_provider()
//!         .install_default()
//!         .expect("Failed to install rustls crypto provider");
//!
//!     let mut client = Client::builder("your-bot-token")
//!         .event_handler(MyHandler)
//!         .build();
//!
//!     client.start().await.expect("Client error");
//! }
//! ```
//!
//! # Rustls Crypto Provider
//!
//! You need to install a rustls crypto provider before creating the client, otherwise
//! it'll panic at runtime. Just add this at the top of `main()`:
//!
//! ```rust,no_run
//! rustls::crypto::ring::default_provider()
//!     .install_default()
//!     .expect("Failed to install rustls crypto provider");
//! ```
//!
//! This is because `rustls` 0.23+ doesn't auto-select a backend when both `ring` and
//! `aws-lc-rs` are available (livekit pulls in the latter).

pub mod client;
pub mod event;
pub mod error;
pub mod http;
pub mod model;
pub mod voice;

/// Re-exports the stuff you'll need most of the time so you can just `use fluxer::prelude::*;` and get going.
pub mod prelude {
    pub use crate::client::{Client, ClientBuilder, Context};
    pub use crate::error::ClientError;
    pub use crate::event::EventHandler;
    pub use crate::model::*;
    pub use crate::voice::FluxerVoiceConnection;
}