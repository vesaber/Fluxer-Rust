pub mod client;
pub mod event;
pub mod error;
pub mod http;
pub mod model;
pub mod voice;

pub mod prelude {
    pub use crate::client::{Client, ClientBuilder, Context};
    pub use crate::error::ClientError;
    pub use crate::event::EventHandler;
    pub use crate::model::*;
    pub use crate::voice::FluxerVoiceConnection;
}